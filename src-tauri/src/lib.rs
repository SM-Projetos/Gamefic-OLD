use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

pub mod monitor;
pub mod system;

#[derive(Debug)]
pub enum PrefsError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for PrefsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrefsError::Io(e) => write!(f, "erro ao acessar o arquivo de preferências: {e}"),
            PrefsError::Json(e) => write!(f, "erro de formato no arquivo de preferências: {e}"),
        }
    }
}

impl serde::Serialize for PrefsError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

fn prefs_path(app: &tauri::AppHandle) -> Result<PathBuf, PrefsError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(std::io::Error::other)
        .map_err(PrefsError::Io)?;
    fs::create_dir_all(&dir).map_err(PrefsError::Io)?;
    Ok(dir.join("preferencias.json"))
}

#[tauri::command]
fn get_setting(app: tauri::AppHandle, key: String) -> Result<Option<String>, PrefsError> {
    if key.is_empty() {
        return Ok(None);
    }
    let path = prefs_path(&app)?;
    let raw = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => return Ok(None),
    };
    let map: std::collections::HashMap<String, String> =
        serde_json::from_str(&raw).map_err(PrefsError::Json)?;
    Ok(map.get(&key).cloned())
}

#[tauri::command]
fn set_setting(
    app: tauri::AppHandle,
    key: String,
    value: String,
) -> Result<(), PrefsError> {
    if key.is_empty() {
        return Err(PrefsError::Io(std::io::Error::other("chave vazia")));
    }
    let path = prefs_path(&app)?;
    let mut map: std::collections::HashMap<String, String> = match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map_err(PrefsError::Json)?,
        Err(_) => std::collections::HashMap::new(),
    };
    map.insert(key, value);
    let json = serde_json::to_string_pretty(&map).map_err(PrefsError::Json)?;
    fs::write(path, json).map_err(PrefsError::Io)?;
    Ok(())
}

#[tauri::command]
fn get_system_info() -> system::SystemSnapshot {
    system::snapshot()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(monitor::MonitorInner::default()) as monitor::MonitorState)
        .invoke_handler(tauri::generate_handler![
            get_setting,
            set_setting,
            get_system_info,
            monitor::start_monitoring,
            monitor::stop_monitoring,
            monitor::get_history
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Gamefic");
}
