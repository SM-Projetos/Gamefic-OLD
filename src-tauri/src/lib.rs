use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

pub mod events;
pub mod monitor;
pub mod recommendations;
pub mod system;

#[cfg(test)]
mod install_tests;

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
fn refresh_tool_icons() -> u32 {
    system::icons::refresh_cache()
}

#[tauri::command]
async fn get_app_detail(app_id: String) -> Option<system::install::AppDetail> {
    tauri::async_runtime::spawn_blocking(move || system::install::detail(&app_id))
        .await
        .ok()
        .flatten()
}

#[tauri::command]
async fn app_action(app_id: String, acao: String, fonte: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        system::install::acao_app(&app_id, &acao, &fonte)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn listar_compat_forks() -> serde_json::Value {
    tauri::async_runtime::spawn_blocking(|| {
        let dados = serde_json::json!({
            "wines": system::install::forks_wine(),
            "protons": system::install::forks_proton(),
        });
        system::install::salvar_cache_forks(&dados);
        dados
    })
    .await
    .unwrap_or(serde_json::json!({ "wines": [], "protons": [] }))
}

/// lê o último snapshot de forks/versões guardado em disco
#[tauri::command]
fn cache_compat_forks() -> serde_json::Value {
    system::install::ler_cache_forks()
}

#[tauri::command]
async fn listar_proton_versoes(fork: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let versoes = system::install::versoes_proton(&fork).map_err(|e| e.to_string())?;
        system::install::salvar_versoes_cache(&fork, &versoes);
        Ok(versoes)
    })
    .await.unwrap_or_else(|e| Err(e.to_string()))
}

#[tauri::command]
async fn instalar_wine(pkg: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        system::install::instalar_wine(&pkg).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn instalar_proton_versao(fork: String, versao: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        system::install::instalar_proton(&fork, &versao).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_system_info() -> system::SystemSnapshot {
    system::snapshot()
}

#[tauri::command]
fn get_analysis() -> recommendations::Analysis {
    let snapshot = system::snapshot();
    recommendations::analyze(&snapshot)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Arc::new(monitor::MonitorInner::default()) as monitor::MonitorState)
        .manage(Arc::new(events::EventsState::default()) as events::SharedEvents)
        .setup(|app| {
            use events::{EventCategory, EventResult};

            events::record(
                app.handle(),
                &app.state::<events::SharedEvents>().inner().clone(),
                EventCategory::Sistema,
                "inicialização",
                EventResult::Concluido,
                &format!("Gamefic iniciado (v{})", app.package_info().version),
                None,
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_setting,
            set_setting,
            get_system_info,
            monitor::start_monitoring,
            monitor::stop_monitoring,
            monitor::get_history,
            events::add_event,
            events::get_events,
            events::clear_events,
            get_system_info,
            refresh_tool_icons,
            get_app_detail,
            app_action,
            listar_compat_forks,
            cache_compat_forks,
            listar_proton_versoes,
            instalar_wine,
            instalar_proton_versao,
            get_analysis
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Gamefic");
}
