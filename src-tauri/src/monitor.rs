use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::system::{cpu, gpu, memory, sensors};

pub const HISTORY_LIMIT: usize = 600;
pub const DEFAULT_INTERVAL_MS: u64 = 2000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricSample {
    pub timestamp_ms: u64,
    pub cpu_percent: Option<f32>,
    pub cpu_freq_avg_mhz: Option<u32>,
    pub mem_used_percent: Option<f32>,
    pub mem_used_kb: Option<u64>,
    pub mem_total_kb: Option<u64>,
    pub swap_used_percent: Option<f32>,
    pub temp_max_c: Option<f32>,
    pub temp_label: Option<String>,
    pub gpu_utilization_percent: Option<f32>,
    pub gpu_mem_used_mb: Option<f32>,
    pub gpu_mem_total_mb: Option<f32>,
    pub gpu_temp_c: Option<f32>,
}

#[derive(Debug, Default)]
pub struct MonitorInner {
    running: AtomicBool,
    generation: AtomicU64,
    lifecycle: Mutex<()>,
    history: Mutex<VecDeque<MetricSample>>,
}

pub type MonitorState = Arc<MonitorInner>;

fn generation_is_active(state: &MonitorInner, generation: u64) -> bool {
    state.running.load(Ordering::SeqCst)
        && state.generation.load(Ordering::SeqCst) == generation
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn collect_sample() -> MetricSample {
    let mem = memory::detect();

    let mem_total_kb = mem.as_ref().and_then(|m| m.total_kb);
    let mem_used_kb = mem.as_ref().and_then(|m| m.used_kb);

    let mem_used_percent = match (mem_used_kb, mem_total_kb) {
        (Some(used), Some(total)) if total > 0 => Some((used as f32 / total as f32) * 100.0),
        _ => None,
    };

    let swap_used_percent = match (
        mem.as_ref().and_then(|m| m.swap_used_kb),
        mem.as_ref().and_then(|m| m.swap_total_kb),
    ) {
        (Some(used), Some(total)) if total > 0 => Some((used as f32 / total as f32) * 100.0),
        _ => None,
    };

    let temps = sensors::detect();
    let hottest = temps
        .iter()
        .filter(|t| t.celsius > 0.0)
        .max_by(|a, b| a.celsius.total_cmp(&b.celsius));

    let gpu_metrics = gpu::metrics();

    MetricSample {
        timestamp_ms: now_ms(),
        cpu_percent: cpu::usage_percent(),
        cpu_freq_avg_mhz: cpu_avg_freq(),
        mem_used_percent,
        mem_used_kb,
        mem_total_kb,
        swap_used_percent,
        temp_max_c: hottest.map(|t| t.celsius),
        temp_label: hottest.map(|t| format!("{}·{}", t.chip, t.label)),
        gpu_utilization_percent: gpu_metrics.as_ref().and_then(|g| g.utilization_percent),
        gpu_mem_used_mb: gpu_metrics.as_ref().and_then(|g| g.mem_used_mb),
        gpu_mem_total_mb: gpu_metrics.as_ref().and_then(|g| g.mem_total_mb),
        gpu_temp_c: gpu_metrics.as_ref().and_then(|g| g.temp_c),
    }
}

fn cpu_avg_freq() -> Option<u32> {
    let mut freqs: Vec<u32> = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/sys/devices/system/cpu") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !(name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit())) {
                continue;
            }
            if let Ok(raw) = std::fs::read_to_string(entry.path().join("cpufreq/scaling_cur_freq"))
            {
                let khz: u32 = raw.trim().parse().unwrap_or(0);
                if khz > 0 {
                    freqs.push(khz / 1000);
                }
            }
        }
    }

    if freqs.is_empty() {
        None
    } else {
        let sum: u64 = freqs.iter().map(|f| *f as u64).sum();
        Some((sum / freqs.len() as u64) as u32)
    }
}

/// eslint-disable: cada coleta acontece na thread de fundo, nunca na interface.
#[tauri::command]
pub fn start_monitoring(
    app: AppHandle,
    state: State<'_, MonitorState>,
    interval_ms: u64,
) -> Result<(), String> {
    let interval = interval_ms.max(500);
    let _lifecycle = state
        .lifecycle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // Chamadas repetidas não devem iniciar outro trabalhador.
    if state.running.load(Ordering::SeqCst) {
        return Ok(());
    }

    let generation = state
        .generation
        .fetch_add(1, Ordering::SeqCst)
        .wrapping_add(1);
    state.running.store(true, Ordering::SeqCst);

    let shared = Arc::clone(&state);
    let app_handle = app.clone();

    std::thread::Builder::new()
        .name("gamefic-monitor".to_string())
        .spawn(move || {
            loop {
                if !generation_is_active(&shared, generation) {
                    break;
                }

                let sample = collect_sample();

                {
                    // Serializa a publicação da amostra com stop/start. Se esta
                    // geração foi invalidada durante a coleta, descarta a amostra.
                    let _lifecycle = shared
                        .lifecycle
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    if !generation_is_active(&shared, generation) {
                        break;
                    }

                    let mut history = shared
                        .history
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    history.push_back(sample.clone());
                    while history.len() > HISTORY_LIMIT {
                        history.pop_front();
                    }
                    drop(history);
                    let _ = app_handle.emit("metrics", sample);
                }

                let mut slept = 0u64;
                while slept < interval && generation_is_active(&shared, generation) {
                    let step = 200.min(interval - slept);
                    std::thread::sleep(Duration::from_millis(step));
                    slept += step;
                }
            }
        })
        .map_err(|e| {
            state.running.store(false, Ordering::SeqCst);
            state.generation.fetch_add(1, Ordering::SeqCst);
            format!("falha ao iniciar a coleta: {e}")
        })?;

    Ok(())
}

#[tauri::command]
pub fn stop_monitoring(state: State<'_, MonitorState>) {
    let _lifecycle = state
        .lifecycle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.running.store(false, Ordering::SeqCst);
    // Invalida qualquer trabalhador antigo, inclusive se um novo iniciar logo depois.
    state.generation.fetch_add(1, Ordering::SeqCst);
}

#[tauri::command]
pub fn get_history(state: State<'_, MonitorState>) -> Vec<MetricSample> {
    let history = state
        .history
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    history.iter().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::{generation_is_active, MonitorInner};
    use std::sync::atomic::Ordering;

    #[test]
    fn current_generation_is_active_while_running() {
        let state = MonitorInner::default();
        state.generation.store(7, Ordering::SeqCst);
        state.running.store(true, Ordering::SeqCst);

        assert!(generation_is_active(&state, 7));
    }

    #[test]
    fn stopped_or_replaced_generation_is_inactive() {
        let state = MonitorInner::default();
        state.generation.store(7, Ordering::SeqCst);
        state.running.store(true, Ordering::SeqCst);
        assert!(generation_is_active(&state, 7));

        state.generation.store(8, Ordering::SeqCst);
        assert!(!generation_is_active(&state, 7));
        assert!(generation_is_active(&state, 8));

        state.running.store(false, Ordering::SeqCst);
        assert!(!generation_is_active(&state, 8));
    }
}
