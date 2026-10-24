use std::fs;

use super::SensorReading;

pub fn detect() -> Vec<SensorReading> {
    let mut readings: Vec<SensorReading> = Vec::new();

    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return readings;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.join("temp1_input").exists() {
            continue;
        }

        let chip = fs::read_to_string(path.join("name"))
            .map(|n| n.trim().to_string())
            .unwrap_or_else(|_| "desconhecido".to_string());

        let index = hwmon_index(&entry.file_name().to_string_lossy());

        for i in 1..=100 {
            let temp_input = path.join(format!("temp{i}_input"));
            if !temp_input.exists() {
                break;
            }

            let Ok(raw) = fs::read_to_string(&temp_input) else {
                continue;
            };

            let Ok(millidegrees) = raw.trim().parse::<f32>() else {
                continue;
            };
            if millidegrees <= -50000.0 || millidegrees >= 150000.0 {
                continue;
            }

            let label = fs::read_to_string(path.join(format!("temp{i}_label")))
                .map(|l| l.trim().to_string())
                .ok()
                .filter(|l| !l.is_empty())
                .unwrap_or_else(|| format!("temp{i}"));

            readings.push(SensorReading {
                chip: format!("{chip} {index}").trim().to_string(),
                label,
                celsius: millidegrees / 1000.0,
            });
        }
    }

    readings
}

fn hwmon_index(name: &str) -> String {
    match name.strip_prefix("hwmon") {
        Some(group) if !group.is_empty() => group.to_string(),
        _ => String::new(),
    }
}
