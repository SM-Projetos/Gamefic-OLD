use std::fs;
use std::path::PathBuf;

use super::tools::base64;

const MAX_ICONE: usize = 600_000;

fn emb(key: &str) -> Option<(&'static [u8], &'static str)> {
    match key {
        "steam" => Some((include_bytes!("../../assets/icons/steam.png"), "image/png")),
        "lutris" => Some((
            include_bytes!("../../assets/icons/lutris.svg"),
            "image/svg+xml",
        )),
        "heroic" => Some((include_bytes!("../../assets/icons/heroic.png"), "image/png")),
        "bottles" => Some((
            include_bytes!("../../assets/icons/bottles.svg"),
            "image/svg+xml",
        )),
        "itch" => Some((include_bytes!("../../assets/icons/itch.png"), "image/png")),
        "minigalaxy" => Some((
            include_bytes!("../../assets/icons/minigalaxy.png"),
            "image/png",
        )),
        "mangohud" => Some((
            include_bytes!("../../assets/icons/mangohud.svg"),
            "image/svg+xml",
        )),
        "lact" => Some((include_bytes!("../../assets/icons/lact.png"), "image/png")),
        "goverlay" => Some((
            include_bytes!("../../assets/icons/goverlay.svg"),
            "image/svg+xml",
        )),
        "wine" => Some((
            include_bytes!("../../assets/icons/wine.svg"),
            "image/svg+xml",
        )),
        "winetricks" => Some((
            include_bytes!("../../assets/icons/winetricks.svg"),
            "image/svg+xml",
        )),
        "proton" => Some((include_bytes!("../../assets/icons/proton.png"), "image/png")),
        "retroarch" => Some((
            include_bytes!("../../assets/icons/retroarch.svg"),
            "image/svg+xml",
        )),
        "dolphin" => Some((
            include_bytes!("../../assets/icons/dolphin.svg"),
            "image/svg+xml",
        )),
        "pcsx2" => Some((include_bytes!("../../assets/icons/pcsx2.png"), "image/png")),
        "rpcs3" => Some((
            include_bytes!("../../assets/icons/rpcs3.ico"),
            "image/x-icon",
        )),
        "ppsspp" => Some((include_bytes!("../../assets/icons/ppsspp.png"), "image/png")),
        "duck" => Some((include_bytes!("../../assets/icons/duck.png"), "image/png")),
        "mgba" => Some((include_bytes!("../../assets/icons/mgba.png"), "image/png")),
        "mame" => Some((
            include_bytes!("../../assets/icons/mame.svg"),
            "image/svg+xml",
        )),
        _ => None,
    }
}

fn fonte_url(key: &str) -> Option<&'static str> {
    match key {
        "lutris" => Some(
            "https://raw.githubusercontent.com/lutris/lutris/master/share/icons/hicolor/scalable/apps/net.lutris.Lutris.svg",
        ),
        "heroic" => Some(
            "https://raw.githubusercontent.com/Heroic-Games-Launcher/HeroicGamesLauncher/main/public/icon.png",
        ),
        "bottles" => Some(
            "https://raw.githubusercontent.com/bottlesdevs/Bottles/main/data/icons/hicolor/scalable/apps/com.usebottles.bottles.svg",
        ),
        "itch" => Some(
            "https://raw.githubusercontent.com/itchio/itch/master/release/images/itch-icons/icon256.png",
        ),
        "minigalaxy" => Some(
            "https://raw.githubusercontent.com/sharkwouter/minigalaxy/master/data/icons/192x192/io.github.sharkwouter.Minigalaxy.png",
        ),
        "mangohud" => Some(
            "https://raw.githubusercontent.com/flightlessmango/MangoHud/master/data/io.github.flightlessmango.mangohud.svg",
        ),
        "lact" => Some(
            "https://raw.githubusercontent.com/ilya-zlobintsev/LACT/master/res/io.github.ilya_zlobintsev.LACT.png",
        ),
        "goverlay" => Some(
            "https://raw.githubusercontent.com/benjamimgois/goverlay/main/data/icons/goverlay.svg",
        ),
        "wine" => Some(
            "https://raw.githubusercontent.com/wine-mirror/wine/master/programs/winecfg/logo.svg",
        ),
        _ => None,
    }
}

pub fn chaves_embutidas() -> Vec<&'static str> {
    const CHAVES: &[&str] = &[
        "steam",
        "lutris",
        "heroic",
        "bottles",
        "itch",
        "minigalaxy",
        "mangohud",
        "lact",
        "goverlay",
        "wine",
        "winetricks",
        "proton",
        "retroarch",
        "dolphin",
        "pcsx2",
        "rpcs3",
        "ppsspp",
        "duck",
        "mgba",
        "mame",
    ];
    CHAVES.to_vec()
}

fn diretorio_cache() -> Option<PathBuf> {
    let base = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{h}/.local/share"))
        })?;
    let dir = PathBuf::from(base).join("com.sergio.gamefic").join("icons");
    fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn ler_cache(key: &str) -> Option<(Vec<u8>, &'static str)> {
    let dir = diretorio_cache()?;
    for (exto, mime) in [
        ("svg", "image/svg+xml"),
        ("png", "image/png"),
        ("ico", "image/x-icon"),
    ] {
        let path = dir.join(format!("{key}.{exto}"));
        if let Ok(bytes) = fs::read(&path) {
            if !bytes.is_empty() && bytes.len() <= MAX_ICONE {
                return Some((bytes, mime));
            }
        }
    }
    None
}

pub fn icone_data_uri(key: &str, system_paths: &[&str]) -> Option<String> {
    if let Some((bytes, mime)) = ler_cache(key) {
        return Some(format!("data:{mime};base64,{}", base64(&bytes)));
    }
    if let Some((bytes, mime)) = emb(key) {
        return Some(format!("data:{mime};base64,{}", base64(bytes)));
    }
    super::tools::first_icon(system_paths)
}

pub fn byte_aceito(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > MAX_ICONE {
        return false;
    }
    bytes.starts_with(b"\x89PNG")
        || bytes.starts_with(b"GIF8")
        || bytes.starts_with(&[0xFF, 0xD8, 0xFF])
        || bytes.starts_with(b"<svg")
        || bytes.starts_with(b"<?xml")
        || bytes.starts_with(b"\x00\x00\x01\x00")
}

pub fn refresh_cache() -> u32 {
    let mut atualizados = 0u32;
    let Some(dir) = diretorio_cache() else {
        return 0;
    };
    for key in chaves_embutidas() {
        let Some(url) = fonte_url(key) else {
            continue;
        };
        let bytes: Vec<u8> = match ureq::get(url)
            .timeout(std::time::Duration::from_secs(20))
            .call()
        {
            Ok(resp) => {
                use std::io::Read;
                let mut v = Vec::new();
                let resultado = resp
                    .into_reader()
                    .take(MAX_ICONE as u64 + 1)
                    .read_to_end(&mut v);
                match resultado {
                    Ok(_) => v,
                    Err(_) => continue,
                }
            }
            Err(_) => continue,
        };
        if !byte_aceito(&bytes) {
            continue;
        }
        let exto = if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
            "svg"
        } else if bytes.starts_with(b"\x00\x00\x01\x00") {
            "ico"
        } else {
            "png"
        };
        let nome = dir.join(format!("{key}.{exto}"));
        if fs::read(&nome).map(|atual| atual != bytes).unwrap_or(true) {
            if fs::write(&nome, &bytes).is_ok() {
                atualizados += 1;
            }
        }
    }
    atualizados
}
