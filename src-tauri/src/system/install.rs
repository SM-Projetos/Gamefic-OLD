use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::appinfo::{AppInfo, REGISTRY};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FonteInfo {
    pub fonte: String,
    pub disponivel: bool,
    pub instalado: bool,
    pub versao: Option<String>,
    pub versao_repo: Option<String>,
    /// motivo da indisponibilidade, quando aplicável
    pub motivo: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDetail {
    pub id: String,
    pub nome: String,
    pub gui: bool,
    pub fornecedor: String,
    pub licenca: String,
    pub desc_curta: String,
    #[serde(rename = "descLonga")]
    pub desc_longa: String,
    pub icone: Option<String>,
    pub fontes: Vec<FonteInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoCompat {
    pub chave: String,
    pub nome: String,
    pub disponivel: bool,
    pub versao_atual: Option<String>,
    pub versao_repo: Option<String>,
}

fn achado(id: &str) -> Option<&'static AppInfo> {
    REGISTRY.iter().find(|a| a.id == id)
}

// ---------- pacotes e flatpak ----------

fn rodar(prog: &str, args: &[&str]) -> Option<String> {
    Command::new(prog)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// extrai a linha "Version: x" de `flatpak remote-info`
pub(crate) fn versao_remote_info(saida: &str) -> Option<String> {
    saida.lines().find_map(|l| {
        let l = l.trim();
        if l.starts_with("Version:") {
            let v = l["Version:".len()..].trim();
            if v.is_empty() { None } else { Some(v.to_string()) }
        } else {
            None
        }
    })
}

/// versão disponível no flathub para um escopo; falha se flatpak ausente,
/// flathub não configurado ou app indisponível (a chamada distingue o motivo)
fn flatpak_versao_flathub(id: &str, escopo: &str) -> Option<String> {
    rodar("flatpak", &["remote-info", escopo, "flathub", id])
        .and_then(|s| versao_remote_info(&s))
}

/// extrai (app-id, versão) ao gerar listas com `--columns=application,version`
pub(crate) fn listar_com_version(saida: &str) -> Vec<(String, String)> {
    saida
        .lines()
        .filter_map(|l| {
            let mut p = l.split('\t');
            let id = p.next()?.trim().to_string();
            let versao = p.next().unwrap_or("").trim().to_string();
            Some((id, versao))
        })
        .collect()
}

fn flatpaks_escopo(escopo: &str) -> Vec<(String, String)> {
    rodar(
        "flatpak",
        &["list", "--app", escopo, "--columns=application,version"],
    )
    .map(|s| listar_com_version(&s))
    .unwrap_or_default()
}

// referências do sistema atual (consultas, sem instalar nada)
fn flatpaks_instalados() -> Vec<String> {
    flatpaks_escopo("--system")
        .into_iter()
        .chain(flatpaks_escopo("--user"))
        .map(|(id, _)| id)
        .collect()
}

pub fn detail(id: &str) -> Option<AppDetail> {
    let info = achado(id)?;
    let tem_flatpak_cmd = rodar("flatpak", &["--version"]).is_some();
    let sistema = flatpaks_escopo("--system");
    let usuario = flatpaks_escopo("--user");

    let mut fontes = Vec::new();
    for pkg in info.nativos {
        let na_repo = super::pkgmgr::na_repo(pkg);
        let local = super::pkgmgr::instalado(pkg);
        let motivo = if local.is_none() && na_repo.is_none() {
            match super::pkgmgr::atual() {
                super::pkgmgr::PkgMgr::Nenhum =>
                    Some("esta distribuição não tem fonte nativa compatível".to_string()),
                _ => Some("pacote não está nos repositórios habilitados".to_string()),
            }
        } else {
            None
        };
        fontes.push(FonteInfo {
            fonte: format!("nativo:{pkg}"),
            disponivel: local.is_some() || na_repo.is_some(),
            instalado: local.is_some(),
            versao: local,
            versao_repo: na_repo,
            motivo,
        });
    }
    if let Some(fid) = info.flatpak {
        if !tem_flatpak_cmd {
            fontes.push(FonteInfo {
                fonte: "flatpak".to_string(),
                disponivel: false,
                instalado: false,
                versao: None,
                versao_repo: None,
                motivo: Some("flatpak não está instalado no sistema".to_string()),
            });
        } else {
            let no_sistema = sistema.iter().find(|(i, _)| i == fid);
            let no_usuario = usuario.iter().find(|(i, _)| i == fid);

            if let Some((_, versao)) = no_sistema {
                fontes.push(FonteInfo {
                    fonte: "flatpak".to_string(),
                    disponivel: true,
                    instalado: true,
                    versao: Some(versao.clone()),
                    versao_repo: flatpak_versao_flathub(fid, "--system")
                        .or_else(|| flatpak_versao_flathub(fid, "--user")),
                    motivo: None,
                });
            } else if let Some(repo) = flatpak_versao_flathub(fid, "--system")
                .or_else(|| flatpak_versao_flathub(fid, "--user"))
            {
                fontes.push(FonteInfo {
                    fonte: "flatpak".to_string(),
                    disponivel: true,
                    instalado: false,
                    versao: None,
                    versao_repo: Some(repo),
                    motivo: None,
                });
            } else {
                fontes.push(FonteInfo {
                    fonte: "flatpak".to_string(),
                    disponivel: false,
                    instalado: false,
                    versao: None,
                    versao_repo: None,
                    motivo: Some("não está disponível no Flathub configurado do sistema".to_string()),
                });
            }

            if let Some((_, versao)) = no_usuario {
                if no_sistema.is_none() {
                    fontes.push(FonteInfo {
                        fonte: "flatpak-user".to_string(),
                        disponivel: true,
                        instalado: true,
                        versao: Some(versao.clone()),
                        versao_repo: flatpak_versao_flathub(fid, "--user"),
                        motivo: None,
                    });
                }
            }
        }
    }

    Some(AppDetail {
        id: info.id.to_string(),
        nome: info.nome.to_string(),
        gui: info.gui,
        fornecedor: info.fornecedor.to_string(),
        licenca: info.licenca.to_string(),
        desc_curta: info.desc_curta.to_string(),
        desc_longa: info.desc_longa.to_string(),
        icone: super::icons::icone_data_uri(id, &[]),
        fontes,
    })
}

// ---------- abrir launcher instalado ----------

/// Executa o launcher já instalado — na fonte em que ele foi detectado.
/// Flatpak: `flatpak run <id>`; nativo: o binário do pacote.
pub fn abrir_launcher(id: &str) -> Result<String, String> {
    let info = achado(id).ok_or_else(|| "aplicativo desconhecido".to_string())?;
    if !info.gui {
        return Err(format!(
            "{} é uma ferramenta de linha de comando e não tem interface gráfica para iniciar",
            info.nome
        ));
    }
    let instalados = flatpaks_instalados();

    if let Some(fid) = info.flatpak {
        if instalados.iter().any(|l| l == fid) {
            Command::new("flatpak")
                .args(["run", fid])
                .spawn()
                .map_err(|e| format!("falha ao iniciar: {e}"))?;
            return Ok("Aplicativo iniciado".to_string());
        }
    }
    for pkg in info.nativos {
        if super::pkgmgr::instalado(pkg).is_some() {
            Command::new(pkg)
                .spawn()
                .map_err(|e| format!("falha ao iniciar: {e}"))?;
            return Ok("Aplicativo iniciado".to_string());
        }
    }
    Err(format!("{} não está instalado", info.nome))
}

// ---------- ações (install/remove/update) ----------

#[derive(Debug, Clone, Serialize)]
pub enum InstalarError {
    Fonte,
    Falha(String),
    NaoDisponivel,
    Execucao(String),
    /// conflito real entre pacotes (p.ex. variantes de Wine que se contradizem)
    Conflito(String),
}

impl std::fmt::Display for InstalarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InstalarError::Fonte => write!(f, "fonte desconhecida"),
            InstalarError::Falha(m) => write!(f, "{m}"),
            InstalarError::NaoDisponivel => write!(f, "fonte não está disponível"),
            InstalarError::Execucao(m) => write!(f, "falha na execução: {m}"),
            InstalarError::Conflito(m) => write!(f, "conflito de pacotes: {m}"),
        }
    }
}

