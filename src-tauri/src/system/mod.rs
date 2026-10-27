pub mod appinfo;
pub mod cpu;
pub mod gpu;
pub mod icons;
pub mod install;
pub mod memory;
pub mod os;

pub mod pkgmgr;

pub mod steam;
pub mod sensors;
pub mod storage;
pub mod tools;
pub mod vulkan;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub cores_physical: Option<u32>,
    pub threads_logical: Option<u32>,
    pub base_freq_mhz: Option<u32>,
    pub current_freqs_mhz: Vec<u32>,
    pub usage_percent: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_kb: Option<u64>,
    pub used_kb: Option<u64>,
    pub available_kb: Option<u64>,
    pub swap_total_kb: Option<u64>,
    pub swap_used_kb: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub pci_address: Option<String>,
    pub vendor_id: Option<String>,
    pub vendor_name: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub driver: Option<String>,
    pub driver_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VulkanGpu {
    pub device_name: Option<String>,
    pub driver_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VulkanInfo {
    pub gpus: Vec<VulkanGpu>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorReading {
    pub chip: String,
    pub label: String,
    pub celsius: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageDevice {
    pub name: String,
    pub model: Option<String>,
    pub kind: Option<String>,
    pub size_bytes: Option<u64>,
    pub used_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub available: bool,
    pub version: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub available: bool,
    pub version: Option<String>,
    pub icon: Option<String>,
    /// "instalado" | "nao_instalado" | "desconhecido" (§2.2)
    pub estado: String,
    /// formato de instalação identificado (Flatpak, nativo, binário, Steam)
    pub formato: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistroInfo {
    pub name: String,
    pub version: Option<String>,
    pub family: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemSnapshot {
    pub distro: Option<DistroInfo>,
    pub uptime_seconds: Option<u64>,
    pub storage: Vec<StorageDevice>,
    pub kernel: Option<String>,
    pub arch: Option<String>,
    pub cpu: Option<CpuInfo>,
    pub cpu_temp_c: Option<f32>,
    pub mesa_version: Option<String>,
    pub memory: Option<MemoryInfo>,
    pub gpus: Vec<GpuInfo>,
    pub vulkan: Option<VulkanInfo>,
    pub temperatures: Vec<SensorReading>,
    pub tools: Vec<ToolInfo>,
    pub launchers: Vec<AppEntry>,
    pub emuladores: Vec<AppEntry>,
    pub compatibilidade: Vec<AppEntry>,
}

pub fn snapshot() -> SystemSnapshot {
    let temperatures = sensors::detect();

    // Versão do driver agregada por GPU (Mesa para drivers de código aberto; nvidia-smi para o driver próprio)
    let gpus: Vec<GpuInfo> = detect_gpus_with_driver_versions();

    let mesa_version = parse_mesa_version().or_else(|| {
        gpus.iter()
            .find(|g| g.driver.as_deref() != Some("nvidia"))
            .and_then(|g| g.driver_version.clone())
    });

    SystemSnapshot {
        distro: os::detect_distro(),
        uptime_seconds: os::detect_uptime_seconds(),
        storage: storage::detect(),
        kernel: os::detect_kernel(),
        arch: os::detect_arch(),
        cpu: cpu::detect(),
        memory: memory::detect(),
        gpus,
        vulkan: vulkan::detect(),
        cpu_temp_c: detect_cpu_temp(&temperatures),
        mesa_version,
        temperatures,
        tools: tools::detect(),
        launchers: tools::launchers(),
        emuladores: tools::emuladores(),
        compatibilidade: tools::compatibility(),
    }
}

fn run(program: &str, args: &[&str]) -> std::io::Result<String> {
    let output = std::process::Command::new(program).args(args).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "falha do programa externo",
        ))
    }
}

fn detect_gpus_with_driver_versions() -> Vec<GpuInfo> {
    gpu::detect()
        .into_iter()
        .map(|mut g| {
            g.driver_version = detect_driver_version(g.driver.clone());
            g
        })
        .collect()
}

fn detect_driver_version(driver: Option<String>) -> Option<String> {
    let driver = driver?;
    if driver == "nvidia" {
        parse_nvidia_driver_version()
    } else {
        parse_mesa_version()
    }
}

fn parse_mesa_version() -> Option<String> {
    limpar_mesa(pacote_mesa())
}

// Remove epoch de pacote ("3:26.2.4-1" -> "26.2.4-1") e sufixos Debian ("~")
fn limpar_mesa(version: Option<String>) -> Option<String> {
    let version = version?;
    let version = version.split_whitespace().next()?.to_string();
    if let Some(pos) = version.find(':') {
        if version[..pos].chars().all(|c| c.is_ascii_digit()) && pos > 0 {
            return Some(version[pos + 1..].to_string());
        }
    }
    let version = version.split('~').next().unwrap_or(&version).to_string();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

fn pacote_mesa() -> Option<String> {
    // 1) glxinfo -B → "OpenGL version string: 4.6 ... Mesa 24.3.1"
    if let Ok(output) = run("glxinfo", &["-B"]) {
        if let Some(line) = output.lines().find(|l| l.contains("OpenGL version string")) {
            if let Some(value) = line.split("OpenGL version string:").nth(1) {
                let value = value.trim();
                if let Some(pos) = value.find("Mesa") {
                    let version = value[pos..]
                        .trim_start_matches("Mesa ")
                        .split_whitespace()
                        .next()
                        .map(|s| s.to_string())
                        .filter(|v| !v.is_empty());
                    if version.is_some() {
                        return version;
                    }
                }
            }
        }
    }

    // 2) Gerenciador de pacotes da família da distro
    if let Ok(output) = run("pacman", &["-Q", "mesa"]) {
        if let Some(version) = output.split_whitespace().nth(1) {
            return Some(version.trim().to_string());
        }
    }
    if let Ok(output) = run("dpkg-query", &["-W", "-f=${Version}\\n", "libgl1-mesa-dri"]) {
        let version = output.trim();
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    if let Ok(output) = run("rpm", &["-q", "mesa-dri-drivers"]) {
        if let Some(version) = output.trim().strip_prefix("mesa-dri-drivers-") {
            return Some(version.to_string());
        }
    }
    None
}

fn parse_nvidia_driver_version() -> Option<String> {
    let output = run(
        "nvidia-smi",
        &["--query-gpu=driver_version", "--format=csv,noheader"],
    )
    .ok()?;
    output.lines().next()?.trim().to_string().into()
}

fn detect_cpu_temp(temperatures: &[SensorReading]) -> Option<f32> {
    let package = temperatures
        .iter()
        .find(|t| t.label.eq_ignore_ascii_case("package id 0"));
    let fanless = temperatures.iter().find(|t| {
        let l = t.label.to_lowercase();
        l.contains("package") || l == "tdie" || l == "tctl" || l == "cpu"
    });
    let first_core = temperatures.iter().find(|t| {
        let l = t.label.to_lowercase();
        l.contains("core 0")
            || (t.chip.contains("k10temp") || t.chip.contains("coretemp")) && l.starts_with("temp1")
    });
    let fallback = if temperatures.is_empty() {
        None
    } else {
        Some(&temperatures[0])
    };
    package
        .or(fanless)
        .or(first_core)
        .or(fallback)
        .map(|t| t.celsius)
}
