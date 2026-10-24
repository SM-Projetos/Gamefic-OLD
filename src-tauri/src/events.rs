use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventResult {
    Concluido,
    Falha,
    Cancelado,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventCategory {
    Sistema,
    Deteccao,
    Configuracao,
    Otimizacao,
    Diagnostico,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEvent {
    pub id: u64,
    pub timestamp_ms: u64,
    pub category: EventCategory,
    pub operation: String,
    pub result: EventResult,
    pub message: String,
    pub details: Option<String>,
}

pub const EVENTS_LIMIT: usize = 500;

#[derive(Debug, Default)]
pub struct EventsState {
    next_id: Mutex<u64>,
}

pub type SharedEvents = std::sync::Arc<EventsState>;

fn events_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("diretório de dados indisponível: {e}"))?;
    fs::create_dir_all(&dir).map_err(|e| format!("falha ao criar diretório de dados: {e}"))?;
    Ok(dir.join("eventos.json"))
}

fn load(app: &AppHandle) -> Vec<AppEvent> {
    match events_path(app) {
        Ok(path) => match fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

fn persist(app: &AppHandle, events: &[AppEvent]) -> Result<(), String> {
    let path = events_path(app)?;
    let json =
        serde_json::to_string_pretty(events).map_err(|e| format!("erro de serialização: {e}"))?;
    fs::write(path, json).map_err(|e| format!("falha ao gravar eventos: {e}"))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn record(
    app: &AppHandle,
    state: &SharedEvents,
    category: EventCategory,
    operation: &str,
    result: EventResult,
    message: &str,
    details: Option<String>,
) {
    let mut guard = match state.next_id.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };

    let id = *guard;
    *guard += 1;
    drop(guard);

    let event = AppEvent {
        id,
        timestamp_ms: now_ms(),
        category,
        operation: operation.to_string(),
        result,
        message: message.to_string(),
        details,
    };

    let mut events = load(app);
    events.push(event);
    while events.len() > EVENTS_LIMIT {
        events.remove(0);
    }
    if let Err(e) = persist(app, &events) {
        eprintln!("não foi possível persistir eventos: {e}");
    }
}

#[tauri::command]
pub fn add_event(
    app: AppHandle,
    state: State<'_, SharedEvents>,
    category: EventCategory,
    operation: String,
    result: EventResult,
    message: String,
    details: Option<String>,
) -> Result<(), String> {
    if operation.trim().is_empty() || message.trim().is_empty() {
        return Err("operação e mensagem são obrigatórias".to_string());
    }
    record(
        &app,
        &state.inner(),
        category,
        &operation,
        result,
        &message,
        details,
    );
    Ok(())
}

#[tauri::command]
pub fn get_events(
    app: AppHandle,
    filter_category: Option<EventCategory>,
    filter_result: Option<EventResult>,
) -> Result<Vec<AppEvent>, String> {
    let all = load(&app);
    Ok(all
        .into_iter()
        .filter(|e| {
            let matches_category = filter_category.is_none_or(|c| e.category == c);
            let matches_result = filter_result.is_none_or(|r| e.result == r);
            matches_category && matches_result
        })
        .rev()
        .collect())
}

#[tauri::command]
pub fn clear_events(app: AppHandle) -> Result<usize, String> {
    let count = load(&app).len();
    persist(&app, &[])?;
    Ok(count)
}
