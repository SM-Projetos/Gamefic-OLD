//! Diagnóstico manual das fontes de um aplicativo do catálogo no sistema atual.
//! Uso: cargo run --example dump_fontes -- steam bottles wine retroarch
fn main() {
    let ids: Vec<String> = std::env::args().skip(1).collect();
    let ids = if ids.is_empty() {
        vec!["steam".to_string(), "bottles".to_string(), "wine".to_string(), "retroarch".to_string()]
    } else {
        ids
    };
    let mgr = match gamefic_lib::system::pkgmgr::atual() {
        gamefic_lib::system::pkgmgr::PkgMgr::Pacman => "pacman",
        gamefic_lib::system::pkgmgr::PkgMgr::Apt => "apt",
        gamefic_lib::system::pkgmgr::PkgMgr::Dnf => "dnf",
        gamefic_lib::system::pkgmgr::PkgMgr::Zypper => "zypper",
        gamefic_lib::system::pkgmgr::PkgMgr::Apk => "apk",
        gamefic_lib::system::pkgmgr::PkgMgr::Xbps => "xbps",
        gamefic_lib::system::pkgmgr::PkgMgr::Emerge => "emerge",
        gamefic_lib::system::pkgmgr::PkgMgr::Eopkg => "eopkg",
        _ => "nenhum",
    };
    println!("gerenciador: {mgr}");
    for id in ids {
        if let Some(d) = gamefic_lib::system::install::detail(&id) {
            println!("== {} ({}):", d.id, d.nome);
            for f in &d.fontes {
                println!(
                    "   {} disp={} inst={} indet={} vrepo={:?} motivo={:?}",
                    f.fonte, f.disponivel, f.instalado, f.indeterminado, f.versao_repo, f.motivo
                );
            }
        }
    }
}