pub fn acao_app(id: &str, acao: &str, fonte: &str) -> Result<String, InstalarError> {
    let info = achado(id).ok_or(InstalarError::Fonte)?;

    let pkgmgr_op = |op: super::pkgmgr::PkgOp, pkg: &str| -> Result<String, InstalarError> {
        // revalida a condição antes de executar: vigiar compatibilidade,
        // disponibilidade e estado real (ainda instalado / presente na repo)
        let resultado = match op {
            super::pkgmgr::PkgOp::Instalar => super::pkgmgr::instalar(pkg),
            super::pkgmgr::PkgOp::Remover => super::pkgmgr::remover(pkg),
        };
        resultado
            .map(|_| match op {
                super::pkgmgr::PkgOp::Instalar => "Instalação concluída".to_string(),
                super::pkgmgr::PkgOp::Remover => "Remoção concluída".to_string(),
            })
            .map_err(InstalarError::Falha)
    };

    match fonte {
        f if f.starts_with("nativo:") => {
            // valida que o pacote é do catálogo, nunca do frontend
            let pkg = f.strip_prefix("nativo:").ok_or(InstalarError::Fonte)?;
            if !info.nativos.contains(&pkg) {
                return Err(InstalarError::Fonte);
            }
            let op = match acao {
                "instalar" => super::pkgmgr::PkgOp::Instalar,
                "atualizar" => super::pkgmgr::PkgOp::Instalar,
                "remover" => {
                    if super::pkgmgr::instalado(pkg).is_none() {
                        return Err(InstalarError::Falha(
                            "o pacote nativo não está mais instalado".into(),
                        ));
                    }
                    super::pkgmgr::PkgOp::Remover
                }
                _ => return Err(InstalarError::Fonte),
            };
            pkgmgr_op(op, pkg)
        }
        fonte_escopo @ ("flatpak" | "flatpak-user") => {
            let Some(fid) = info.flatpak else {
                return Err(InstalarError::Fonte);
            };
            // revalida: evitar remover/atualizar uma instalação de outro escopo
            let escopo = if fonte_escopo == "flatpak-user" { "--user" } else { "--system" };
            let instalado_no_escopo = rodar(
                "flatpak",
                &["list", "--app", escopo, "--columns=application"],
            )
            .map(|s| listar_com_version(&s).iter().any(|(i, _)| i == fid))
            .unwrap_or(false);
            if fonte_escopo == "flatpak-user" && !instalado_no_escopo && acao == "remover" {
                return Err(InstalarError::Falha(
                    "não há instalação do seu usuário para remover neste escopo".into(),
                ));
            }
            let out = match acao {
                "instalar" | "atualizar" => Command::new("flatpak")
                    .args([
                        "install",
                        escopo,
                        "--noninteractive",
                        "-y",
                        "flathub",
                        fid,
                    ])
                    .output()
                    .map_err(|e| InstalarError::Execucao(e.to_string()))?,
                "remover" => Command::new("flatpak")
                    .args(["uninstall", escopo, "-y", "--noninteractive", fid])
                    .output()
                    .map_err(|e| InstalarError::Execucao(e.to_string()))?,
                _ => return Err(InstalarError::Fonte),
            };
            if out.status.success() {
                Ok("Operação concluída".to_string())
            } else {
                Err(InstalarError::Falha(
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ))
            }
        }
        _ => Err(InstalarError::Fonte),
    }
}

