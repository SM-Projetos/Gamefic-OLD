use Availability::{Binario, PadraoFlatpak};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::icons::icone_data_uri;
use super::{AppEntry, ToolInfo};

pub(crate) const ICON_SIZE_LIMIT: u64 = 300_000;

pub fn detect() -> Vec<ToolInfo> {
    let definition: &[(&str, &'static str, Availability, &'static str, &[&str])] = &[
        (
            "gamemode",
            "GameMode",
            either(Binario("gamemoded"), Binario("gamemode")),
            "Daemon de otimização para jogos",
            &["/usr/share/icons/hicolor/256x256/apps/gamemode.png"],
        ),
        (
            "mangohud",
            "MangoHud",
            either(Binario("mangohud"), Binario("mangohud-gl.so")),
            "Overlay de desempenho em jogos",
            &["/usr/share/icons/hicolor/256x256/apps/mangohud.png"],
        ),
        (
            "gamescope",
            "Gamescope",
            Binario("gamescope"),
            "Compositor de sessão de jogos",
            &["/usr/share/icons/hicolor/256x256/apps/gamescope.png"],
        ),
        (
            "lact",
            "LACT",
            Binario("lact"),
            "Controle de GPU/ventoinha (AMD/NVIDIA/Intel)",
            &["/usr/share/icons/hicolor/scalable/apps/io.github.ilya_zlobintsev.LACT.png"],
        ),
        (
            "goverlay",
            "GOverlay",
            Binario("goverlay"),
            "Configuração gráfica do MangoHud",
            &["/usr/share/icons/hicolor/256x256/apps/goverlay.png"],
        ),
        (
            "winetricks",
            "Winetricks",
            Binario("winetricks"),
            "Utilitário de bibliotecas para prefixos Wine",
            &["/usr/share/icons/hicolor/256x256/apps/winetricks.png"],
        ),
        (
            "protontricks",
            "Protontricks",
            either(
                Binario("protontricks"),
                PadraoFlatpak("com.github.Matoking.protontricks"),
            ),
            "Winetricks dirigido por jogos Steam",
            &["/usr/share/icons/hicolor/256x256/apps/protontricks.png"],
        ),
    ];

    definition
        .iter()
        .map(|(key, name, availability, description, icons)| ToolInfo {
            id: (*key).to_string(),
            name: (*name).to_string(),
            description: (*description).to_string(),
            available: availability.clone().check(),
            version: None,
            icon: icone_data_uri(key, icons),
        })
        .collect()
}

pub fn launchers() -> Vec<AppEntry> {
    let definition: &[(&str, &'static str, Availability, &'static str, &[&str])] = &[
        (
            "steam",
            "Steam",
            either(Binario("steam"), PadraoFlatpak("com.valvesoftware.Steam")),
            "Loja e cliente de jogos da Valve",
            &[
                "/usr/share/pixmaps/steam.png",
                "/usr/share/icons/hicolor/256x256/apps/steam.png",
            ],
        ),
        (
            "lutris",
            "Lutris",
            either(Binario("lutris"), PadraoFlatpak("net.lutris.Lutris")),
            "Gerenciador de jogos Wine/Proton",
            &[
                "/usr/share/icons/hicolor/512x512/apps/net.lutris.Lutris.png",
                "/usr/share/pixmaps/lutris.png",
            ],
        ),
        (
            "heroic",
            "Heroic Games Launcher",
            either(
                Binario("heroic"),
                PadraoFlatpak("com.heroicgameslauncher.hgl"),
            ),
            "Launcher da Epic/GOG/Amazon",
            &["/usr/share/icons/hicolor/256x256/apps/com.heroicgameslauncher.hgl.png"],
        ),
        (
            "bottles",
            "Bottles",
            either(Binario("bottles"), PadraoFlatpak("com.usebottles.bottles")),
            "Gerenciador de prefixos Wine",
            &["/usr/share/icons/hicolor/256x256/apps/com.usebottles.bottles.png"],
        ),
        (
            "itch",
            "Itch",
            either(Binario("itch"), PadraoFlatpak("io.itch.itch")),
            "Loja indie da itch.io",
            &["/usr/share/pixmaps/itch.png"],
        ),
        (
            "minigalaxy",
            "Minigalaxy",
            Binario("minigalaxy"),
            "Cliente GOG simples",
            &["/usr/share/icons/hicolor/256x256/apps/com.github.sharkwouter.minigalaxy.png"],
        ),
    ];

    definition
        .iter()
        .map(|(key, name, _availability, description, icons)| {
            let estado = estado_instalacao(key);
            AppEntry {
                id: (*key).to_string(),
                name: (*name).to_string(),
                description: (*description).to_string(),
                available: estado.estado == "instalado",
                version: estado.versao.clone(),
                icon: icone_data_uri(key, icons),
                estado: estado.estado.clone(),
                formato: estado.formato,
            }
        })
        .collect()
}

