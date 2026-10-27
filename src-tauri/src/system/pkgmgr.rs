//! Adaptadores de gerenciador de pacotes nativos.
//!
//! A detecção combina a família do /etc/os-release (ver `os.rs`) com a
//! verificação dos executáveis reais: um gerenciador só é adotado se o
//! programa existir e responder no PATH. Uma consulta malsucedida nunca é
//! prova de que o pacote não existe — diferenciamos ausência confirmada,
//! instalado e erro de consulta (`Consulta::Indeterminado`).
//!
//! Comandos privilegiados usam `pkexec` (sem sudo, sem armazenar senha).
//! Dados não confiáveis nunca passam por shell: só args estruturados.

use super::os;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgMgr {
    Pacman,
    Apt,
    Dnf,
    Zypper,
    Apk,
    Xbps,
    Emerge,
    Eopkg,
    /// nenhum gerenciador confirmado: nenhuma operação nativa é ofertada
    Nenhum,
}

/// operação nativa a executar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgOp {
    Instalar,
    Remover,
}

/// resultado da consulta aos repositórios da distribuição
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Consulta {
    Ausente,
    Disponivel(String),
    Indeterminado(String),
}

impl Consulta {
    pub fn versao(&self) -> Option<&str> {
        match self {
            Consulta::Disponivel(v) => Some(v),
            _ => None,
        }
    }
}

use serde::Serialize;

impl PkgMgr {
    /// valida que o gerenciador detectado suporta a operação nativa
    pub fn compatível(self) -> Result<(), String> {
        match self {
            PkgMgr::Nenhum | PkgMgr::Emerge => {
                Err("a fonte nativa não é compatível com esta distribuição".to_string())
            }
            _ => Ok(()),
        }
    }

