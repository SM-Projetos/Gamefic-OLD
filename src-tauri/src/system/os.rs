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

fn detect_family(fields: &std::collections::HashMap<String, String>) -> Option<String> {
    if let Some(id) = fields.get("ID") {
        if id == "cachyos" || id == "arch" {
            return Some("Arch".to_string());
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
    match std::fs::read_to_string("/proc/sys/kernel/arch") {
        Ok(arch) => {
            let trimmed = arch.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Err(_) => None,
    }
}