pub fn emuladores() -> Vec<AppEntry> {
    let definition: &[(&str, &'static str, Availability, &'static str, &str)] = &[
        (
            "retroarch",
            "RetroArch",
            Binario("retroarch"),
            "Reúne vários emuladores em uma só interface",
            "/usr/share/icons/hicolor/256x256/apps/retroarch.png",
        ),
        (
            "dolphin",
            "Dolphin",
            either(Binario("dolphin-emu"), Binario("dolphin-emu-nogui")),
            "Nintendo GameCube e Wii",
            "/usr/share/icons/hicolor/256x256/apps/dolphin-emu.png",
        ),
        (
            "pcsx2",
            "PCSX2",
            either(Binario("pcsx2"), Binario("pcsx2-qt")),
            "PlayStation 2",
            "/usr/share/icons/hicolor/256x256/apps/pcsx2.png",
        ),
        (
            "rpcs3",
            "RPCS3",
            Binario("rpcs3"),
            "PlayStation 3",
            "/usr/share/icons/hicolor/256x256/apps/rpcs3.png",
        ),
        (
            "ppsspp",
            "PPSSPP",
            either(Binario("ppsspp"), Binario("PPSSPPSDL")),
            "PlayStation Portable (PSP)",
            "/usr/share/icons/hicolor/256x256/apps/ppsspp.png",
        ),
        (
            "duck",
            "DuckStation",
            Binario("duckstation"),
            "PlayStation 1",
            "/usr/share/icons/hicolor/256x256/apps/duckstation.png",
        ),
        (
            "mgba",
            "mGBA",
            either(Binario("mgba"), Binario("mgba-qt")),
            "Game Boy Advance",
            "/usr/share/icons/hicolor/256x256/apps/io.mgba.mGBA.png",
        ),
        (
            "mame",
            "MAME",
            Binario("mame"),
            "Jogos de arcade",
            "/usr/share/icons/hicolor/256x256/apps/mame.png",
        ),
    ];

    definition
        .iter()
        .map(
            |(key, name, availability, description, icon_path)| {
                let _ = availability;
                let estado = estado_instalacao(key);
                AppEntry {
                    id: (*key).to_string(),
                    name: (*name).to_string(),
                    description: (*description).to_string(),
                    available: estado.estado == "instalado",
                    version: estado.versao.clone(),
                    icon: icone_data_uri(key, &[*icon_path]),
                    estado: estado.estado.clone(),
                    formato: estado.formato,
                }
            },
        )
        .collect()
}