// ---------- progresso dos processos de instalação ----------
// O ciclo de vida das instalações é independente da janela: o estado vive
// aqui, em memória, e a UI só consulta/atualiza por meio destes helpers.

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressoItem {
    pub id: String,
    pub nome: String,
    pub desc: String,
    pub pct: Option<u32>,
    /// "aguardando" | "baixando" | "pausado" | "instalando" | "concluido" | "erro" | "cancelado"
    pub estado: String,
    /// subfase detalhada para os controles da interface
    pub fase: Option<String>,
    pub kb_feitos: Option<u64>,
    pub kb_total: Option<u64>,
    /// bytes/s medidos no intervalo de amostragem
    pub velocidade: Option<u64>,
}

static PROGRESSO: std::sync::Mutex<Vec<ProgressoItem>> = std::sync::Mutex::new(Vec::new());

pub type PassoProg<'a> = &'a mut (dyn FnMut(Option<u32>, &str) + Send + 'a);

/// inicia um item de progresso; devolve false se já houver um em andamento
pub fn progresso_iniciar(id: &str, nome: &str, desc: &str) -> bool {
    let mut l = PROGRESSO.lock().expect("mutex do progresso");
    if l.iter().any(|p| p.id == id && p.estado == "andamento") {
        return false;
    }
    l.retain(|p| p.id != id);
    l.push(ProgressoItem {
        id: id.to_string(),
        nome: nome.to_string(),
        desc: desc.to_string(),
        pct: None,
        estado: "andamento".to_string(),
        fase: Some("aguardando".to_string()),
        kb_feitos: None,
        kb_total: None,
        velocidade: None,
    });
    true
}

/// atualiza a subfase do item (usada pelo motor de download)
pub fn progresso_fase(id: &str, fase: &str) {
    let mut l = PROGRESSO.lock().expect("mutex do progresso");
    if let Some(p) = l.iter_mut().find(|p| p.id == id) {
        p.fase = Some(fase.to_string());
    }
}

pub fn progresso_atualizar(id: &str, desc: Option<&str>, pct: Option<u32>, estado: Option<&str>) {
    let mut l = PROGRESSO.lock().expect("mutex do progresso");
    if let Some(p) = l.iter_mut().find(|p| p.id == id) {
        if let Some(d) = desc {
            p.desc = d.to_string();
        }
        if let Some(n) = pct {
            p.pct = Some(n);
        }
        if let Some(e) = estado {
            p.estado = e.to_string();
        }
        if estado == Some("erro") || estado == Some("concluido") {
            p.pct = if estado == Some("concluido") {
                Some(100)
            } else {
                None
            };
        }
    }
}

/// limpa o histórico: "erro" remove apenas itens finalizados com falha
/// (erro ou cancelado); "finalizados" remove tudo que não esteja em
/// andamento/pausado — operações ativas nunca são afetadas
pub fn progresso_limpar(modo: &str) {
    let mut l = PROGRESSO.lock().expect("mutex do progresso");
    match modo {
        "erro" => l.retain(|p| p.estado != "erro" && p.estado != "cancelado"),
        "finalizados" => l.retain(|p| {
            matches!(p.estado.as_str(), "andamento" | "pausado" | "cancelando")
        }),
        _ => {}
    }
}

pub fn progresso_listar() -> Vec<ProgressoItem> {
    let l = PROGRESSO.lock().expect("mutex do progresso");
    l.clone()
}

// ---------- Wine ----------

/// remove uma variante de Wine pelo gerenciador de pacotes; o pacote precisa
/// pertencer ao catálogo da própria ferramenta e estar efetivamente instalado
pub fn remover_wine(pkg: &str) -> Result<String, InstalarError> {
    if !["wine", "wine-staging", "wine-cachyos"].contains(&pkg) {
        return Err(InstalarError::Fonte);
    }
    let Some(versao) = super::pkgmgr::instalado(pkg) else {
        return Err(InstalarError::Execucao(format!(
            "o pacote {pkg} não está instalado por este gerenciador"
        )));
    };
    super::pkgmgr::remover(pkg)
        .map(|_| format!("Wine {versao} removido"))
        .map_err(InstalarError::Falha)
}

pub fn forks_wine() -> Vec<InfoCompat> {
    let mut lista = Vec::new();
    for pkg in ["wine", "wine-staging", "wine-cachyos"] {
        let repo = super::pkgmgr::na_repo(pkg);
        lista.push(InfoCompat {
            chave: pkg.to_string(),
            nome: match pkg {
                "wine" => "Wine (Stable)".to_string(),
                "wine-staging" => "Wine-Staging".to_string(),
                _ => "Wine CachyOS (otimizado)".to_string(),
            },
            disponivel: repo.is_some(),
            versao_atual: super::pkgmgr::instalado(pkg),
            versao_repo: repo,
        });
    }
    lista
}

/// lista de versões oficiais do Wine, direto da API do projeto (GitLab do WineHQ);
/// tags como "wine-11.20" viram "11.20"
pub fn versoes_wine() -> Result<Vec<String>, InstalarError> {
    let texto = http_texto(
        "https://gitlab.winehq.org/api/v4/projects/wine%2Fwine/releases?per_page=15",
        4_000_000,
    )?;
    let json: serde_json::Value =
        serde_json::from_str(&texto).map_err(|e| InstalarError::Execucao(e.to_string()))?;
    let mut v: Vec<String> = json
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|r| r["tag_name"].as_str().and_then(|s| s.strip_prefix("wine-")))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    v.dedup();
    if v.is_empty() {
        Err(InstalarError::Execucao("sem releases do Wine".to_string()))
    } else {
        Ok(v)
    }
}

