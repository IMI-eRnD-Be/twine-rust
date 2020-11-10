const EXPECTED_OUTPUT: &str = "\
Ruiner le nom d'un groupe en le traduisant en français
Colère contre la machine
Hello, World !
73 %
0xBADCAFE
Ruin a band name by translating it in French
Rage Against the Machine
Hello, World!
73%
badcafe
Ruin a band name by translating it in French
Wrath Against the Machine
Hello, World!
% 73 % foo
badcafe";

#[test]
fn ui() {
    let output = std::process::Command::new("cargo")
        .args(&["run", "--manifest-path", "tests/test-crate/Cargo.toml"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("stdout:\n{}\n", stdout);
    eprintln!("stderr:\n{}\n", stderr);
    assert!(output.status.success());
    assert_eq!(stdout.trim(), EXPECTED_OUTPUT.trim());
}
