pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod os;
pub mod sensors;
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
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub available: bool,
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
    pub kernel: Option<String>,
    pub arch: Option<String>,
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub gpus: Vec<GpuInfo>,
    pub vulkan: Option<VulkanInfo>,
    pub temperatures: Vec<SensorReading>,
    pub tools: Vec<ToolInfo>,
}

pub fn snapshot() -> SystemSnapshot {
    SystemSnapshot {
        distro: os::detect_distro(),
        kernel: os::detect_kernel(),
        arch: os::detect_arch(),
        cpu: cpu::detect(),
        memory: memory::detect(),
        gpus: gpu::detect(),
        vulkan: vulkan::detect(),
        temperatures: sensors::detect(),
        tools: tools::detect(),
    }
}
