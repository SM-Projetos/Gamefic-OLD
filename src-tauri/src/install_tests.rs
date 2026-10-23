use crate::system::install::{detail, forks_proton, forks_wine, versoes_proton};

#[test]
fn debug_forks() {
    for w in forks_wine() {
        eprintln!("WINE {} disp={} atual={:?} repo={:?}", w.chave, w.disponivel, w.versao_atual, w.versao_repo);
    }
    for p in forks_proton() {
        eprintln!("PROTON {} disp={} repo={:?}", p.chave, p.disponivel, p.versao_repo);
    }
    if let Some(d) = detail("steam") {
        for f in d.fontes {
            eprintln!("STEAM {} disp={} inst={} {:?}", f.fonte, f.disponivel, f.instalado, f.versao);
        }
    }
}

#[test]
fn debug_versoes() {
    for fork in ["ge", "cachyos", "dw", "rtsp", "em", "wineland", "luxtorpeda"] {
        match versoes_proton(fork) {
            Ok(v) => eprintln!("VERSOES {fork}: {:?}", &v[..v.len().min(4)]),
            Err(e) => eprintln!("VERSOES {fork} ERRO {e}"),
        }
    }
}
