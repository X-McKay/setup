//! Component contract test: for the subset of components that are both
//! docker-testable and reversible, assert install -> is_installed true,
//! uninstall -> is_installed false. Runs only when SETUP_CONTRACT_TESTS=1.

#[test]
fn contract_install_uninstall_roundtrip() {
    if std::env::var("SETUP_CONTRACT_TESTS").ok().as_deref() != Some("1") {
        eprintln!("SETUP_CONTRACT_TESTS != 1 - skipping.");
        return;
    }

    use setup::components::registry::Registry;
    use setup::manifest::loader;

    let manifest = loader::load().expect("manifest");
    let registry = Registry::build();

    for spec in &manifest.components {
        if !spec.docker_testable() {
            eprintln!("[skip] {} (not docker_testable)", spec.id);
            continue;
        }
        let c = registry.get(&spec.id).expect("registered");
        if !c.is_reversible() {
            eprintln!("[skip] {} (not reversible)", spec.id);
            continue;
        }

        c.install()
            .unwrap_or_else(|e| panic!("{}: install failed: {}", spec.id, e));
        assert!(
            c.is_installed().unwrap_or(false),
            "{}: is_installed false after install",
            spec.id
        );
        c.uninstall()
            .unwrap_or_else(|e| panic!("{}: uninstall failed: {}", spec.id, e));
        assert!(
            !c.is_installed().unwrap_or(true),
            "{}: still installed after uninstall",
            spec.id
        );
    }
}
