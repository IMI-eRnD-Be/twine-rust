//! Library for internationalization using the Twine file format.
//!
//! # Usage
//!
//! You first need to add twine to your `[build-dependencies]` in `Cargo.toml`.
//!
//! Create (or edit) your `build.rs` file:
//!
//! ```no_run
//! fn main() {
//!     println!("cargo:rerun-if-changed=build.rs");
//!     twine::build_translations!("translations.ini" => "i18n.rs");
//! }
//! ```
//!
//! You need an INI file with your translations. Example with `translations.ini`:
//!
//! ```text
//! [app_ruin_the_band]
//!     en = Ruin a band name by translating it in French
//!     fr = Ruiner le nom d'un groupe en le traduisant en français
//! [band_tool]
//!     en = Tool
//!     fr = Outil
//! [band_the_doors]
//!     en = The Doors
//!     fr = Les portes
//! [band_rage_against_the_machine]
//!     en = Rage Against the Machine
//!     en-gb = Wrath Against the Machine
//!     fr = Colère contre la machine
//! [band_the_jackson_5]
//!     en = The Jackson 5
//!     fr = Les 5 fils de Jack
//! [format_percentage]
//!     en = %.0f%
//!     fr = %.0f %
//! ```
//!
//! Now in your project you can use the macro `t!` to translate anything:
//!
//! ```no_run
//! # enum Lang { Fr(&'static str) }
//! # macro_rules! t {
//! # ($($tokens:tt)+) => {{
//! # }};
//! # }
//! // use "" if there is no localization
//! let lang = Lang::Fr("be");
//!
//! // will output "Ruiner le nom d'un groupe en le traduisant en français"
//! t!(app_ruin_the_band => lang);
//!
//! // using formatted arguments, this will output "73 %"
//! t!(format_percentage, 73.02f32 => lang)
//! ```
//!
//! # Implementation Notes
//!
//! All translation keys must have all the languages of all the keys. For example, if all your keys
//! have translations for `en` and `fr`, if one key has only `en`, it will fail to compile.
//!
//! Localized translation can be provided and will be used if available. Otherwise it will
//! fallback to the default translation for that language.
//!
//! Any typo in the key will make the compilation fail. Missing format arguments will also make
//! the compilation fail.
//!
//! # License
//!
//! This work is dual-licensed under Apache 2.0 and MIT.
//! You can choose between one of them if you use this work.

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
        let re_lang = $crate::regex::Regex::new(r"(\w+)(-(\w+))?").unwrap();
        let mut src = String::new();
        let mut all_languages = HashSet::new();
        src.push_str("macro_rules! t {\n");
        for (key, translations) in map {
            let key = key.to_snake_case().replace(".", "__");
            src.push_str(&format!(
                "({} $(, $fmt_args:expr)* => $lang:expr) => {{\nmatch $lang {{\n", key,
            ));
            let mut match_arms = Vec::new();
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
                let caps = re_lang.captures(lang.as_str()).expect("lang can be parsed");
                let lang = caps
                    .get(1)
                    .expect("the language is always there")
                    .as_str()
                    .to_camel_case();
                let region = caps
                    .get(3)
                    .map(|x| format!("{:?}", x.as_str()));
                let no_region = "_".to_string();
                match_arms.push((format!(
                    "Lang::{}({}) => format!({:?} $(, $fmt_args)*),\n",
                    lang,
                    region.as_ref().unwrap_or(&no_region),
                    out,
                ), region.is_some()));
                all_languages.insert(lang);
            }
            match_arms.sort_unstable_by_key(|(_, has_region)| !has_region);
            src.extend(match_arms.iter().map(|(match_arm, _)| match_arm.as_str()));
            src.push_str("}};\n")
        }
        src.push_str("}
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
#[allow(dead_code)]
enum Lang {\n");
        for lang in all_languages {
            src.push_str(&format!("{}(&'static str),\n", lang));
        }
        src.push_str("}\n");

        let out_dir = std::env::var_os("OUT_DIR").unwrap();
        let dest_path = Path::new(&out_dir).join($output_file);
        let _ = std::fs::create_dir_all(dest_path.parent().unwrap());
        std::fs::write(&dest_path, src).unwrap();
    }};
}
