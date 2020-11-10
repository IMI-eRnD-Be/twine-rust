#[test]
fn ui() {
    assert!(std::process::Command::new("cargo")
        .args(&["build", "--manifest-path", "tests/test-crate/Cargo.toml"])
        .status()
        .unwrap()
        .success());
}
