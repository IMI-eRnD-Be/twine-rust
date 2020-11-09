#[doc(hidden)]
pub use configparser;
#[doc(hidden)]
pub use heck;
#[doc(hidden)]
pub use regex;

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

        // NOTE: # cannot be used in the INI because it is considered as comment
        let re_printf = $crate::regex::Regex::new(
            r"%([-+#])?(\d+)?(\.\d+)?([dis@xXf])|.+"
        ).unwrap();
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
                let mut out = String::new();
                for caps in re_printf.captures_iter(text.as_str()) {
                    if let Some(type_) = caps.get(4) {
                        out.push_str("{:");
                        if let Some(flag) = caps.get(1) {
                            out.push_str(flag.as_str());
                        }
                        if let Some(width) = caps.get(2) {
                            out.push_str(width.as_str());
                        }
                        if let Some(precision) = caps.get(3) {
                            out.push_str(precision.as_str());
                        }
                        match type_.as_str() {
                            x @ "x" | x @ "X" => out.push_str(x),
                            _ => {},
                        }
                        out.push_str("}");
                    } else {
                        out.push_str(&caps[0]);
                    }
                }
                let lang = lang.to_camel_case();
                src.push_str(&format!(
                    "Lang::{} => format!({:?} $(, $fmt_args)*),\n",
                    lang,
                    out,
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
