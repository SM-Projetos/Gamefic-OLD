use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::GpuInfo;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuMetrics {
    pub utilization_percent: Option<f32>,
    pub mem_used_mb: Option<f32>,
    pub mem_total_mb: Option<f32>,
    pub temp_c: Option<f32>,
}

pub fn metrics() -> Option<GpuMetrics> {
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next()?;
    let fields: Vec<&str> = line.split(',').map(|f| f.trim()).collect();

    if fields.len() < 4 {
        return None;
    }

    Some(GpuMetrics {
        utilization_percent: fields.first().and_then(|f| f.parse().ok()),
        mem_used_mb: fields.get(1).and_then(|f| f.parse().ok()),
        mem_total_mb: fields.get(2).and_then(|f| f.parse().ok()),
        temp_c: fields.get(3).and_then(|f| f.parse().ok()),
    })
}

pub fn detect() -> Vec<GpuInfo> {
    let mut gpus: Vec<GpuInfo> = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
        for entry in entries.flatten() {
            let path = entry.path();

            let class = fs::read_to_string(path.join("class"))
                .map(|c| c.trim().to_string())
                .unwrap_or_default();

            if !is_vga_or_display(&class) {
                continue;
            }

            gpus.push(inspect_device(&path));
        }
    }

    let pci_ids = load_pci_ids();
    if let Some((vendors, devices)) = &pci_ids {
        for gpu in &mut gpus {
            if let Some(vendor) = &gpu.vendor_id {
                let vendor_name = vendors.get(vendor).cloned();
                if let Some(vendor_name) = vendor_name {
                    gpu.vendor_name = Some(vendor_name);
                }
                if let Some(device_id) = &gpu.device_id {
                    if let Some(devices_of_vendor) = devices.get(vendor) {
                        if let Some(device_name) = devices_of_vendor.get(device_id) {
                            gpu.device_name = Some(device_name.clone());
                        }
                    }
                }
            }
        }
    }

    if let Ok(output) = run("lspci", &["-k", "-mm"]) {
        apply_lspci_output(gpus.as_mut_slice(), &output);
    }

    gpus.sort_by(|a, b| a.pci_address.cmp(&b.pci_address));
    gpus
}

fn is_vga_or_display(class: &str) -> bool {
    class.starts_with("0x0300") || class.starts_with("0x0302")
}

fn inspect_device(path: &Path) -> GpuInfo {
    GpuInfo {
        pci_address: Some(
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
        ),
        vendor_id: hex_value(fs::read_to_string(path.join("vendor"))),
        device_id: hex_value(fs::read_to_string(path.join("device"))),
        vendor_name: None,
        device_name: None,
        driver: driver_of(path),
        driver_version: None,
    }
}

fn hex_value(raw: std::io::Result<String>) -> Option<String> {
    let text = raw.ok()?;
    let trimmed = text.trim().trim_start_matches("0x").to_uppercase();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn driver_of(path: &Path) -> Option<String> {
    fs::read_link(path.join("driver"))
        .ok()?
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
}

type PciIds = (
    HashMap<String, String>,
    HashMap<String, HashMap<String, String>>,
);

fn load_pci_ids() -> Option<PciIds> {
    let text = fs::read_to_string("/usr/share/hwdata/pci.ids")
        .or_else(|_| fs::read_to_string("/usr/share/misc/pci.ids"))
        .ok()?;

    let mut vendors: HashMap<String, String> = HashMap::new();
    let mut devices: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_vendor: Option<String> = None;

    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if !line.starts_with(|c: char| c == '\t' || c == ' ') {
            let Some((id, name)) = line.split_once(char::is_whitespace) else {
                continue;
            };
            if id.len() != 4 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
                continue;
            }
            vendors.insert(id.to_uppercase(), name.trim().to_string());
            current_vendor = Some(id.to_uppercase());
            continue;
        }

        if line.starts_with("\t\t") || line.starts_with("  ") {
            continue;
        }

        let (Some(vendor), Some((id, name))) = (
            current_vendor.as_deref(),
            line.trim_start().split_once(char::is_whitespace),
        ) else {
            continue;
        };

        if id.len() != 4 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }

        devices
            .entry(vendor.to_string())
            .or_default()
            .insert(id.to_uppercase(), name.trim().to_string());
    }

    Some((vendors, devices))
}

fn run(program: &str, args: &[&str]) -> Result<String, std::io::Error> {
    let output = std::process::Command::new(program).args(args).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "programa externo retornou status de erro",
        ))
    }
}

fn apply_lspci_output(gpus: &mut [GpuInfo], output: &str) {
    let mut current_addr: Option<String> = None;

    for line in output.lines() {
        if let Some(addr_end) = line.find(" VGA compatible controller:") {
            current_addr = Some(line[..addr_end].trim().to_string());
        } else if let Some(addr_end) = line.find(" Display controller:") {
            current_addr = Some(line[..addr_end].trim().to_string());
        } else if line.starts_with('\t') {
        }

        if let (Some(addr), driver) = (current_addr.clone(), extract_driver(line)) {
            if let Some(gpu) = gpus
                .iter_mut()
                .find(|g| g.pci_address.as_deref() == Some(addr.as_str()))
            {
                if gpu.driver.is_none() {
                    gpu.driver = driver;
                }
            }
        }
    }
}

fn extract_driver(line: &str) -> Option<String> {
    let driver = line.trim_start().strip_prefix("Kernel driver in use: ")?;
    Some(driver.trim().to_string())
}
