#[doc(hidden)]
pub use configparser;
#[doc(hidden)]
pub use heck;

#[macro_export]
macro_rules! build_translations {
    ($($ini_files:expr),+ => $output_file:expr) => {{
        use ::std::collections::{HashMap, HashSet};
        use ::std::path::Path;
        use $crate::heck::{CamelCase, SnakeCase};

        let mut map = HashMap::new();

        $(
        let mut config = $crate::configparser::ini::Ini::new();
        match config.load($ini_files) {
            Err(err) => panic!("{}", err),
            Ok(other_map) => map.extend(other_map),
        }
        println!("cargo:rerun-if-changed={}", $ini_files);
        )+

        let mut src = String::new();
        let mut all_languages = HashSet::new();
        src.push_str("macro_rules! t {\n");
        for (key, translations) in map {
            let key = key.to_snake_case().replace(".", "__");
            src.push_str(&format!(
                "({} $(, $fmt_args:expr)* => $lang:expr) => {{\nmatch $lang {{\n", key,
            ));
            for (lang, text) in translations {
                let text = text.expect("all values are provided");
                let lang = lang.to_camel_case();
                src.push_str(&format!(
                    "Lang::{} => format!({:?} $(, $fmt_args)*),\n",
                    lang,
                    text,
                ));
                all_languages.insert(lang);
            }
            src.push_str("}};\n")
        }
        src.push_str("}
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
#[allow(dead_code)]
enum Lang {\n");
        for lang in all_languages {
            src.push_str(&format!("{},\n", lang));
        }
        src.push_str("}\n");

        let out_dir = std::env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join($output_file);
        let _ = std::fs::create_dir_all(dest_path.parent().unwrap());
        std::fs::write(&dest_path, src).unwrap();
    }};
}
