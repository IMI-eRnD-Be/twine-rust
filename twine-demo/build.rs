use twine::build_translations;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_translations(&["./src/i18n/localization.ini"], "i18n.rs").unwrap();
}
