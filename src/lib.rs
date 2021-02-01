//! ![Rust](https://github.com/IMI-eRnD-Be/twine/workflows/Rust/badge.svg)
//! [![Latest Version](https://img.shields.io/crates/v/twine.svg)](https://crates.io/crates/twine)
//! [![Docs.rs](https://docs.rs/twine/badge.svg)](https://docs.rs/twine)
//! [![LOC](https://tokei.rs/b1/github/IMI-eRnD-Be/twine)](https://github.com/IMI-eRnD-Be/twine)
//! [![Dependency Status](https://deps.rs/repo/github/IMI-eRnD-Be/twine/status.svg)](https://deps.rs/repo/github/IMI-eRnD-Be/twine)
//!
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
//!     twine::build_translations(&["translations.ini"], "i18n.rs");
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
//! [format_string]
//!     en = %s, %@!
//!     fr = %s, %@ !
//! [format_percentage]
//!     en = %.0f%
//!     fr = %.0f %
//! [format_hexadecimal]
//!     en = %x
//!     fr = %#X
//! ```
//!
//! Now in your project you can use the macro `t!` to translate anything:
//!
//! ```ignore
//! # enum Lang { Fr(&'static str) }
//! # macro_rules! t {
//! # ($($tokens:tt)+) => {{
//! # }};
//! # }
//! // you need to include the generated file somewhere
//! include!(concat!(env!("OUT_DIR"), "/i18n.rs"));
//!
//! fn main() {
//!     // use "" if there is no localization
//!     let lang = Lang::Fr("be");
//!
//!     // will output "Ruiner le nom d'un groupe en le traduisant en français"
//!     t!(app_ruin_the_band => lang);
//!
//!     // using formatted arguments, this will output "73 %"
//!     t!(format_percentage, 73.02f32 => lang);
//! }
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

use heck::{CamelCase, SnakeCase};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::Path;

#[allow(clippy::single_char_add_str)]
pub fn build_translations<P: AsRef<Path>, O: AsRef<Path>>(
    ini_files: &[P],
    output_file: O,
) -> io::Result<()> {
    // regex that tries to parse printf's format placeholders
    // see: https://docs.microsoft.com/en-us/cpp/c-runtime-library/format-specification-syntax-printf-and-wprintf-functions?view=msvc-160
    let re_printf = Regex::new(r"%([-+#])?(\d+)?(\.\d+)?([dis@xXf])|[^%]+|%%|%$").unwrap();
    let re_lang = Regex::new(r"(\w+)(-(\w+))?").unwrap();

    // turns all the keys into snake case automatically
    let normalize_key = |key: &str| key.to_snake_case().replace(".", "__");

    // generate match arms for each language (for a given translation key)
    let generate_match_arms = |translations: HashMap<String, String>,
                               all_languages: &mut HashSet<String>| {
        let mut match_arms = Vec::new();
        for (lang, text) in translations {
            // transform all printf's format placeholder to Rust's format
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
                        _ => {}
                    }
                    out.push_str("}");
                } else if &caps[0] == "%%" {
                    out.push_str("%");
                } else {
                    out.push_str(&caps[0]);
                }
            }

            // parse the language and region, then push the match arm
            let caps = re_lang.captures(lang.as_str()).expect("lang can be parsed");
            let lang = caps
                .get(1)
                .expect("the language is always there")
                .as_str()
                .to_camel_case();
            let region = caps.get(3).map(|x| format!("{:?}", x.as_str()));
            let no_region = "_".to_string();
            match_arms.push((
                format!(
                    "$crate::Lang::{}({}) => format!({:?} $(, $fmt_args)*),\n",
                    lang,
                    region.as_ref().unwrap_or(&no_region),
                    out,
                ),
                region.is_some(),
            ));
            all_languages.insert(lang);
        }
        match_arms.sort_unstable_by_key(|(_, has_region)| !has_region);

        match_arms
    };

    let mut map = HashMap::new();

    // read all the INI files (might override existing keys)
    for file_path in ini_files {
        let file_path = file_path.as_ref();
        match read_twine_ini(file_path) {
            Err(err) => panic!(
                "could not read Twine INI file `{}`: {}",
                file_path.display(),
                err
            ),
            Ok(other_map) => map.extend(other_map),
        }
        println!("cargo:rerun-if-changed={}", file_path.display());
    }

    let mut src = String::new();
    let mut all_languages = HashSet::new();
    src.push_str("#[macro_export]\nmacro_rules! t {\n");
    for (key, translations) in map {
        let key = normalize_key(key.as_str());
        src.push_str(&format!(
            "({} $(, $fmt_args:expr)* => $lang:expr) => {{\nmatch $lang {{\n",
            key,
        ));

        let match_arms = generate_match_arms(translations, &mut all_languages);

        src.extend(match_arms.iter().map(|(match_arm, _)| match_arm.as_str()));
        src.push_str("}};\n")
    }
    src.push_str("}\n");

    // generate the `Lang` enum and its variants
    src.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Hash)]
#[allow(dead_code)]
pub enum Lang {\n",
    );
    for lang in all_languages {
        src.push_str(&format!("{}(&'static str),\n", lang));
    }
    src.push_str("}\n");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(output_file);
    let _ = std::fs::create_dir_all(dest_path.parent().unwrap());
    std::fs::write(&dest_path, src)?;

    Ok(())
}

pub fn read_twine_ini<P: AsRef<Path>>(
    path: P,
) -> io::Result<HashMap<String, HashMap<String, String>>> {
    use std::io::BufRead;

    let re_section = regex::Regex::new(r"^\s*\[([^\]]+)\]").unwrap();
    let re_key_value = regex::Regex::new(r"^\s*([^\s=;#]+)\s*=\s*(.+?)\s*$").unwrap();

    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut section = map.entry("".to_owned()).or_default();

    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = re_section.captures(line.as_str()) {
            section = map
                .entry(caps.get(1).unwrap().as_str().to_owned())
                .or_default();
        }
        if let Some(caps) = re_key_value.captures(line.as_str()) {
            section.insert(
                caps.get(1).unwrap().as_str().to_owned(),
                caps.get(2).unwrap().as_str().to_owned(),
            );
        }
    }

    Ok(map)
}
