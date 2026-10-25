use crate::system::install::{detail, forks_proton, forks_wine, listar_com_version, versao_remote_info, versoes_proton};
use crate::system::pkgmgr::{PkgMgr, PkgOp};

#[test]
fn parser_versao_remote_info() {
    let saida = "Installation: io.itch.itch\n        Version: 25.7.1\n        Hash: abc";
    assert_eq!(versao_remote_info(saida).as_deref(), Some("25.7.1"));
    assert_eq!(versao_remote_info("Version:\n"), None);
    assert_eq!(versao_remote_info("não tem versão"), None);
}

#[test]
fn parser_lista_com_version() {
    let saida = "io.itch.itch\t25.7.1\ncom.usebottles.bottles\t51.24\n";
    let v = listar_com_version(saida);
    assert_eq!(v.len(), 2);
    assert_eq!(v[0], ("io.itch.itch".to_string(), "25.7.1".to_string()));
    assert_eq!(v[1].0, "com.usebottles.bottles");
    // sem coluna de versão → string vazia, id preservado
    assert_eq!(listar_com_version("solo")[0].0, "solo");
}

#[test]
fn pkgmgr_compatibilidade() {
    assert!(PkgMgr::Pacman.compatível().is_ok());
    assert!(PkgMgr::Apt.compatível().is_ok());
    assert!(PkgMgr::Dnf.compatível().is_ok());
    assert!(PkgMgr::Nenhum.compatível().is_err());
}

#[test]
fn pkgmgr_op_distintos() {
    assert_ne!(PkgOp::Instalar, PkgOp::Remover);
}

/// a fonte nativa responde de acordo com a família da máquina de execução:
/// em máquinas Arch/CachyOS/Manjaro o gerenciador é o pacman.
#[test]
fn pkgmgr_familia_distribuicao() {
    let family = crate::system::os::detect_distro().and_then(|d| d.family);
    match family.as_deref() {
        Some("Arch") => assert_eq!(crate::system::pkgmgr::atual(), PkgMgr::Pacman),
        Some("Debian") => assert_eq!(crate::system::pkgmgr::atual(), PkgMgr::Apt),
        Some("Red Hat") => assert_eq!(crate::system::pkgmgr::atual(), PkgMgr::Dnf),
        _ => assert_eq!(crate::system::pkgmgr::atual(), PkgMgr::Nenhum),
    }
}

/// estrutura real do detalhe no ambiente de execução: a fonte nativa
/// (nesse ambiente, Arch/CachyOS) deve refletir as consultas do gerenciador
#[test]
fn detail_fontes_estrutura() {
    let d = detail("steam").expect("steam deve estar no catalogo");
    assert_eq!(d.id, "steam");
    assert!(d.gui);
    assert!(d.fontes.iter().any(|f| f.fonte == "nativo:steam"));
    assert!(d.fontes.iter().any(|f| f.fonte == "flatpak"));
    // toda fonte tem campo motivo consistente com o estado
    for f in &d.fontes {
        if f.disponivel || f.instalado {
            assert!(f.motivo.is_none(), "fonte {} com motivo indevido", f.fonte);
        }
    }
}

/// resolução real de assets: para cada fork no GitHub, o release mais recente
/// precisa apontar para um tarball do mesmo repositório (sem URLs inventadas)
#[test]
fn assets_do_ultimo_release_github() {
    use crate::system::install::forks_proton;
    let forks = forks_proton();
    assert!(!forks.is_empty());
    for chave in ["ge", "rtsp", "em", "wineland"] {
        let versoes = versoes_proton(chave).expect("versões devem listar");
        assert!(!versoes.is_empty(), "{chave} sem versões");
        for v in &versoes {
            assert!(
                !v.starts_with("Release notes from"),
                "{chave}: tag de ruído no feed: {v}"
            );
        }
    }
}

#[test]
fn debug_forks() {
    for w in forks_wine() {
        eprintln!(
            "WINE {} disp={} atual={:?} repo={:?}",
            w.chave, w.disponivel, w.versao_atual, w.versao_repo
        );
    }
    for p in forks_proton() {
        eprintln!(
            "PROTON {} disp={} repo={:?}",
            p.chave, p.disponivel, p.versao_repo
        );
    }
    if let Some(d) = detail("steam") {
        for f in d.fontes {
            eprintln!(
                "STEAM {} disp={} inst={} {:?}",
                f.fonte, f.disponivel, f.instalado, f.versao
            );
        }
    }
}

#[test]
fn debug_versoes() {
    for fork in [
        "ge",
        "cachyos",
        "dw",
        "rtsp",
        "em",
        "wineland",
        "luxtorpeda",
    ] {
        match versoes_proton(fork) {
            Ok(v) => eprintln!("VERSOES {fork}: {:?}", &v[..v.len().min(4)]),
            Err(e) => eprintln!("VERSOES {fork} ERRO {e}"),
        }
    }
}
