use orco_ra::*;

fn main() {
    let mut frontend = RAFrontend::new();
    let path = std::fs::canonicalize("samples/simple.rs").unwrap();
    let vpath = ra::vfs::VfsPath::new_real_path(path.to_string_lossy().into_owned());
    frontend
        .vfs
        .set_file_contents(vpath.clone(), std::fs::read(path).ok());
    let file_id = frontend.vfs.file_id(&vpath).unwrap().0;

    frontend.crate_graph.add_crate_root(
        file_id,
        ra_ap_ide::Edition::CURRENT,
        None,
        None,
        ra::hir::CfgOptions::default(),
        None,
        Default::default(),
        ra::CrateOrigin::Local {
            repo: None,
            name: None,
        },
        false,
        vpath.as_path().unwrap().parent().unwrap().to_owned().into(),
        ra::CrateWorkspaceData {
            data_layout: Err("fixture has no layout".into()),
            toolchain: None,
        }
        .into(),
    );
    frontend.apply_changes(true);
    println!("Status: {}", frontend.status(file_id));

    for krate in ra::hir::Crate::all(frontend.db()) {
        for module in krate.modules(frontend.db()) {
            dbg!(module.declarations(frontend.db()));
        }
    }
}
