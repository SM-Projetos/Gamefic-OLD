use std::process::Command;

use super::ToolInfo;

const CANDIDATES: &[(&str, &str, &str)] = &[
    ("Steam", "steam", "Loja e cliente de jogos"),
    ("Lutris", "lutris", "Gerenciador de jogos wine/proton"),
    ("Wine", "wine", "Camada de compatibilidade Windows"),
    ("Heroic Games Launcher", "heroic", "Launcher da Epic/GOG/Amazon"),
    ("Bottles", "bottles", "Gerenciador de prefixos Wine"),
    ("Gamemode", "gamemoded", "Daemon de otimização para jogos"),
    ("MangoHud", "mangohud", "Overlay de desempenho em jogos"),
    ("Gamescope", "gamescope", "Compositor de sessão de jogos"),
    ("CoreCtrl", "corectrl", "Controle de CPU/GPU/disco"),
    ("Governor atual (cpupower)", "cpupower", "Ferramenta de governadores de CPU"),
];

pub fn detect() -> Vec<ToolInfo> {
    CANDIDATES
        .iter()
        .map(|(name, binary, description)| ToolInfo {
            name: (*name).to_string(),
            description: (*description).to_string(),
            available: is_available(binary),
        })
        .collect()
}

fn is_available(binary: &str) -> bool {
    match Command::new("which").arg(binary).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
