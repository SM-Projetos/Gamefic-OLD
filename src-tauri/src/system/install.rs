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
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDetail {
    pub id: String,
    pub nome: String,
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

fn pacote_repo(pkg: &str) -> Option<String> {
    let saida = rodar("pacman", &["-Si", pkg])?;
    saida.lines().find_map(|l| {
        let l = l.trim();
        for pref in ["Version", "Versão"] {
            if let Some(r) = l.strip_prefix(pref) {
                if let Some(v) = r.trim_start().strip_prefix(':') {
                    return Some(v.trim().to_string());
                }
            }
        }
        None
    })
}

fn pacote_instalado(pkg: &str) -> Option<String> {
    let saida = rodar("pacman", &["-Q", pkg])?;
    saida.split_whitespace().nth(1).map(|v| v.to_string())
}

fn flatpaks_instalados() -> Vec<String> {
    rodar("flatpak", &["list", "--app", "--columns=application"])
        .map(|s| s.lines().map(|l| l.trim().to_string()).collect())
        .unwrap_or_default()
}

fn flatpak_disponível(id: &str) -> bool {
    rodar("flatpak", &["remote-info", "--system", "flathub", id]).is_some()
        || rodar("flatpak", &["remote-info", "--user", "flathub", id]).is_some()
}

pub fn detail(id: &str) -> Option<AppDetail> {
    let info = achado(id)?;
    let instalados = flatpaks_instalados();

    let mut fontes = Vec::new();
    for pkg in info.nativos {
        let na_repo = pacote_repo(pkg);
        let local = pacote_instalado(pkg);
        fontes.push(FonteInfo {
            fonte: format!("nativo:{pkg}"),
            disponivel: na_repo.is_some(),
            instalado: local.is_some(),
            versao: local.or(na_repo),
        });
    }
    if let Some(fid) = info.flatpak {
        let está = instalados.iter().any(|l| l == fid);
        fontes.push(FonteInfo {
            fonte: "flatpak".to_string(),
            disponivel: está || flatpak_disponível(fid),
            instalado: está,
            versao: None,
        });
    }

    Some(AppDetail {
        id: info.id.to_string(),
        nome: info.nome.to_string(),
        fornecedor: info.fornecedor.to_string(),
        licenca: info.licenca.to_string(),
        desc_curta: info.desc_curta.to_string(),
        desc_longa: info.desc_longa.to_string(),
        icone: super::icons::icone_data_uri(id, &[]),
        fontes,
    })
}

// ---------- ações (install/remove/update) ----------

pub enum InstalarError {
    Fonte,
    Falha(String),
    NaoDisponivel,
    Execucao(String),
}

impl std::fmt::Display for InstalarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            InstalarError::Fonte => write!(f, "fonte desconhecida"),
            InstalarError::Falha(m) => write!(f, "{m}"),
            InstalarError::NaoDisponivel => write!(f, "fonte não está disponível"),
            InstalarError::Execucao(m) => write!(f, "falha na execução: {m}"),
        }
    }
}

