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

/// removes itens com falha/cancelados do histórico
#[tauri::command]
fn limpar_historico_compat(modo: String) {
    system::install::progresso_limpar(&modo);
}

/// pausa um download em andamento (o laço no backend preserva o parcial)
#[tauri::command]
fn pausar_compat(id: String) {
    system::install::pedir_pausar(&id);
}

/// cancela uma operação; só afeta o trabalho pertencente a ela (§8.4)
#[tauri::command]
fn cancelar_compat(id: String) -> Result<String, String> {
    // fase atual decide o que é possível com segurança
    let (estado, fase) = {
        let lista = system::install::progresso_listar();
        lista
            .iter()
            .find(|p| p.id == id)
            .map(|p| (p.estado.clone(), p.fase.clone()))
            .unwrap_or_default()
    };
    if estado.is_empty() {
        return Err("operação não encontrada".to_string());
    }
    // instalação já em transação do gerenciador de pacotes: não há como
    // interromper com segurança; informar em vez de simular (§8.3)
    if estado == "andamento" && fase.as_deref() == Some("instalando") {
        return Err(
            "a instalação já começou e o gerenciador de pacotes do sistema não pode ser interrompido com segurança; aguarde o término"
                .to_string(),
        );
    }
    if estado == "concluido" || estado == "erro" || estado == "cancelado" {
        return Err("esta operação já terminou".to_string());
    }
    system::install::progresso_atualizar(&id, Some("Cancelando..."), None, Some("cancelando"));
    system::install::pedir_cancelar(&id);
    // se a operação estava pausada (sem laço ativo), o cancelamento é
    // imediato e concluído aqui
    if estado == "pausado" {
        system::install::progresso_atualizar(
            &id,
            Some("Cancelado pelo usuário"),
            None,
            Some("cancelado"),
        );
    }
    Ok("cancelamento solicitado".to_string())
}

/// retoma um download pausado via HTTP Range; sem suporte a Range no
/// servidor, recomeça do zero (nunca anexa dados incompatíveis)
#[tauri::command]
async fn retomar_compat(fork: String, versao: String) -> Result<String, String> {
    let id = format!("proton:{fork}:{versao}");
    if !system::install::progresso_iniciar(&id, "Proton", "Retomando download...") {
        return Err("Este instalador já está em andamento".to_string());
    }
    let fork_bg = fork.clone();
    let versao_bg = versao.clone();
    let id_bg = id.clone();
    tauri::async_runtime::spawn(async move {
        let id_cb = id_bg.clone();
        let r = tauri::async_runtime::spawn_blocking(move || {
            let id_prog = id_cb.clone();
            system::install::instalar_proton(
                &fork_bg,
                &versao_bg,
                &id_cb,
                true,
                Some(&mut move |pct, desc| {
                    system::install::progresso_atualizar(&id_prog, Some(desc), pct, None);
                }),
            )
            .map_err(|e| e.to_string())
        })
        .await;
        finalizar_proton(&id_bg, r.map_err(|e| e.to_string()));
    });
    Ok("retomando".to_string())
}

/// estado final de uma execução: mapeia __pausado__ e o cancelamento
fn finalizar_proton(
    id: &str,
    r: Result<Result<String, String>, String>,
) {
    match r {
        Ok(Ok(msg)) => {
            system::install::progresso_atualizar(id, Some(&msg), None, Some("concluido"))
        }
        Ok(Err(e)) => {
            let estado = if e.contains("__pausado__") {
                Some("pausado")
            } else if e.contains("cancelado pelo usuário") {
                Some("cancelado")
            } else {
                Some("erro")
            };
            let desc = e
                .replace("__pausado__", "Pausado — use Retomar para continuar")
                .replace("download cancelado pelo usuário", "Cancelado pelo usuário");
            system::install::progresso_atualizar(id, Some(&desc), None, estado);
        }
        Err(_) => system::install::progresso_atualizar(
            id,
            Some("Processo finalizado inesperadamente"),
            None,
            Some("erro"),
        ),
    }
}

/// ferramentas de compat do Steam (oficiais + customs) com caminhos reais
#[tauri::command]
fn listar_protons_steam() -> Vec<system::steam::ProtonLocal> {
    let mut v = system::steam::protons_oficiais();
    v.extend(system::steam::protons_custom());
    v
}

/// remove uma ferramenta de compat do Steam após validação de identidade
#[tauri::command]
fn remover_ferramenta_compat(caminho: String) -> Result<String, String> {
    system::steam::remover_ferramenta(&caminho)
}

/// remove uma variante de Wine via gerenciador de pacotes (valida o catálogo)
#[tauri::command]
fn remover_wine_compat(pkg: String) -> Result<String, String> {
    system::install::remover_wine(&pkg).map_err(|e| e.to_string())
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
            let id_prog = id_cb.clone();
            system::install::instalar_wine(
                &pkg_bg,
                &id_cb,
                Some(&mut move |pct, desc| {
                    system::install::progresso_atualizar(&id_prog, Some(desc), pct, None);
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
                // cancelamento antes da transação do gerenciador → cancelado
                let estado = if e.contains("cancelada pelo usuário") {
                    Some("cancelado")
                } else {
                    Some("erro")
                };
                let desc = e
                    .replace("instalação cancelada pelo usuário", "Cancelado pelo usuário");
                system::install::progresso_atualizar(&id_bg, Some(&desc), None, estado);
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
            let id_prog = id_cb.clone();
            system::install::instalar_proton(
                &fork_bg,
                &versao_bg,
                &id_cb,
                false,
                Some(&mut move |pct, desc| {
                    system::install::progresso_atualizar(&id_prog, Some(desc), pct, None);
                }),
            )
            .map_err(|e| e.to_string())
        })
        .await;
        finalizar_proton(&id_bg, r.map_err(|e| e.to_string()));
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
            limpar_historico_compat,
            pausar_compat,
            cancelar_compat,
            retomar_compat,
            listar_protons_steam,
            remover_ferramenta_compat,
            remover_wine_compat,
            abrir_app,
            instalar_wine,
            instalar_proton_versao,
            get_analysis
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Gamefic");
}
