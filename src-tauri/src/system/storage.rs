use std::fs;
use std::path::Path;

use super::StorageDevice;

const VIRTUAL_PREFIXES: [&str; 9] = ["loop", "ram", "zram", "fd", "dm-", "md", "sr", "mtd", "nbd"];

pub fn detect() -> Vec<StorageDevice> {
    let mut devices: Vec<StorageDevice> = Vec::new();
    let usage = detect_usage();

    if let Ok(entries) = fs::read_dir("/sys/block") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();

            if VIRTUAL_PREFIXES.iter().any(|p| name.starts_with(p)) {
                continue;
            }

            let mut device = inspect_device(&entry.path(), &name);
            if !device.size_bytes.is_some_and(|s| s > 0) {
                continue;
            }
            if let Some((used, total)) = usage.get(&name) {
                device.used_bytes = Some(*used);
                device.total_bytes = Some(*total);
            }
            devices.push(device);
        }
    }

    devices.sort_by(|a, b| a.name.cmp(&b.name));
    devices
}

/// Espaço por disco somando as partições montadas (fonte: df).
fn detect_usage() -> std::collections::HashMap<String, (u64, u64)> {
    let mut usage: std::collections::HashMap<String, (u64, u64)> = Default::default();

    let output = match std::process::Command::new("df")
        .args(["-B1", "--output=source,size,used,target"])
        .output()
    {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => return usage,
    };

    for line in output.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        let source = fields[0];
        let size: u64 = fields[1].parse().unwrap_or(0);
        let used: u64 = fields[2].parse().unwrap_or(0);
        if size == 0 {
            continue;
        }
        let Some(dev) = source.strip_prefix("/dev/") else {
            continue;
        };
        // O disco base é o nome antes do sufixo da partição (sda1 -> sda; nvme0n1p2 -> nvme0n1)
        let disk = base_disk(dev);
        let entry = usage.entry(disk).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(used);
        entry.1 = entry.1.saturating_add(size);
    }
    usage
}

fn base_disk(dev: &str) -> String {
    // nvme0n1p2 -> nvme0n1
    if let Some(rest) = dev.strip_prefix("nvme") {
        if let Some(p_pos) = rest.find('p') {
            let tail = &rest[p_pos + 1..];
            if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
                return format!("nvme{}", rest[..p_pos].replace("p", "n"));
            }
        }
        return dev.to_string();
    }
    // mmcblk0p3 -> mmcblk0
    if let Some(p_pos) = dev.find('p') {
        if p_pos > 0
            && dev[..p_pos]
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_digit())
        {
            let tail = &dev[p_pos + 1..];
            if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
                return dev[..p_pos].to_string();
            }
        }
    }
    // sda1 -> sda
    if let Some(pos) = dev.find(|c: char| c.is_ascii_digit()) {
        return dev[..pos].to_string();
    }
    dev.to_string()
}

fn inspect_device(sysfs: &Path, name: &str) -> StorageDevice {
    let size_bytes = fs::read_to_string(sysfs.join("size"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .filter(|&size| size > 0)
        .map(|sectors| sectors * 512);

    let model = fs::read_to_string(sysfs.join("device/model"))
        .ok()
        .map(|m| m.trim_end_matches(' ').trim().to_string())
        .filter(|m| !m.is_empty());

    let rotational = fs::read_to_string(sysfs.join("queue/rotational"))
        .ok()
        .map(|r| r.trim() == "1")
        .unwrap_or(false);

    let kind = if name.starts_with("nvme") {
        Some("NVMe SSD".to_string())
    } else if rotational {
        Some("HDD".to_string())
    } else {
        Some("SATA SSD".to_string())
    };

    StorageDevice {
        name: name.to_string(),
        model,
        kind,
        size_bytes,
        used_bytes: None,
        total_bytes: None,
    }
}