pub fn instalar_wine(pkg: &str, mut prog: Option<PassoProg>) -> Result<String, InstalarError> {
    if !["wine", "wine-staging", "wine-cachyos"].contains(&pkg) {
        return Err(InstalarError::Fonte);
    }

    // variante incompatível com a distribuição a partir do gerenciador real
    match super::pkgmgr::atual() {
        super::pkgmgr::PkgMgr::Nenhum => {
            return Err(InstalarError::Execucao(
                "a fonte nativa não é compatível com esta distribuição".to_string(),
            ));
        }
        _ => {
            if super::pkgmgr::na_repo(pkg).is_none() {
                return Err(InstalarError::NaoDisponivel);
            }
        }
    }

    // variantes de Wine entram em conflito entre si (todos fornecem a
    // mesma camada): diagnosticar antes da transação do gerenciador
    let outras: &[(&str, &str)] = &[
        ("wine", "Wine Stable"),
        ("wine-staging", "Wine-Staging"),
        ("wine-cachyos", "Wine CachyOS (otimizado)"),
    ];
    let conflitos: Vec<&str> = outras
        .iter()
        .filter(|(p, _)| *p != pkg)
        .filter_map(|(p, nome)| super::pkgmgr::instalado(p).map(|_| *nome))
        .collect();
    if !conflitos.is_empty() {
        return Err(InstalarError::Conflito(format!(
            "instale '{}' desinstalando antes {}: todas as variantes de Wine fornecem a mesma camada de compatibilidade e o gerenciador de pacotes não resolve a substituição com --noconfirm",
            pkg,
            conflitos.join(" e ")
        )));
    }

    if let Some(cb) = &mut prog {
        cb(None, "Consultando os repositórios...");
    }
    super::pkgmgr::instalar(pkg)
        .map(|_| "Wine instalado".to_string())
        .map_err(InstalarError::Falha)
}

// ---------- Proton ----------

pub(crate) const PROTON_MAX_MB: u64 = 900;

fn diretorio_proton() -> Option<PathBuf> {
    // destino correto conforme a instalação efetiva do Steam detectada
    // (nativa, Flatpak ou Snap); mantém fallback nos caminhos clássicos
    if let Some(d) = super::steam::destino_compat() {
        return Some(d);
    }
    let home = std::env::var("HOME").ok()?;
    let base = [
        PathBuf::from(&home).join(".local/share/Steam/compatibilitytools.d"),
        PathBuf::from(&home).join(".steam/steam/compatibilitytools.d"),
    ];
    for b in base {
        if fs::create_dir_all(&b).is_ok() {
            return Some(b);
        }
    }
    None
}

fn http_texto(url: &str, max: usize) -> Result<String, InstalarError> {
    use std::io::Read;
    let resp = ureq::get(url)
        .set("User-Agent", "gamefic")
        .timeout(std::time::Duration::from_secs(25))
        .call()
        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
    let mut texto = String::new();
    resp.into_reader()
        .take(max as u64)
        .read_to_string(&mut texto)
        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
    Ok(texto)
}

fn releases_tag(repositorio: &str) -> Result<Vec<String>, InstalarError> {
    let url = format!("https://api.github.com/repos/{repositorio}/releases?per_page=15");
    match http_texto(&url, 8_000_000) {
        Ok(texto) => {
            let json: Vec<serde_json::Value> =
                serde_json::from_str(&texto).map_err(|e| InstalarError::Execucao(e.to_string()))?;
            let mut tags: Vec<String> = json
                .iter()
                .filter_map(|r| r["tag_name"].as_str().map(|s| s.to_string()))
                .collect();
            if !tags.is_empty() {
                return Ok(tags);
            }
            releases_atom(repositorio)
        }
        Err(_) => releases_atom(repositorio),
    }
}

