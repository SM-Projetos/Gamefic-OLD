//! Adaptadores de gerenciador de pacotes nativos, escolhidos pela família
//! da distribuição detectada via /etc/os-release (ver `os.rs`).
//!
//! Família Arch → pacman; Debian → apt; Red Hat → dnf.
//! Comandos privilegiados usam `pkexec` (sem sudo, sem armazenar senha).
//! Dados não confiáveis nunca são passados para shell: só args estruturados.

use super::os;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgMgr {
    Pacman,
    Apt,
    Dnf,
    /// família não suportada: nenhuma operação nativa é ofertada
    Nenhum,
}

/// operação nativa a executar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PkgOp {
    Instalar,
    Remover,
}

impl PkgMgr {
    /// valida que o gerenciador da família suporta a operação nativa
    pub fn compatível(self) -> Result<(), String> {
        match self {
            PkgMgr::Nenhum => Err("a fonte nativa não é compatível com esta distribuição".to_string()),
            _ => Ok(()),
        }
    }
}

pub fn atual() -> PkgMgr {
    match os::detect_distro().and_then(|d| d.family) {
        Some(f) if f == "Arch" => PkgMgr::Pacman,
        Some(f) if f == "Debian" => PkgMgr::Apt,
        Some(f) if f == "Red Hat" || f == "SUSE" => PkgMgr::Dnf,
        _ => PkgMgr::Nenhum,
    }
}

fn rodar(prog: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(prog).args(args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        None
    }
}

/// versão instalada, via consulta própria do gerenciador da família
pub fn instalado(pkg: &str) -> Option<String> {
    match atual() {
        PkgMgr::Pacman => {
            let s = rodar("pacman", &["-Q", pkg])?;
            s.split_whitespace().nth(1).map(str::to_string)
        }
        PkgMgr::Apt => {
            let s = rodar("dpkg-query", &["-W", "-f=${Version}\\n", pkg])?;
            let v = s.trim();
            if v.is_empty() { None } else { Some(v.to_string()) }
        }
        PkgMgr::Dnf => {
            let s = rodar("rpm", &["-q", "--qf", "%{VERSION}\\n", pkg])?;
            let v = s.trim();
            if v.is_empty() { None } else { Some(v.to_string()) }
        }
        PkgMgr::Nenhum => None,
    }
}

/// versão candidata nos repositórios habilitados (sem instalar nada)
pub fn na_repo(pkg: &str) -> Option<String> {
    match atual() {
        PkgMgr::Pacman => {
            let s = rodar("pacman", &["-Si", pkg])?;
            s.lines().find_map(|l| {
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
        PkgMgr::Apt => {
            let s = rodar("apt-cache", &["policy", pkg])?;
            s.lines().find_map(|l| {
                let l = l.trim();
                l.strip_prefix("Candidate:")
                    .map(|v| v.trim())
                    .filter(|v| !v.is_empty() && *v != "(none)")
                    .map(str::to_string)
            })
        }
        PkgMgr::Dnf => {
            let s = rodar("dnf", &["info", pkg])?;
            s.lines().find_map(|l| {
                let l = l.trim();
                l.strip_prefix("Version")
                    .and_then(|r| r.trim_start().strip_prefix(':'))
                    .map(|v| v.trim().to_string())
            })
        }
        PkgMgr::Nenhum => None,
    }
}

pub fn instalar(pkg: &str) -> Result<(), String> {
    let (cmd, args): (String, Vec<String>) = match atual() {
        PkgMgr::Pacman => ("pkexec".into(), vec!["pacman".into(), "-S".into(), "--noconfirm".into(), pkg.into()]),
        PkgMgr::Apt => ("pkexec".into(), vec!["apt-get".into(), "install".into(), "-y".into(), pkg.into()]),
        PkgMgr::Dnf => ("pkexec".into(), vec!["dnf".into(), "install".into(), "-y".into(), pkg.into()]),
        PkgMgr::Nenhum => return Err("a fonte nativa não é compatível com esta distribuição".into()),
    };
    match Command::new(cmd).args(args).output() {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&o.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn remover(pkg: &str) -> Result<(), String> {
    let (cmd, args): (String, Vec<String>) = match atual() {        PkgMgr::Pacman => ("pkexec".into(), vec!["pacman".into(), "-Rns".into(), "--noconfirm".into(), pkg.into()]),
        PkgMgr::Apt => ("pkexec".into(), vec!["apt-get".into(), "remove".into(), "-y".into(), pkg.into()]),
        PkgMgr::Dnf => ("pkexec".into(), vec!["dnf".into(), "remove".into(), "-y".into(), pkg.into()]),
        PkgMgr::Nenhum => return Err("a fonte nativa não é compatível com esta distribuição".into()),
    };
    match Command::new(cmd).args(args).output() {
        Ok(o) => {
            if o.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8_lossy(&o.stderr).to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