pub fn compatibility() -> Vec<AppEntry> {
    let mut entries: Vec<AppEntry> = Vec::new();

    entries.push(AppEntry {
        id: "wine".to_string(),
        name: "Wine".to_string(),
        description: "Camada de compatibilidade Windows".to_string(),
        available: Binario("wine").check(),
        version: versao("wine", &["--version"]).map(|v| v.lines().next().unwrap_or("").to_string()),
        icon: icone_data_uri(
            "wine",
            &[
                "/usr/share/icons/hicolor/256x256/apps/wine.png",
                "/usr/share/pixmaps/wine.png",
            ],
        ),
        estado: "desconhecido".to_string(),
        formato: None,
    });
    entries.last_mut().map(|e| {
        e.estado = if e.available { "instalado" } else { "nao_instalado" }.to_string();
        e.formato = e.available.then(|| "nativo".to_string());
    });

    // customizados em compatibilitytools.d (todas as raízes)
    for (label, version) in detectar_protons() {
        entries.push(AppEntry {
            id: format!("proton:{label}"),
            name: label.clone(),
            description: "Proton para Steam (compatibilitytools.d)".to_string(),
            available: true,
            version,
            icon: icone_data_uri("proton", &[]),
            estado: "instalado".to_string(),
            formato: Some("Steam (custom)".to_string()),
        });
    }

    // oficiais gerenciados pelo Steam, em cada biblioteca detectada
    for p in super::steam::protons_oficiais() {
        if entries.iter().any(|e| e.id == format!("proton:{}", p.nome)) {
            continue;
        }
        entries.push(AppEntry {
            id: format!("proton:{}", p.nome),
            name: p.nome.clone(),
            description: "Proton gerenciado pelo Steam".to_string(),
            available: true,
            version: p.versao,
            icon: icone_data_uri("proton", &[]),
            estado: "instalado".to_string(),
            formato: Some("Steam (oficial)".to_string()),
        });
    }

    entries
}

// ---------- detecção central do estado de instalação (§2) ----------
// Mesma fonte de verdade da página de detalhes: pacotes do gerenciador
// nativo, Flatpak por escopo, AppImage/diretórios do usuário e binário;
// conclusão sem consulta concluída vira "desconhecido", nunca "não instalado".

pub(crate) struct EstadoReal {
    pub estado: String,
    pub formato: Option<String>,
    pub versao: Option<String>,
}

/// detecção real de um aplicativo do catálogo pelo id, usando os
/// identificadores confiáveis do REGISTRY (pacote nativo, app-id do Flatpak)
pub(crate) fn estado_instalacao(id: &str) -> EstadoReal {
    let info = super::install::achado_pub(id);

    // flatpak by escopo (mesma consulta do detail())
    let flatpak_ids: Option<(Vec<(String, String)>, Vec<(String, String)>)> = Command::new("flatpak")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|_| {
            (
                super::install::flatpaks_escopo_pub("--system"),
                super::install::flatpaks_escopo_pub("--user"),
            )
        });

    if let Some(i) = info {
        if let Some(fid) = i.flatpak {
            match flatpak_ids {
                None => {
                    if i.nativos.iter().any(|p| super::pkgmgr::instalado(p.pkg).is_some()) {
                        let familia = super::os::detect_distro().and_then(|d| d.family);
                        return EstadoReal {
                            estado: "instalado".into(),
                            formato: Some("nativo".into()),
                            versao: i
                                .pacotes_aptos(familia.as_deref())
                                .find_map(|p| super::pkgmgr::instalado(p)),
                        };
                    }
                    // flatpak presente como única fonte conhecida mas consulta
                    // indisponível: não é possível descartar a instalação
                    return EstadoReal { estado: "desconhecido".into(), formato: None, versao: None };
                }
                Some((sistema, usuario)) => {
                    if let Some((_, v)) = sistema.iter().chain(usuario.iter()).find(|(a, _)| a == fid) {
                        let escopo = sistema.iter().any(|(a, _)| a == fid)
                            .then(|| "sistema")
                            .unwrap_or("usuário");
                        return EstadoReal {
                            estado: "instalado".into(),
                            formato: Some(format!("Flatpak ({escopo})")),
                            versao: (!v.is_empty()).then(|| v.clone()),
                        };
                    }
                }
            }
        }
        for pkg in i.pacotes_aptos(super::os::detect_distro().and_then(|d| d.family).as_deref()) {
            if let Some(v) = super::pkgmgr::instalado(pkg) {
                return EstadoReal {
                    estado: "instalado".into(),
                    formato: Some("nativo".into()),
                    versao: Some(v),
                };
            }
        }
        return EstadoReal { estado: "nao_instalado".into(), formato: None, versao: None };
    }

    // id fora do REGISTRY: consulta só por binário (o estado depende do
    // comando existir de forma confiável)
    let achou = consegui_executar("which", [id]);
    EstadoReal {
        estado: if achou { "instalado".into() } else { "nao_instalado".into() },
        formato: achou.then(|| "binário".to_string()),
        versao: None,
    }
}


