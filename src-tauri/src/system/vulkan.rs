use std::process::Command;

use super::{VulkanGpu, VulkanInfo};

pub fn detect() -> Option<VulkanInfo> {
    let output = Command::new("vulkaninfo").arg("--summary").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).into_owned();

    let mut gpus: Vec<VulkanGpu> = Vec::new();
    let mut current_device: Option<String> = None;
    let mut current_driver: Option<String> = None;

    for line in text.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("deviceName") {
            if let Some(previous) = current_device.take() {
                gpus.push(VulkanGpu {
                    device_name: Some(previous),
                    driver_name: current_driver.take(),
                });
            }
            current_device = value_of(rest);
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("driverName") {
            current_driver = value_of(rest);
            continue;
        }

        if trimmed.starts_with("deviceUUID")
            || trimmed.starts_with("apiVersion")
            || trimmed.starts_with("deviceType")
        {
            if let Some(device) = current_device.take() {
                gpus.push(VulkanGpu {
                    device_name: Some(device),
                    driver_name: current_driver.take(),
                });
            }
        }
    }

    if let Some(device) = current_device.take() {
        gpus.push(VulkanGpu {
            device_name: Some(device),
            driver_name: current_driver.take(),
        });
    }

    if gpus.is_empty() {
        None
    } else {
        Some(VulkanInfo { gpus })
    }
}

fn value_of(rest: &str) -> Option<String> {
    let (_, value) = rest.split_once('=')?;
    let value = value.trim();
    if value.is_empty() {
        None
    } else if let Some((first, _)) = value.split_once('|') {
        Some(first.trim().to_string())
    } else {
        Some(value.to_string())
    }
}