fn releases_atom(repositorio: &str) -> Result<Vec<String>, InstalarError> {
    let texto = http_texto(
        &format!("https://github.com/{repositorio}/releases.atom"),
        2_000_000,
    )?;
    let mut tags: Vec<String> = Vec::new();
    for bloco in texto.split("<entry>") {
        let Some(t) = bloco.split("</title>").next() else {
            continue;
        };
        let Some(t) = t.split("<title>").nth(1) else {
            continue;
        };
        let mut tag = t.trim().to_string();
        // ruído do feed: entrada sem release próprio ("Release notes from …")
        if tag.starts_with("Release notes from") {
            continue;
        }
        if tag.ends_with("from proton-cachyos") {
            continue;
        }
        // título do feed: "GE-ProtonXX-YY Released", "Version X" ou "cachyos-X-native"
        if tag.ends_with(" Released") {
            tag = tag.trim_end_matches(" Released").to_string();
        }
        if let Some(rest) = tag.strip_prefix("Version ") {
            tag = rest.to_string();
        }
        if tag.ends_with("-native") || tag.is_empty() {
            continue;
        }
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    if tags.is_empty() {
        Err(InstalarError::Execucao("sem releases no feed".to_string()))
    } else {
        Ok(tags.into_iter().take(15).collect())
    }
}

#[derive(Clone, Copy, PartialEq)]
enum FonteProton {
    /// release/feed no GitHub (atom + expanded_assets)
    Github,
    /// API JSON do Forgejo/Gitea (/api/v1/.../releases)
    Forgejo,
    /// distribuído apenas pelo Steam (sem endereço oficial de download)
    Steam,
}

struct ForkProton {
    chave: &'static str,
    nome: &'static str,
    repositorio: &'static str,
    pacote: Option<&'static str>,
    fonte: FonteProton,
    /// trecho usado para detectar instalação local (lowercase)
    detect: &'static str,
}

const FORKS_PROTON: &[ForkProton] = &[
    ForkProton {
        chave: "ge",
        nome: "GE-Proton",
        repositorio: "GloriousEggroll/proton-ge-custom",
        pacote: None,
        fonte: FonteProton::Github,
        detect: "ge-proton",
    },
    ForkProton {
        chave: "cachyos",
        nome: "Proton-cachyos",
        repositorio: "CachyOS/proton-cachyos",
        pacote: Some("proton-cachyos"),
        fonte: FonteProton::Github,
        detect: "cachyos",
    },
    ForkProton {
        chave: "dw",
        nome: "DW-Proton",
        repositorio: "https://dawn.wine/api/v1/repos/dawn-winery/dwproton/releases?limit=10",
        pacote: None,
        fonte: FonteProton::Forgejo,
        detect: "dw-proton",
    },
    ForkProton {
        chave: "rtsp",
        nome: "Proton-GE-RTSP",
        repositorio: "SpookySkeletons/proton-rtsp",
        pacote: None,
        fonte: FonteProton::Github,
        detect: "rtsp",
    },
    ForkProton {
        chave: "em",
        nome: "Proton-EM",
        repositorio: "BananaWorks07/Proton",
        pacote: None,
        fonte: FonteProton::Github,
        detect: "proton-em",
    },
    ForkProton {
        chave: "wineland",
        nome: "Proton-Wineland",
        // renomeado de nanomatters/proton-cachyos; releases "Proton Wineland"
        repositorio: "nanomatters/proton-salkim",
        pacote: None,
        fonte: FonteProton::Github,
        detect: "wineland",
    },
    ForkProton {
        chave: "winege",
        nome: "Wine-GE (histórico)",
        // projeto arquivado (07/2025); sucessor: UMU. Versões históricas
        // ainda instaláveis em compatibilitytools.d (ge-proton8/wine-lutris)
        repositorio: "GloriousEggroll/wine-ge-custom",
        pacote: None,
        fonte: FonteProton::Github,
        detect: "wine-lutris",
    },
    ForkProton {
        chave: "luxtorpeda",
        nome: "Luxtorpeda",
        repositorio: "https://codeberg.org/api/v1/repos/luxtorpeda/luxtorpeda/releases?limit=10",
        pacote: None,
        fonte: FonteProton::Forgejo,
        detect: "luxtorpeda",
    },
    ForkProton {
        chave: "steam",
        nome: "Proton (Steam)",
        repositorio: "",
        pacote: None,
        fonte: FonteProton::Steam,
        detect: "proton",
    },
];

fn fork_proton(chave: &str) -> Result<&'static ForkProton, InstalarError> {
    FORKS_PROTON
        .iter()
        .find(|f| f.chave == chave)
        .ok_or(InstalarError::Fonte)
}

/// releases do Forgejo: lista (tag, asset tar.xz/tar.gz)
fn releases_forgejo(url: &str) -> Result<Vec<(String, Option<String>)>, InstalarError> {
    let texto = http_texto(url, 4_000_000)?;
    let json: serde_json::Value =
        serde_json::from_str(&texto).map_err(|e| InstalarError::Execucao(e.to_string()))?;
    let Some(lista) = json.as_array() else {
        return Err(InstalarError::Execucao("resposta inesperada".to_string()));
    };
    let mut saida = Vec::new();
    for r in lista {
        let tag = r["tag_name"].as_str().unwrap_or_default().to_string();
        if tag.is_empty() {
            continue;
        }
        let asset = r["assets"]
            .as_array()
            .and_then(|a| {
                let tarballs: Vec<&str> = a
                    .iter()
                    .filter_map(|x| x["browser_download_url"].as_str())
                    .filter(|u| {
                        let l = u.to_lowercase();
                        l.ends_with(".tar.xz") || l.ends_with(".tar.gz") || l.ends_with(".tar.zst")
                    })
                    .collect();
                // prioriza um tarball com o runtime x86_64 explícito no nome
                tarballs
                    .iter()
                    .find(|u| {
                        let l = u.to_lowercase();
                        l.contains("x86_64") || l.contains("x64") || l.contains("amd64")
                    })
                    .or_else(|| tarballs.first())
                    .copied()
            })
            .map(|s| s.to_string());
        saida.push((tag, asset));
    }
    if saida.is_empty() {
        Err(InstalarError::Execucao("sem releases no feed".to_string()))
    } else {
        Ok(saida)
    }
}

/// releases por fork: lista (tag, asset opcional)
fn releases_fork(f: &ForkProton) -> Result<Vec<(String, Option<String>)>, InstalarError> {
    match f.fonte {
        FonteProton::Forgejo => releases_forgejo(f.repositorio),
        FonteProton::Github => {
            let tags = releases_tag(f.repositorio)?;
            Ok(tags.into_iter().map(|t| (t, None)).collect())
        }
        FonteProton::Steam => Err(InstalarError::Fonte),
    }
}

pub fn forks_proton() -> Vec<InfoCompat> {
    let mut lista = Vec::new();
    let locais: Vec<String> = super::tools::detectar_protons()
        .into_iter()
        .map(|(n, _)| n.to_lowercase())
        .collect();

    for f in FORKS_PROTON {
        let mut latest = if f.fonte == FonteProton::Steam {
            None
        } else {
            releases_fork(f)
                .map(|v| v.into_iter().next().map(|(t, _)| t))
                .unwrap_or(None)
        };
        let local = locais.iter().any(|n| n.contains(f.detect));
        if String::is_empty(&latest.clone().unwrap_or_default()) {
            latest = None;
        }
        lista.push(InfoCompat {
            chave: f.chave.to_string(),
            nome: f.nome.to_string(),
            disponivel: latest.is_some() || f.pacote.is_some(),
            versao_atual: local.then(|| "instalado".to_string()),
            versao_repo: latest,
        });
    }
    lista
}