/// Protons customizados instalados (compatibilitytools.d de cada raiz Steam)
pub(crate) fn detectar_protons() -> Vec<(String, Option<String>)> {
    let mut achados: Vec<String> = Vec::new();

    // todas as raízes Steam detectadas (nativa, Flatpak, Snap)
    let bases: Vec<PathBuf> = super::steam::raizes_steam()
        .iter()
        .map(|r| r.join("compatibilitytools.d"))
        .collect();

    for base in bases {
        if let Ok(entries) = fs::read_dir(&base) {
            for entry in entries.flatten() {
                let nome = entry.file_name().to_string_lossy().to_string();
                if !base.join(&nome).join("proton").exists() {
                    continue;
                }
                if !achados.contains(&nome) {
                    achados.push(nome.clone());
                }
            }
        }
    }

    achados.sort();
    achados.into_iter().map(|nome| (nome, None)).collect()
}

// ---------- tipos de disponibilidade (mesma semântica, sem shell) ----------

#[derive(Clone)]
enum Availability {
    Binario(&'static str),
    PadraoFlatpak(&'static str),
    Qualquer(Box<Availability>, Box<Availability>),
}

impl Availability {
    fn check(self) -> bool {
        match self {
            Availability::Binario(b) => consegui_executar("which", [b]),
            Availability::PadraoFlatpak(id) => consegui_executar("flatpak", ["info", "--app", id]),
            Availability::Qualquer(a, b) => a.check() || b.check(),
        }
    }
}

fn either(a: Availability, b: Availability) -> Availability {
    Availability::Qualquer(Box::new(a), Box::new(b))
}

fn consegui_executar<A: AsRef<str>>(programa: &str, args: impl IntoIterator<Item = A>) -> bool {
    let args: Vec<String> = args.into_iter().map(|a| a.as_ref().to_string()).collect();
    Command::new(programa)
        .args(&args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn versao(programa: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(programa).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let primeira = text.trim().lines().next().unwrap_or("").trim();
    if primeira.is_empty() {
        return None;
    }
    let util = primeira
        .split_whitespace()
        .find(|token| !token.eq_ignore_ascii_case(programa) && *token != "-")?;
    let mut curta: String = util.chars().take(40).collect();
    if curta.ends_with('.') {
        curta.pop();
    }
    Some(curta)
}

// ---------- ícones do sistema ----------

pub(crate) fn first_icon(paths: &[&str]) -> Option<String> {
    paths.iter().find_map(|path| icone_base64(Path::new(path)))
}

fn icone_base64(path: &Path) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    if metadata.len() == 0 || metadata.len() > ICON_SIZE_LIMIT {
        return None;
    }
    let mime = match path.extension().and_then(|e| e.to_str()) {
        Some("svg") => "image/svg+xml",
        _ => "image/png",
    };
    let bytes = fs::read(path).ok()?;
    Some(format!("data:{};base64,{}", mime, base64(&bytes)))
}

pub(crate) fn base64(data: &[u8]) -> String {
    const TBL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TBL[((n >> 18) & 63) as usize] as char);
        out.push(TBL[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TBL[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TBL[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}