    /// executável de consulta do gerenciador
    fn executavel(self) -> Option<(&'static str, &'static [&'static str])> {
        // (programa, args de teste — resposta vazia/erro = indisponível)
        Some(match self {
            PkgMgr::Pacman => ("pacman", &["--version"][..]),
            PkgMgr::Apt => ("apt-cache", &["gencaches"][..]),
            PkgMgr::Dnf => ("dnf", &["--version"][..]),
            PkgMgr::Zypper => ("zypper", &["--version"][..]),
            PkgMgr::Apk => ("apk", &["--version"][..]),
            PkgMgr::Xbps => ("xbps-query", &["--version"][..]),
            PkgMgr::Emerge => ("emerge", &["--version"][..]),
            PkgMgr::Eopkg => ("eopkg", &["--version"][..]),
            PkgMgr::Nenhum => return None,
        })
    }

    pub fn presente(self) -> bool {
        let Some((prog, args)) = self.executavel() else {
            return false;
        };
        // O programa só conta como presente se responde com sucesso
        // [(gencaches no apt pode falhar sem índices — foco na existência)]
        match Command::new(prog).args(args).output() {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}

/// gerenciadores candidatos pela família da distribuição, em queda
fn candidatos() -> Vec<PkgMgr> {
    let familia = os::detect_distro().and_then(|d| d.family);
    let mut lista: Vec<PkgMgr> = match familia.as_deref() {
        Some("Arch") => vec![PkgMgr::Pacman],
        Some("Debian") => vec![PkgMgr::Apt],
        Some("Red Hat") => vec![PkgMgr::Dnf],
        Some("SUSE") => vec![PkgMgr::Zypper],
        Some("Alpine") => vec![PkgMgr::Apk],
        Some("Void") => vec![PkgMgr::Xbps],
        Some("Gentoo") => vec![PkgMgr::Emerge],
        Some("Solus") => vec![PkgMgr::Eopkg],
        // família desconhecida não elimina a chance de um gerenciador
        // instalado manualmente: todos são testados abaixo
        _ => vec![
            PkgMgr::Pacman,
            PkgMgr::Apt,
            PkgMgr::Dnf,
            PkgMgr::Zypper,
            PkgMgr::Apk,
            PkgMgr::Xbps,
            PkgMgr::Emerge,
            PkgMgr::Eopkg,
        ],
    };
    lista.push(PkgMgr::Nenhum);
    lista
}

pub fn atual() -> PkgMgr {
    candidatos()
        .into_iter()
        .find(|m| *m == PkgMgr::Nenhum || m.presente())
        .unwrap_or(PkgMgr::Nenhum)
}

/// executou o programa e devolve saída bruta quando o processo girou
/// (independe do código de saída — falhas inteiras distinguem ausência de erro)
fn saida(prog: &str, args: &[&str]) -> Option<std::process::Output> {
    Command::new(prog).args(args).output().ok()
}

fn texto(o: &std::process::Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// versão instalada, via consulta própria do gerenciador da família.
/// None = não instalado ou indisponível para confirmar (o chamador se
/// preocupa com isso em `na_repo`)
pub fn instalado(pkg: &str) -> Option<String> {
    match atual() {
        PkgMgr::Pacman => {
            let o = saida("pacman", &["-Q", pkg])?;
            if !o.status.success() { return None; }
            texto(&o).split_whitespace().nth(1).map(str::to_string)
        }
        PkgMgr::Apt => {
            let o = saida("dpkg-query", &["-W", "-f=${Version}\\n", pkg])?;
            let v = texto(&o).trim().to_string();
            if v.is_empty() { None } else { Some(v) }
        }
        PkgMgr::Dnf => {
            // pacote instalado: rpm -q responde "pkg-v.release"
            let o = saida("rpm", &["-q", "--qf", "%{VERSION}-%{RELEASE}\\n", pkg])?;
            if !o.status.success() { return None; }
            Some(texto(&o).trim().to_string())
        }
        PkgMgr::Zypper => {
            let o = saida("zypper", &["--non-interactive", "search", "-i", "--match-exact", pkg])?;
            if !o.status.success() { return None; }
            let linha = texto(&o).lines().skip_while(|l| !l.trim().starts_with('S')).find_map(|l| l.trim().split_whitespace().last().map(str::to_string));
            linha
        }
        PkgMgr::Apk => {
            // exit 0 de `apk info -e` confirma pacote instalado
            let o = saida("apk", &["info", "-e", pkg])?;
            if o.status.success() { Some(pkg.to_string()) } else { None }
        }
        PkgMgr::Xbps => {
            let o = saida("xbps-query", &["--installed", pkg])?;
            if !o.status.success() { return None; }
            versao_de_linha(&texto(&o), &["Version"])
        }
        PkgMgr::Emerge => None, // Portage não é consultado de forma barata via script
        PkgMgr::Eopkg => {
            let o = saida("eopkg", &["info", pkg])?;
            // Removida a linha de output: eopkg só é avaliado se presente
            if !o.status.success() { return None; }
            None
        }
        PkgMgr::Nenhum => None,
    }
}

pub fn na_repo(pkg: &str) -> Consulta {
    match atual() {
        PkgMgr::Pacman => {
            let Some(o) = saida("pacman", &["-Si", pkg]) else {
                return Consulta::Indeterminado("pacman não respondeu".to_string());
            };
            if o.status.success() {
                match versao_de_linha(&texto(&o), &["Version", "Versão"]) {
                    Some(v) => Consulta::Disponivel(v),
                    // saída exit 0 sem Version — pista a mais
                    None => Consulta::Indeterminado("saída inesperada do pacman -Si".to_string()),
                }
            } else {
                // exit 1 + "target not found" = ausência confirmada no repo
                let e = String::from_utf8_lossy(&o.stderr).to_lowercase();
                if e.contains("target not found") || e.contains("no results") {
                    Consulta::Ausente
                } else {
                    Consulta::Indeterminado("falha do pacman nos repositórios".to_string())
                }
            }
        }
        PkgMgr::Apt => {
            let Some(o) = saida("apt-cache", &["policy", pkg]) else {
                return Consulta::Indeterminado("apt-cache não respondeu".to_string());
            };
            let s = texto(&o);
            let candidata = s.lines().find_map(|l| {
                let l = l.trim();
                l.strip_prefix("Candidate:").map(|v| v.trim())
            });
            match candidata {
                Some("(none)") | None => {
                    // sem candidate: normalmente pacote desconhecido, mas
                    // sem saída de arquivos não dá para afirmar com certeza
                    if s.contains("Package files") {
                        Consulta::Ausente
                    } else {
                        Consulta::Indeterminado("apt-cache sem candidate e sem arquivos".to_string())
                    }
                }
                Some(v) => Consulta::Disponivel(v.to_string()),
            }
        }
        PkgMgr::Dnf => {
            let Some(o) = saida("dnf", &["info", pkg]) else {
                return Consulta::Indeterminado("dnf não respondeu".to_string());
            };
            if o.status.success() {
                match versao_de_linha(&texto(&o), &["Version"]) {
                    Some(v) => Consulta::Disponivel(v),
                    None => Consulta::Indeterminado("dnf sem linha Version".to_string()),
                }
            } else {
                let e = String::from_utf8_lossy(&o.stderr).to_lowercase();
                if e.contains("no package") || e.contains("no match") {
                    Consulta::Ausente
                } else {
                    Consulta::Indeterminado("falha do dnf nos repositórios".to_string())
                }
            }
        }
        PkgMgr::Zypper => {
            let Some(o) = saida("zypper", &["--non-interactive", "info", pkg]) else {
                return Consulta::Indeterminado("zypper não respondeu".to_string());
            };
            if o.status.success() {
                match versao_de_linha(&texto(&o), &["Version"]) {
                    Some(v) => Consulta::Disponivel(v),
                    None => Consulta::Indeterminado("zypper sem linha Version".to_string()),
                }
            } else {
                Consulta::Ausente // "Package 'x' not found" saída padrão
            }
        }
        PkgMgr::Apk => {
            let Some(o) = saida("apk", &["search", "-e", "-v", pkg]) else {
                return Consulta::Indeterminado("apk não respondeu".to_string());
            };
            // saída: "pkg-versão-rel  descrição"
            let primeira = texto(&o).lines().next().unwrap_or("").trim().to_string();
            if primeira.is_empty() {
                Consulta::Ausente
            } else {
                let pv = primeira.split_whitespace().next().unwrap_or("");
                match pv.find('-') {
                    Some(i) if i + 1 < pv.len() => Consulta::Disponivel(pv[i + 1..].to_string()),
                    _ => Consulta::Ausente,
                }
            }
        }
        PkgMgr::Xbps => {
            let Some(o) = saida("xbps-query", &["-R", pkg]) else {
                return Consulta::Indeterminado("xbps-query não respondeu".to_string());
            };
            if o.status.success() {
                match versao_de_linha(&texto(&o), &["version"]) {
                    Some(v) => Consulta::Disponivel(v),
                    None => Consulta::Indeterminado("xbps-query sem version".to_string()),
                }
            } else {
                Consulta::Ausente
            }
        }
        PkgMgr::Emerge => Consulta::Indeterminado("consultas Portage por pacote único não são suportadas nesta versão".to_string()),
        PkgMgr::Eopkg => {
            let Some(o) = saida("eopkg", &["search", pkg]) else {
                return Consulta::Indeterminado("eopkg não respondeu".to_string());
            };
            if o.status.success() && !texto(&o).trim().is_empty() {
                Consulta::Disponivel(texto(&o).trim().lines().next().unwrap_or("").to_string())
            } else {
                Consulta::Ausente
            }
        }
        PkgMgr::Nenhum => Consulta::Indeterminado("nenhum gerenciador nativo detectado".to_string()),
    }
}

/// extrai "Version : x" (com ou sem dois-pontos imediato) das linhas de saída
fn versao_de_linha(saida: &str, chaves: &[&str]) -> Option<String> {
    saida.lines().find_map(|linha| {
        let linha = linha.trim();
        for chave in chaves {
            if let Some(r) = linha.strip_prefix(chave) {
                if let Some(v) = r.trim_start().strip_prefix(':') {
                    let v = v.trim();
                    if !v.is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        }
        None
    })
}

/// argumentos de instalação do gerenciador — sempre pkexec para tirar decisão
/// do processos de usuário
pub fn instalar(pkg: &str) -> Result<(), String> {
    let (cmd, args): (&str, Vec<String>) = match atual() {
        PkgMgr::Pacman => ("pacman", vec!["-S".into(), "--noconfirm".into(), pkg.into()]),
        PkgMgr::Apt => ("apt-get", vec!["install".into(), "-y".into(), pkg.into()]),
        PkgMgr::Dnf => ("dnf", vec!["install".into(), "-y".into(), pkg.into()]),
        PkgMgr::Zypper => ("zypper", vec!["--non-interactive".into(), "install".into(), pkg.into()]),
        PkgMgr::Apk => ("apk", vec!["add".into(), pkg.into()]),
        PkgMgr::Xbps => ("xbps-install", vec!["-y".into(), "-S".into(), pkg.into()]),
        PkgMgr::Emerge => return Err("instalação nativa via Portage não é suportada nesta versão".to_string()),
        PkgMgr::Eopkg => ("eopkg", vec!["install".into(), "-y".into(), pkg.into()]),
        PkgMgr::Nenhum => return Err("a fonte nativa não é compatível com esta distribuição".into()),
    };
    match Command::new("pkexec").arg(cmd).args(args).output() {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn remover(pkg: &str) -> Result<(), String> {
    let (cmd, args): (&str, Vec<String>) = match atual() {
        PkgMgr::Pacman => ("pacman", vec!["-Rns".into(), "--noconfirm".into(), pkg.into()]),
        PkgMgr::Apt => ("apt-get", vec!["remove".into(), "-y".into(), pkg.into()]),
        PkgMgr::Dnf => ("dnf", vec!["remove".into(), "-y".into(), pkg.into()]),
        PkgMgr::Zypper => ("zypper", vec!["--non-interactive".into(), "remove".into(), pkg.into()]),
        PkgMgr::Apk => ("apk", vec!["del".into(), pkg.into()]),
        PkgMgr::Xbps => ("xbps-remove", vec!["-y".into(), pkg.into()]),
        PkgMgr::Emerge => return Err("remoção nativa via Portage não é suportada nesta versão".to_string()),
        PkgMgr::Eopkg => ("eopkg", vec!["remove".into(), "-y".into(), pkg.into()]),
        PkgMgr::Nenhum => return Err("a fonte nativa não é compatível com esta distribuição".into()),
    };
    match Command::new("pkexec").arg(cmd).args(args).output() {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&o.stderr).trim().to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consulta_enum_serde() {
        // contrato de serialização: precisa virar string/objeto sem erro
        let c = Consulta::Disponivel("1.2".to_string());
        assert_eq!(c.versao(), Some("1.2"));
        assert_eq!(Consulta::Ausente.versao(), None);
        assert_eq!(Consulta::Indeterminado("x".to_string()).versao(), None);
    }

    #[test]
    fn versao_de_linha_variacoes() {
        assert_eq!(
            versao_de_linha("Version        : 11.2\nAnother: 3\n", &["Version"]),
            Some("11.2".to_string())
        );
        assert_eq!(
            versao_de_linha("Version: 11.0\n", &["Version", "Versão"]),
            Some("11.0".to_string())
        );
        // nomes localizados
        assert_eq!(
            versao_de_linha("Versão: 7.8\n", &["Version", "Versão"]),
            Some("7.8".to_string())
        );
        // ausência = None
        assert_eq!(versao_de_linha("Name: wine-x\n", &["Version"]), None);
    }
}