/// ---- cache de forks/versões em disco ----

fn arquivo_cache() -> Option<std::path::PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{h}/.local/share"))
        })?;
    let dir = std::path::PathBuf::from(base)
        .join("com.sergio.gamefic")
        .join("cache");
    fs::create_dir_all(&dir).ok()?;
    Some(dir.join("forks.json"))
}

pub fn salvar_cache_forks(dados: &serde_json::Value) {
    if let Some(arq) = arquivo_cache() {
        if let Ok(texto) = serde_json::to_string(dados) {
            let _ = fs::write(&arq, texto);
        }
    }
}

pub fn ler_cache_forks() -> serde_json::Value {
    let Some(arq) = arquivo_cache() else {
        return serde_json::json!({ "wines": [], "protons": [] });
    };
    fs::read_to_string(&arq)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(serde_json::json!({ "wines": [], "protons": [] }))
}

pub fn salvar_versoes_cache(fork: &str, versoes: &[String]) {
    let Some(arq) = arquivo_cache() else {
        return;
    };
    let mut dados: serde_json::Value = fs::read_to_string(&arq)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(serde_json::json!({ "wines": [], "protons": [] }));
    dados["versoes"][fork] = serde_json::json!(versoes);
    if let Ok(texto) = serde_json::to_string(&dados) {
        let _ = fs::write(&arq, texto);
    }
}

pub fn versoes_proton(fork: &str) -> Result<Vec<String>, InstalarError> {
    let f = fork_proton(fork)?;
    match f.fonte {
        FonteProton::Steam => Ok(Vec::new()),
        _ => Ok(releases_fork(f)?.into_iter().map(|(t, _)| t).collect()),
    }
}

fn asset_url_github(repositorio: &str, tag: &str) -> Result<String, InstalarError> {
    let page = http_texto(
        &format!("https://github.com/{repositorio}/releases/expanded_assets/{tag}"),
        2_000_000,
    )?;
    // usa apenas assets que o release realmente publica; sem inventar nomes
    let mut tarballs: Vec<String> = Vec::new();
    for bloco in page.split("href=\"") {
        let Some(href) = bloco.split('"').next() else {
            continue;
        };
        let minic = href.to_lowercase();
        if minic.ends_with(".tar.gz") || minic.ends_with(".tar.xz") || minic.ends_with(".tar.zst") {
            let url = if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://github.com{href}")
            };
            tarballs.push(url);
        }
    }
    if tarballs.is_empty() {
        return Err(InstalarError::Execucao(format!(
            "o release {tag} de {repositorio} não publica arquivos de distribuição (tarball); versão não instalável por este método"
        )));
    }
    let escolhido = tarballs
        .iter()
        .find(|u| u.to_lowercase().contains("proton"))
        .cloned()
        .unwrap_or_else(|| tarballs[0].clone());
    Ok(escolhido)
}

// ---------- controles e estado dos downloads por operação ----------

/// 0 = normal | 1 = pediu pausar | 2 = pediu cancelar
#[derive(Default, Clone)]
pub struct EstadoDl {
    pub feitos: u64,
    pub total: Option<u64>,
    pub url: String,
    pub tmp: std::path::PathBuf,
    pub tag: String,
    pub extensao: String,
}

fn controles_dl() -> &'static std::sync::Mutex<std::collections::HashMap<String, Arc<std::sync::atomic::AtomicU8>>> {
    static M: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, Arc<std::sync::atomic::AtomicU8>>>,
    > = std::sync::OnceLock::new();
    M.get_or_init(Default::default)
}

fn estados_dl() -> &'static std::sync::Mutex<std::collections::HashMap<String, EstadoDl>> {
    static M: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, EstadoDl>>,
    > = std::sync::OnceLock::new();
    M.get_or_init(Default::default)
}

use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

fn controle_de(id: &str) -> Arc<AtomicU8> {
    controles_dl()
        .lock()
        .expect("mutex dos controles")
        .entry(id.to_string())
        .or_insert_with(|| Arc::new(AtomicU8::new(0)))
        .clone()
}

/// estado salvo de um download (feitos, url, arquivo parcial)
pub fn estado_dl(id: &str) -> Option<EstadoDl> {
    estados_dl().lock().expect("mutex dos estados").get(id).cloned()
}

fn descartar_estado_dl(id: &str) {
    estados_dl().lock().expect("mutex dos estados").remove(id);
}

/// pausa a operação: o laço de download percebe e preserva o arquivo parcial
pub fn pedir_pausar(id: &str) {
    controle_de(id).store(1, Ordering::SeqCst);
}

/// cancela a operação: o laço (se ativo) apaga o parcial e volta erro;
/// se a operação está pausada, a limpeza acontece imediatamente aqui
pub fn pedir_cancelar(id: &str) {
    controle_de(id).store(2, Ordering::SeqCst);
    let parcial = estados_dl()
        .lock()
        .expect("mutex dos estados")
        .get(id)
        .map(|e| e.tmp.clone());
    if let Some(tmp) = parcial {
        let _ = fs::remove_file(&tmp);
    }
    descartar_estado_dl(id);
}

