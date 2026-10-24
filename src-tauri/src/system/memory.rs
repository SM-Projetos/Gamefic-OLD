use std::fs;

use super::MemoryInfo;

pub fn detect() -> Option<MemoryInfo> {
    let text = fs::read_to_string("/proc/meminfo").ok()?;

    let mut total_kb: Option<u64> = None;
    let mut available_kb: Option<u64> = None;
    let mut swap_total_kb: Option<u64> = None;
    let mut swap_free_kb: Option<u64> = None;

    for line in text.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let kb = parse_kb(value);

        match key {
            "MemTotal" if total_kb.is_none() => total_kb = kb,
            "MemAvailable" if available_kb.is_none() => available_kb = kb,
            "SwapTotal" if swap_total_kb.is_none() => swap_total_kb = kb,
            "SwapFree" if swap_free_kb.is_none() => swap_free_kb = kb,
            _ => {}
        }
    }

    if total_kb.is_none() {
        return None;
    }

    let used_kb = match (total_kb, available_kb) {
        (Some(total), Some(available)) => Some(total.saturating_sub(available)),
        _ => None,
    };

    let swap_used_kb = match (swap_total_kb, swap_free_kb) {
        (Some(total), Some(free)) => Some(total.saturating_sub(free)),
        _ => None,
    };

    Some(MemoryInfo {
        total_kb,
        used_kb,
        available_kb,
        swap_total_kb,
        swap_used_kb,
    })
}

fn parse_kb(value: &str) -> Option<u64> {
    let head = value.split_whitespace().next()?;
    head.parse().ok()
}
