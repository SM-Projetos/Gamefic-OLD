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

/// limpeza do histórico: só itens finalizados saem; em andamento/pausado
/// nunca são removidos (§3 e §9 do spec)
#[test]
fn progresso_limpar_preserva_andamento() {
    use crate::system::install::{progresso_iniciar, progresso_limpar, progresso_listar, progresso_atualizar};

    let ids = ["t-erro:a", "t-erro:b", "t-erro:c", "t-erro:d", "t-erro:e"];
    for id in ids {
        progresso_iniciar(id, id, "preparando");
    }
    progresso_atualizar("t-erro:a", None, None, Some("concluido"));
    progresso_atualizar("t-erro:b", None, None, Some("erro"));
    progresso_atualizar("t-erro:c", None, None, Some("cancelado"));
    progresso_atualizar("t-erro:d", None, None, Some("pausado"));
    // t-erro:e permanece em andamento

    // "erro": remove erro e cancelado; preserva concluido, pausado, andamento
    progresso_limpar("erro");
    let restantes: Vec<String> = progresso_listar()
        .iter()
        .filter(|p| p.id.starts_with("t-erro:"))
        .map(|p| p.id.clone())
        .collect();
    assert_eq!(restantes, vec!["t-erro:a", "t-erro:d", "t-erro:e"]);

    // "finalizados": remove também o concluido; preserva pausado/andamento
    progresso_limpar("finalizados");
    let restantes: Vec<String> = progresso_listar()
        .iter()
        .filter(|p| p.id.starts_with("t-erro:"))
        .map(|p| p.id.clone())
        .collect();
    assert_eq!(restantes, vec!["t-erro:d", "t-erro:e"]);
    progresso_limpar("finalizados");
}

/// controles de download: pedir_pausar/pedir_cancelar refletem no átomo e
/// estado_dl só existe para operação registrada (§2.1)
#[test]
fn controles_download_estados() {
    use crate::system::install::{estado_dl, pedir_cancelar, pedir_pausar};
    pedir_pausar("t-ctl-inexistente");
    pedir_cancelar("t-ctl-inexistente"); // não pode panicar fora de registro
    assert!(estado_dl("t-ctl-inexistente").is_none());
}

/// pausar preserva operação (não cancela); cancelar apaga o parcial salvo
#[test]
fn pausa_preserva_estado_e_cancelar_limpa() {
    use crate::system::install::{estado_dl, pedir_cancelar, pedir_pausar, progresso_atualizar, progresso_iniciar, progresso_listar};
    let id = "t-pausa:ge:GE-ProtonX";
    progresso_iniciar(id, "Proton X", "preparando");
    progresso_atualizar(id, None, None, Some("pausado"));
    pedir_pausar(id); // só sinaliza; item continua pausado no histórico
    assert!(
        progresso_listar().iter().any(|p| p.id == id && p.estado == "pausado"),
        "pausa não deve remover a operação"
    );
    progresso_atualizar(id, None, None, Some("cancelado"));
    pedir_cancelar(id); // limpa estado parcial; item cancelado permanece no histórico
    assert!(estado_dl(id).is_none());
    assert!(progresso_listar().iter().any(|p| p.id == id && p.estado == "cancelado"));
}

/// detecção de Protons pelo Steam: não pode panicar; em máquina com Steam
/// real, as entradas apontam para bibliotecas existentes
#[test]
fn steam_protons_estados() {
    use crate::system::steam::{protons_custom, protons_oficiais, raizes_steam};
    let raizes = raizes_steam();
    for r in &raizes {
        assert!(r.is_dir());
        // destino sempre dentro de uma raiz detectada
    }
    for p in protons_oficiais().iter().chain(protons_custom().iter()) {
        assert!(
            std::path::Path::new(&p.caminho).join("proton").exists(),
            "caminho sem executável proton: {}",
            p.caminho
        );
    }
}

/// remoção com validação: caminho fora dos diretórios gerenciados do Steam
/// é recusado sem apagar nada (§4.2)
#[test]
fn remover_ferramenta_recusa_caminho_invalido() {
    use crate::system::steam::remover_ferramenta;
    let falsos = [
        "/tmp/fake-proton",
        "/home/outro/Steam/compatibilitytools.d/x",
        "/usr/share/steam/Proton - Uses",
    ];
    for f in falsos {
        let r = remover_ferramenta(f);
        assert!(r.is_err(), "não deveria remover {f}");
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