/// atualiza o item de progresso com o andamento real do download
fn progresso_dl(id: &str, feitos: u64, total: Option<u64>, vel: u64) {
    let mut l = PROGRESSO.lock().expect("mutex do progresso");
    if let Some(p) = l.iter_mut().find(|p| p.id == id) {
        p.kb_feitos = Some(feitos / 1024);
        p.kb_total = total.map(|t| t / 1024);
        p.velocidade = if vel > 0 { Some(vel) } else { None };
        match total {
            Some(t) if t > 0 => {
                let pct = ((feitos * 100) / t).min(100) as u32;
                p.pct = Some(pct);
                p.desc = format!(
                    "Baixando ({:.1}/{:.1} MB, {:.1} MB/s)...",
                    feitos as f64 / 1e6,
                    t as f64 / 1e6,
                    vel as f64 / 1e6
                );
            }
            _ => {
                p.desc = format!("Baixando ({:.1} MB)...", feitos as f64 / 1e6);
            }
        }
    }
}

pub fn instalar_proton(
    fork: &str,
    versao: &str,
    id: &str,
    retomar: bool,
    mut prog: Option<PassoProg>,
) -> Result<String, InstalarError> {
    use std::io::{Read, Write};
    let f = fork_proton(fork)?;

    if f.fonte == FonteProton::Steam {
        return Err(InstalarError::Execucao(
            "o Proton padrão é gerenciado pelo próprio Steam".to_string(),
        ));
    }

    if let Some(p) = f.pacote {
        if versao == "pacman" {
            if let Some(cb) = &mut prog {
                cb(None, "Instalando pacote (autorize no pkexec)...");
            }
            let out = Command::new("pkexec")
                .args(["pacman", "-S", "--noconfirm", p])
                .output()
                .map_err(|e| InstalarError::Execucao(e.to_string()))?;
            if out.status.success() {
                return Ok("Proton (pacote) instalado".to_string());
            }
            return Err(InstalarError::Falha(
                String::from_utf8_lossy(&out.stderr).to_string(),
            ));
        }
    }

    let controle = controle_de(id);
    controle.store(0, Ordering::SeqCst);
    progresso_fase(id, "aguardando");

    // 1) resolver versão/endereço — numa retomada, usa o estado salvo
    let salvo = if retomar { estado_dl(id) } else { None };
    let (tag, url_download, _extensao, tmp) = match &salvo {
        Some(s) => (
            s.tag.clone(),
            s.url.clone(),
            s.extensao.clone(),
            s.tmp.clone(),
        ),
        None => {
            if let Some(cb) = &mut prog {
                cb(None, "Resolvendo versão e endereço de download...");
            }

            // descobre o nome interno do fork p/ resolver a versão
            let tag = if versao == "latest" {
                releases_fork(f)?
                    .into_iter()
                    .next()
                    .and_then(|(t, _)| (!t.is_empty()).then_some(t))
                    .ok_or(InstalarError::Execucao("sem releases".to_string()))?
            } else {
                versao.to_string()
            };
            let url_download = match f.fonte {
                FonteProton::Forgejo => releases_fork(f)?
                    .into_iter()
                    .find(|(t, _)| *t == tag)
                    .and_then(|(_, a)| a)
                    .ok_or(InstalarError::Execucao(
                        "sem binário no release".to_string(),
                    ))?,
                _ => asset_url_github(f.repositorio, &tag)?,
            };
            let extensao = if url_download.to_lowercase().ends_with(".tar.xz") {
                "tar.xz"
            } else if url_download.to_lowercase().ends_with(".tar.zst") {
                "tar.zst"
            } else {
                "tar.gz"
            };

            // download para /tmp, com relato de progresso
            let tmp = std::env::temp_dir().join(format!("gamefic-{}.{}", fork, extensao));
            let _ = fs::remove_file(&tmp);
            estados_dl()
                .lock()
                .expect("mutex dos estados")
                .insert(
                    id.to_string(),
                    EstadoDl {
                        feitos: 0,
                        total: None,
                        url: url_download.clone(),
                        tmp: tmp.clone(),
                        tag: tag.clone(),
                        extensao: extensao.to_string(),
                    },
                );
            progresso_fase(id, "baixando");
            (tag, url_download, extensao.to_string(), tmp)
        }
    };

    // 2) abre a requisição (com Range na retomada); se o servidor não
    // suportar Range, recomeça o arquivo do zero — nunca anexa dados
    // incompatíveis ao parcial
    let parcial = tmp.metadata().map(|m| m.len()).unwrap_or(0);
    let retomavel = salvo.is_some() && parcial > 0;
    let resultado_req = if retomavel {
        ureq::get(&url_download)
            .set("User-Agent", "gamefic")
            .set("Range", &format!("bytes={parcial}-"))
            .timeout(std::time::Duration::from_secs(900))
            .call()
    } else {
        ureq::get(&url_download)
            .set("User-Agent", "gamefic")
            .timeout(std::time::Duration::from_secs(900))
            .call()
    };
    let resp = match resultado_req {
        Ok(r) => r,
        Err(e) => {
            if matches!(&e, ureq::Error::Status(404, _)) {
                let _ = fs::remove_file(&tmp);
                return Err(InstalarError::Execucao(format!(
                    "arquivo de distribuição não encontrado no release {tag}: a URL do projeto mudou"
                )));
            } else if let ureq::Error::Status(c, _) = &e {
                if *c >= 400 {
                    return Err(InstalarError::Execucao(format!(
                        "servidor respondeu {c} em {url_download}"
                    )));
                }
                return Err(InstalarError::Execucao(e.to_string()));
            }
            return Err(InstalarError::Execucao(e.to_string()));
        }
    };
    let mut feitos: u64 = 0;
    let mut arquivo: fs::File;
    if retomavel {
        if resp.status() == 206 {
            if let Some(cb) = &mut prog {
                cb(None, &format!(
                    "Retomando desde {:.1} MB...",
                    parcial as f64 / 1e6
                ));
            }
            feitos = parcial;
            arquivo = fs::OpenOptions::new()
                .append(true)
                .open(&tmp)
                .map_err(|e| InstalarError::Execucao(e.to_string()))?;
        } else if resp.status() == 200 {
            // servidor ignora Range: recomeço limpo do zero
            if let Some(cb) = &mut prog {
                cb(None, "O servidor não suporta retomada; recomeçando...");
            }
            arquivo = fs::File::create(&tmp)
                .map_err(|e| InstalarError::Execucao(e.to_string()))?;
        } else {
            let _ = fs::remove_file(&tmp);
            descartar_estado_dl(id);
            return Err(InstalarError::Execucao(format!(
                "servidor respondeu {} ao pedir retomada",
                resp.status()
            )));
        }
    } else {
        arquivo = fs::File::create(&tmp)
            .map_err(|e| InstalarError::Execucao(e.to_string()))?;
    }
    let total = resp
        .header("content-length")
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v + feitos);
    let mut reader = resp.into_reader();

    // grava o total no estado salvo (para exibir tamanho real)
    if let Some(t) = total {
        estados_dl()
            .lock()
            .expect("mutex dos estados")
            .entry(id.to_string())
            .and_modify(|e| e.total = Some(t));
    }

    let limite = PROTON_MAX_MB * 1024 * 1024;
    let mut buf = [0u8; 64 * 1024];
    let mut ultimo_tick = std::time::Instant::now();
    let mut feitos_amostra = feitos;
    loop {
        let ctl = controle.load(Ordering::SeqCst);
        if ctl == 2 {
            let _ = arquivo.flush();
            drop(arquivo);
            let _ = fs::remove_file(&tmp);
            descartar_estado_dl(id);
            return Err(InstalarError::Falha(
                "download cancelado pelo usuário".to_string(),
            ));
        }
        if ctl == 1 {
            // pausa: preserva o arquivo parcial e o estado p/ retomada
            {
                let mut m = estados_dl().lock().expect("mutex dos estados");
                if let Some(s) = m.get_mut(id) {
                    s.feitos = feitos;
                    s.total = total;
                }
            }
            let _ = arquivo.flush();
            drop(arquivo);
            controle.store(0, Ordering::SeqCst);
            return Err(InstalarError::Execucao("__pausado__".to_string()));
        }
        let n = reader
            .read(&mut buf)
            .map_err(|e| InstalarError::Execucao(e.to_string()))?;
        if n == 0 {
            break;
        }
        feitos += n as u64;
        if feitos > limite {
            let _ = fs::remove_file(&tmp);
            descartar_estado_dl(id);
            return Err(InstalarError::Falha(
                "download maior que o permitido".to_string(),
            ));
        }
        arquivo
            .write_all(&buf[..n])
            .map_err(|e| InstalarError::Execucao(e.to_string()))?;
        if ultimo_tick.elapsed() >= std::time::Duration::from_millis(500) {
            let decorrido = ultimo_tick.elapsed().as_secs_f64();
            let vel = ((feitos - feitos_amostra) as f64 / decorrido) as u64;
            feitos_amostra = feitos;
            ultimo_tick = std::time::Instant::now();
            progresso_dl(id, feitos, total, vel);
            if let Some(s) = estados_dl()
                .lock()
                .expect("mutex dos estados")
                .get_mut(id)
            {
                s.feitos = feitos;
                s.total = total;
            }
        }
    }
    let _ = arquivo.flush();
    drop(arquivo);
    descartar_estado_dl(id);
    progresso_fase(id, "instalando");

    // valida o formato real do arquivo antes de extrair: evita tratar
    // uma página HTML de erro como se fosse um tarball do Proton
    let mut cabeca = [0u8; 6];
    fs::File::open(&tmp)
        .and_then(|mut a| a.read_exact(&mut cabeca))
        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
    let gzip = cabeca[0] == 0x1f && cabeca[1] == 0x8b;
    let xz = cabeca[0] == 0xfd && cabeca[1] == 0x37 && cabeca[2] == 0x7a && cabeca[3] == 0x58 && cabeca[4] == 0x5a;
    let zstd = cabeca[0] == 0x28 && cabeca[1] == 0xb5 && cabeca[2] == 0x2f && cabeca[3] == 0xfd;
    if !(gzip || xz || zstd) {
        let _ = fs::remove_file(&tmp);
        return Err(InstalarError::Falha(
            "o arquivo baixado não é um tarball válido (gzip/xz/zstd): o release pode não conter o asset esperado".to_string(),
        ));
    }

    // extrai para compatibilitytools.d
    if let Some(cb) = &mut prog {
        cb(None, "Extraindo para o Steam...");
    }
    let Some(destino) = diretorio_proton() else {
        return Err(InstalarError::Execucao(
            "compatibilitytools.d não encontrado".to_string(),
        ));
    };
    let out = Command::new("tar")
        .args([
            "-xf",
            &tmp.to_string_lossy(),
            "-C",
            &destino.to_string_lossy(),
        ])
        .output()
        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
    if !out.status.success() {
        return Err(InstalarError::Falha(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ));
    }
    let _ = fs::remove_file(&tmp);

    // verificação final: o diretório extraído precisa conter o runtime
    let detectado = diretorio_proton()
        .and_then(|d| fs::read_dir(d).ok())
        .map(|rd| {
            rd.flatten()
                .any(|entry| {
                    let e = entry.file_name().to_string_lossy().to_lowercase();
                    e.contains(f.detect) || e.contains("proton")
                })
        })
        .unwrap_or(false);
    if !detectado {
        return Err(InstalarError::Falha(
            "a extração não produziu o diretório do Proton esperado: consulte os logs".to_string(),
        ));
    }

    Ok(format!(
        "Instalado em {}; reinicie o Steam para aparecer em Proton experimental",
        destino.display()
    ))
}
