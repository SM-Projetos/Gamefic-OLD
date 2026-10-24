use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::CpuInfo;

pub fn detect() -> Option<CpuInfo> {
    let info = parse_cpuinfo();

    if info.model.is_none() && info.threads_logical.map(|t| t == 0).unwrap_or(true) {
        return None;
    }

    Some(info)
}

fn parse_cpuinfo() -> CpuInfo {
    let text = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

    let mut model: Option<String> = None;
    let mut vendor: Option<String> = None;
    let mut processor_count: u32 = 0;

    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());

        match key {
            "model name" if model.is_none() => model = Some(value.to_string()),
            "vendor_id" if vendor.is_none() => vendor = Some(value.to_string()),
            "processor" if value.chars().all(|c| c.is_ascii_digit()) => processor_count += 1,
            _ => {}
        }
    }

    let current_freqs_mhz = current_freqs();
    let threads_logical = if processor_count > 0 {
        Some(processor_count)
    } else {
        Some(current_freqs_mhz.len() as u32)
    };

    CpuInfo {
        model,
        vendor,
        cores_physical: physical_core_count(),
        threads_logical,
        base_freq_mhz: base_freq(),
        usage_percent: usage_percent(),
        current_freqs_mhz,
    }
}

fn cpu_dir() -> Option<&'static Path> {
    let path = Path::new("/sys/devices/system/cpu");
    if path.is_dir() { Some(path) } else { None }
}

fn current_freqs() -> Vec<u32> {
    let mut freqs = Vec::new();

    if let Some(base) = cpu_dir() {
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !looks_like_cpu(&name) {
                    continue;
                }
                if let Ok(raw) = fs::read_to_string(entry.path().join("cpufreq/scaling_cur_freq")) {
                    let khz: u32 = raw.trim().parse().unwrap_or(0);
                    if khz > 0 {
                        freqs.push(khz / 1000);
                    }
                }
            }
        }
    }

    freqs.sort_unstable();
    freqs
}

fn base_freq() -> Option<u32> {
    let text = fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if key.trim() == "cpu MHz" || key.trim() == "clock" {
            let mhz: u32 = value.trim().parse().unwrap_or(0);
            if mhz > 0 {
                return Some(mhz);
            }
        }
    }

    let max_scaling = max_scaling_freq().map(|v| (v / 1000) as u32);
    if let Some(max) = max_scaling {
        return Some(max / 2);
    }

    None
}

fn max_scaling_freq() -> Option<u64> {
    let base = cpu_dir()?;
    let mut max_khz: u64 = 0;

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !looks_like_cpu(&name) {
                continue;
            }
            if let Ok(raw) = fs::read_to_string(entry.path().join("cpufreq/cpuinfo_max_freq")) {
                let khz: u64 = raw.trim().parse().unwrap_or(0);
                if khz > max_khz {
                    max_khz = khz;
                }
            }
        }
    }

    if max_khz > 0 { Some(max_khz) } else { None }
}

fn physical_core_count() -> Option<u32> {
    let base = cpu_dir()?;

    let mut pairs: HashSet<(String, String)> = HashSet::new();

    if let Ok(entries) = fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !looks_like_cpu(&name) {
                continue;
            }

            let topo = entry.path().join("topology");
            let core = topo.join("core_id");
            let package = topo.join("physical_package_id");

            if let (Ok(core), Ok(package)) = (fs::read_to_string(core), fs::read_to_string(package))
            {
                pairs.insert((package.trim().to_string(), core.trim().to_string()));
            }
        }
    }

    if !pairs.is_empty() {
        return Some(pairs.len() as u32);
    }

    None
}

fn looks_like_cpu(name: &str) -> bool {
    if !name.starts_with("cpu") {
        return false;
    }
    name[3..].chars().all(|c| c.is_ascii_digit())
}

pub fn usage_percent() -> Option<f32> {
    let first = cpu_times()?;
    std::thread::sleep(std::time::Duration::from_millis(150));
    let second = cpu_times()?;

    let diff_total = second.total as f64 - first.total as f64;
    let diff_idle = (second.idle as f64) - (first.idle as f64);

    if diff_total <= 0.0 || diff_idle < 0.0 || diff_idle > diff_total {
        return None;
    }

    Some(((1.0 - diff_idle / diff_total) * 100.0) as f32)
}

struct CpuTimes {
    total: u64,
    idle: u64,
}

fn cpu_times() -> Option<CpuTimes> {
    let content = fs::read_to_string("/proc/stat").ok()?;
    let line = content.lines().next()?;

    let Some((_cpu, rest)) = line.split_once(' ') else {
        return None;
    };

    let values: Vec<u64> = rest
        .split_whitespace()
        .filter_map(|v| v.parse().ok())
        .collect();

    if values.len() < 5 {
        return None;
    }

    Some(CpuTimes {
        total: values.iter().sum(),
        idle: values[3] + values.get(4).copied().unwrap_or(0),
    })
}
