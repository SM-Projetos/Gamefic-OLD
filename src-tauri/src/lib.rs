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
fn set_setting(app: tauri::AppHandle, key: String, value: String) -> Result<(), PrefsError> {
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
        system::install::acao_app(&app_id, &acao, &fonte).map_err(|e| e.to_string())
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
    .await
    .unwrap_or_else(|e| Err(e.to_string()))
}

/// versões oficiais do Wine (GitLab do WineHQ), com cache em disco
#[tauri::command]
async fn listar_wine_versoes() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let versoes = system::install::versoes_wine().map_err(|e| e.to_string())?;
        system::install::salvar_versoes_cache("wine-oficial", &versoes);
        Ok(versoes)
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()))
}

/// estado atual dos processos de instalação (para reabrir a janela)
#[tauri::command]
fn progresso_compat() -> Vec<system::install::ProgressoItem> {
    system::install::progresso_listar()
}

/// remove itens concluídos da área de progresso (ao fechar a janela)
#[tauri::command]
fn limpar_progresso_compat() {
    system::install::progresso_remover_concluidos();
}

fn nome_wine(pkg: &str) -> String {
    match pkg {
        "wine" => "Wine (Stable)".to_string(),
        "wine-staging" => "Wine-Staging".to_string(),
        _ => "Wine CachyOS (otimizado)".to_string(),
    }
}

/// dispara a instalação em segundo plano; a janela pode fechar sem interrompê-la
#[tauri::command]
async fn instalar_wine(pkg: String) -> Result<String, String> {
    let id = format!("wine:{pkg}");
    let nome = nome_wine(&pkg);
    if !system::install::progresso_iniciar(&id, &nome, "Preparando instalação...") {
        return Err("Este instalador já está em andamento".to_string());
    }
    let pkg_bg = pkg.clone();
    let id_bg = id.clone();
    tauri::async_runtime::spawn(async move {
        let id_cb = id_bg.clone();
        let r = tauri::async_runtime::spawn_blocking(move || {
            system::install::instalar_wine(
                &pkg_bg,
                Some(&mut move |pct, desc| {
                    system::install::progresso_atualizar(&id_cb, Some(desc), pct, None);
                }),
            )
            .map_err(|e| e.to_string())
        })
        .await;
        match r {
            Ok(Ok(_)) => system::install::progresso_atualizar(
                &id_bg,
                Some("Concluído"),
                None,
                Some("concluido"),
            ),
            Ok(Err(e)) => {
                system::install::progresso_atualizar(&id_bg, Some(&e), None, Some("erro"))
            }
            Err(_) => system::install::progresso_atualizar(
                &id_bg,
                Some("Processo finalizado inesperadamente"),
                None,
                Some("erro"),
            ),
        }
    });
    Ok("em andamento".to_string())
}

/// dispara a instalação do Proton em segundo plano (independente da janela)
#[tauri::command]
async fn instalar_proton_versao(fork: String, versao: String) -> Result<String, String> {
    let id = format!("proton:{fork}:{versao}");
    let rotulo_ver = if versao.len() > 40 {
        format!("{}…", &versao[..40])
    } else {
        versao.clone()
    };
    let nome = format!("Proton {rotulo_ver}");
    if !system::install::progresso_iniciar(&id, &nome, "Preparando instalação...") {
        return Err("Este instalador já está em andamento".to_string());
    }
    let fork_bg = fork.clone();
    let versao_bg = versao.clone();
    let id_bg = id.clone();
    tauri::async_runtime::spawn(async move {
        let id_cb = id_bg.clone();
        let r = tauri::async_runtime::spawn_blocking(move || {
            system::install::instalar_proton(
                &fork_bg,
                &versao_bg,
                Some(&mut move |pct, desc| {
                    system::install::progresso_atualizar(&id_cb, Some(desc), pct, None);
                }),
            )
            .map_err(|e| e.to_string())
        })
        .await;
        match r {
            Ok(Ok(msg)) => {
                system::install::progresso_atualizar(&id_bg, Some(&msg), None, Some("concluido"))
            }
            Ok(Err(e)) => {
                system::install::progresso_atualizar(&id_bg, Some(&e), None, Some("erro"))
            }
            Err(_) => system::install::progresso_atualizar(
                &id_bg,
                Some("Processo finalizado inesperadamente"),
                None,
                Some("erro"),
            ),
        }
    });
    Ok("em andamento".to_string())
}

#[tauri::command]
fn get_system_info() -> system::SystemSnapshot {
    system::snapshot()
}

#[tauri::command]
fn abrir_app(app_id: String) -> Result<String, String> {
    system::install::abrir_launcher(&app_id)
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
            listar_wine_versoes,
            progresso_compat,
            limpar_progresso_compat,
            abrir_app,
            instalar_wine,
            instalar_proton_versao,
            get_analysis
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Gamefic");
}
