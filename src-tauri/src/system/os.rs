use super::DistroInfo;

pub fn detect_distro() -> Option<DistroInfo> {
    let text = std::fs::read_to_string("/etc/os-release").ok()?;

    let mut fields: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim_matches('"').to_string();
            fields.insert(key.trim().to_string(), value);
        }
    }

    let name = fields
        .get("PRETTY_NAME")
        .cloned()
        .or_else(|| fields.get("NAME").cloned())?;

    Some(DistroInfo {
        name,
        version: fields.get("VERSION_ID").cloned(),
        family: detect_family(&fields),
    })
}

/// família ampla da distribuição, a partir de ID e ID_LIKE do os-release.
/// Não é prova do gerenciador: pkgmgr valida os executáveis por cima disto.
fn detect_family(fields: &std::collections::HashMap<String, String>) -> Option<String> {
    if let Some(id) = fields.get("ID") {
        let id = id.to_lowercase();
        if id == "cachyos" || id == "arch" || id == "manjaro" || id == "endeavouros" {
            return Some("Arch".to_string());
        }
        if id == "alpine" {
            return Some("Alpine".to_string());
        }
        if id == "void" {
            return Some("Void".to_string());
        }
        if id == "gentoo" || id == "funtoo" {
            return Some("Gentoo".to_string());
        }
        if id == "solus" {
            return Some("Solus".to_string());
        }
        if id == "nixos" {
            return Some("NixOS".to_string());
        }
        if id == "fedora" {
            return Some("Red Hat".to_string());
        }
        if id == "opensuse-tumbleweed" || id == "opensuse-leap" || id == "opensuse" || id == "sled" || id == "sles" {
            return Some("SUSE".to_string());
        }
    }

    let id_like = fields.get("ID_LIKE")?;
    let like_lower = id_like.to_lowercase();

    if like_lower.contains("arch") {
        Some("Arch".to_string())
    } else if like_lower.contains("debian") {
        Some("Debian".to_string())
    } else if like_lower.contains("rhel") || like_lower.contains("fedora") {
        Some("Red Hat".to_string())
    } else if like_lower.contains("suse") {
        Some("SUSE".to_string())
    } else if like_lower.contains("alpine") {
        Some("Alpine".to_string())
    } else if like_lower.contains("void") {
        Some("Void".to_string())
    } else if like_lower.contains("gentoo") {
        Some("Gentoo".to_string())
    } else if like_lower.contains("solus") {
        Some("Solus".to_string())
    } else if like_lower.contains("nixos") {
        Some("NixOS".to_string())
    } else {
        None
    }
}

pub fn detect_kernel() -> Option<String> {
    match std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        Ok(release) => {
            let trimmed = release.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Err(_) => None,
    }
}

pub fn detect_uptime_seconds() -> Option<u64> {
    let text = std::fs::read_to_string("/proc/uptime").ok()?;
    let first = text.split_whitespace().next()?;
    first.parse::<f64>().ok().map(|s| s as u64)
}

pub fn detect_arch() -> Option<String> {
    // primeiro o arquivo do kernel (não requer processo); fallback uname
    if let Ok(arch) = std::fs::read_to_string("/proc/sys/kernel/arch") {
        let trimmed = arch.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let out = std::process::Command::new("uname").arg("-m").output().ok()?;
    if out.status.success() {
        let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if v.is_empty() { None } else { Some(v) }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::detect_family;
    use std::collections::HashMap;

    fn f(k: &str, v: &str) -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert(k.to_string(), v.to_string());
        m
    }

    #[test]
    fn familias_id_detectadas() {
        assert_eq!(detect_family(&f("ID", "cachyos")).as_deref(), Some("Arch"));
        assert_eq!(detect_family(&f("ID", "void")).as_deref(), Some("Void"));
        assert_eq!(detect_family(&f("ID", "alpine")).as_deref(), Some("Alpine"));
        assert_eq!(detect_family(&f("ID", "solus")).as_deref(), Some("Solus"));
        assert_eq!(detect_family(&f("ID", "nixos")).as_deref(), Some("NixOS"));
        assert_eq!(detect_family(&f("ID", "gentoo")).as_deref(), Some("Gentoo"));
        assert_eq!(detect_family(&f("ID", "opensuse-leap")).as_deref(), Some("SUSE"));
        assert_eq!(detect_family(&f("ID", "fedora")).as_deref(), Some("Red Hat"));
    }

    #[test]
    fn familia_por_id_like() {
        assert_eq!(
            detect_family(&{
                let mut m = f("ID", "artix");
                m.insert("ID_LIKE".to_string(), "arch rolling".to_string());
                m
            })
            .as_deref(),
            Some("Arch")
        );
    }

    #[test]
    fn familia_desconhecida_e_nenhum() {
        assert_eq!(detect_family(&f("ID", "distro-inventada")).is_none(), true);
    }
}