pub fn acao_app(id: &str, acao: &str, fonte: &str) -> Result<String, InstalarError> {
    let info = achado(id).ok_or(InstalarError::Fonte)?;

    match fonte {
        f if f.starts_with("nativo:") => {
            let pkg = f.split(':').nth(1).ok_or(InstalarError::Fonte)?;
            match acao {
                "instalar" | "atualizar" => {
                    let out = Command::new("pkexec")
                        .args(["pacman", "-S", "--noconfirm", pkg])
                        .output()
                        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
                    if out.status.success() {
                        Ok("Instalação concluída".to_string())
                    } else {
                        Err(InstalarError::Falha(
                            String::from_utf8_lossy(&out.stderr).to_string(),
                        ))
                    }
                }
                "remover" => {
                    let out = Command::new("pkexec")
                        .args(["pacman", "-Rns", "--noconfirm", pkg])
                        .output()
                        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
                    if out.status.success() {
                        Ok("Remoção concluída".to_string())
                    } else {
                        Err(InstalarError::Falha(
                            String::from_utf8_lossy(&out.stderr).to_string(),
                        ))
                    }
                }
                _ => Err(InstalarError::Fonte),
            }
        }
        "flatpak" => {
            let Some(fid) = info.flatpak else {
                return Err(InstalarError::Fonte);
            };
            let out = match acao {
                "instalar" | "atualizar" => {
                    Command::new("flatpak")
                        .args([
                            "install", "--system", "--noninteractive", "-y", "flathub", fid,
                        ])
                        .output()
                        .map_err(|e| InstalarError::Execucao(e.to_string()))?
                }
                "remover" => Command::new("flatpak")
                    .args(["uninstall", "--system", "-y", "--noninteractive", fid])
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

// ---------- Wine ----------

pub fn forks_wine() -> Vec<InfoCompat> {
    let mut lista = Vec::new();
    for pkg in ["wine", "wine-staging", "wine-cachyos"] {
        let repo = pacote_repo(pkg);
        lista.push(InfoCompat {
            chave: pkg.to_string(),
            nome: match pkg {
                "wine" => "Wine (Stable)".to_string(),
                "wine-staging" => "Wine-Staging".to_string(),
                _ => "Wine CachyOS (otimizado)".to_string(),
            },
            disponivel: repo.is_some(),
            versao_atual: pacote_instalado(pkg),
            versao_repo: repo,
        });
    }
    lista
}

pub fn instalar_wine(pkg: &str) -> Result<String, InstalarError> {
    if !["wine", "wine-staging", "wine-cachyos"].contains(&pkg) {
        return Err(InstalarError::Fonte);
    }
    let out = Command::new("pkexec")
        .args(["pacman", "-S", "--noconfirm", pkg])
        .output()
        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
    if out.status.success() {
        Ok("Wine instalado".to_string())
    } else {
        Err(InstalarError::Falha(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ))
    }
}

// ---------- Proton ----------

pub(crate) const PROTON_MAX_MB: u64 = 900;

fn diretorio_proton() -> Option<PathBuf> {
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
            let json: Vec<serde_json::Value> = serde_json::from_str(&texto)
                .map_err(|e| InstalarError::Execucao(e.to_string()))?;
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
    let texto = http_texto(&format!("https://github.com/{repositorio}/releases.atom"), 2_000_000)?;
    let mut tags: Vec<String> = Vec::new();
    for bloco in texto.split("<entry>") {
        let Some(t) = bloco.split("</title>").next() else { continue };
        let Some(t) = t.split("<title>").nth(1) else { continue };
        let mut tag = t.trim().to_string();
        if tag == "Release notes from proton-ge-custom" || tag.ends_with("from proton-cachyos") {
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
        repositorio: "nanomatters/proton-cachyos",
        pacote: None,
        fonte: FonteProton::Github,
        detect: "wineland",
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
                a.iter()
                    .filter_map(|x| x["browser_download_url"].as_str())
                    .find(|u| {
                        let l = u.to_lowercase();
                        (l.ends_with(".tar.xz") || l.ends_with(".tar.gz")) && l.contains("x86")
                    })
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
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{h}/.local/share")))?;
    let dir = std::path::PathBuf::from(base).join("com.sergio.gamefic").join("cache");
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

pub fn versoes_proton(fork: &str) -> Result<Vec<String>, InstalarError> {    let f = fork_proton(fork)?;
    match f.fonte {
        FonteProton::Steam => Ok(Vec::new()),
        _ => Ok(releases_fork(f)?
            .into_iter()
            .map(|(t, _)| t)
            .collect()),
    }
}

fn asset_url_github(repositorio: &str, tag: &str) -> Result<String, InstalarError> {
    let page = http_texto(
        &format!("https://github.com/{repositorio}/releases/expanded_assets/{tag}"),
        2_000_000,
    )?;
    for bloco in page.split("href=\"") {
        let Some(href) = bloco.split('"').next() else { continue };
        let minic = href.to_lowercase();
        if (minic.ends_with(".tar.gz") || minic.ends_with(".tar.xz")) && minic.contains("proton") {
            return Ok(if href.starts_with("http") {
                href.to_string()
            } else {
                format!("https://github.com{href}")
            });
        }
    }
    // fallback: nome convencional do GE-Proton
    Ok(format!(
        "https://github.com/{repositorio}/releases/download/{tag}/{tag}.tar.gz"
    ))
}

pub fn instalar_proton(fork: &str, versao: &str) -> Result<String, InstalarError> {
    use std::io::Read;
    let f = fork_proton(fork)?;

    if f.fonte == FonteProton::Steam {
        return Err(InstalarError::Execucao(
            "o Proton padrão é gerenciado pelo próprio Steam".to_string(),
        ));
    }

    if let Some(p) = f.pacote {
        if versao == "pacman" {
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
    } else {
        "tar.gz"
    };

    // download para /tmp
    let tmp = std::env::temp_dir().join(format!("gamefic-{}.{}", fork, extensao));
    let mut reader = ureq::get(&url_download)
        .set("User-Agent", "gamefic")
        .timeout(std::time::Duration::from_secs(900))
        .call()
        .map_err(|e| InstalarError::Execucao(e.to_string()))?
        .into_reader();
    let mut arquivo = fs::File::create(&tmp).map_err(|e| InstalarError::Execucao(e.to_string()))?;
    std::io::copy(&mut reader.by_ref().take(PROTON_MAX_MB * 1024 * 1024), &mut arquivo)
        .map_err(|e| InstalarError::Execucao(e.to_string()))?;

    // extrai para compatibilitytools.d
    let Some(destino) = diretorio_proton() else {
        return Err(InstalarError::Execucao(
            "compatibilitytools.d não encontrado".to_string(),
        ));
    };
    let out = Command::new("tar")
        .args(["-xf", &tmp.to_string_lossy(), "-C", &destino.to_string_lossy()])
        .output()
        .map_err(|e| InstalarError::Execucao(e.to_string()))?;
    if !out.status.success() {
        return Err(InstalarError::Falha(
            String::from_utf8_lossy(&out.stderr).to_string(),
        ));
    }
    let _ = fs::remove_file(&tmp);

    Ok(format!(
        "Instalado em {}; reinicie o Steam para aparecer em Proton experimental",
        destino.display()
    ))
}
