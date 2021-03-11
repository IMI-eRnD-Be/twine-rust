use std::fs;

#[test]
fn deterministic() {
    let translations = r#"
        [band_tool]
            en-us = Tool (US)
            en-gb = Tool (GB)
            fr = Outil
        [band_the_doors]
            en = The Doors
            fr = Les portes
        "#;
    let out_dir = tempfile::tempdir().unwrap();
    std::env::set_var("OUT_DIR", out_dir.as_ref().as_os_str());

    // The dataset is very small so it has chances to passes by luck
    for _ in 0..100 {
        twine::build_translations_from_str(&[translations], "a.rs").unwrap();
        let a = fs::read_to_string(out_dir.as_ref().join("a.rs")).unwrap();
        twine::build_translations_from_str(&[translations], "b.rs").unwrap();
        let b = fs::read_to_string(out_dir.as_ref().join("b.rs")).unwrap();
        println!("{}\n========\n{}", a, b);
        assert!(
            a == b,
            "multiple iterations of generating translations should always output the same",
        );
    }
}
