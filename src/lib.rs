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
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

type TwineData = HashMap<String, HashMap<String, String>>;

/// Generate the `t!()` macro based on the provided list of paths to Twine INI translation files.
pub fn build_translations<P: AsRef<Path>, O: AsRef<Path>>(
    ini_files: &[P],
    output_file: O,
) -> io::Result<()> {
    let mut readers = ini_files
        .iter()
        .map(|file_path| {
            let file_path = file_path.as_ref();
            println!("cargo:rerun-if-changed={}", file_path.display());
            fs::File::open(&file_path)
        })
        .collect::<io::Result<Vec<_>>>()?;

    build_translations_from_readers(readers.as_mut_slice(), output_file)
}

/// Generate the `t!()` macro based on the provided list of `&str` containing Twine INI
/// translations.
pub fn build_translations_from_str<P: AsRef<Path>>(
    strs: &[&str],
    output_file: P,
) -> io::Result<()> {
    let mut readers = strs.iter().map(|s| io::Cursor::new(s)).collect::<Vec<_>>();

    build_translations_from_readers(readers.as_mut_slice(), output_file)
}

/// Generate the `t!()` macro based on the provided list of readers containing Twine INI
/// translations.
pub fn build_translations_from_readers<R: Read, P: AsRef<Path>>(
    readers: &mut [R],
    output_file: P,
) -> io::Result<()> {
    let mut map = HashMap::new();

    // read all the INI files (might override existing keys)
    for reader in readers {
        match read_twine_ini(reader) {
            Err(err) => panic!("could not read Twine INI file: {}", err),
            Ok(other_map) => map.extend(other_map),
        }
    }

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(output_file);
    let _ = fs::create_dir_all(dest_path.parent().unwrap());
    let mut f = io::BufWriter::new(
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(dest_path)?,
    );
    write!(f, "{}", TwineFormatter { map })?;

    Ok(())
}

fn read_twine_ini<R: Read>(reader: &mut R) -> io::Result<TwineData> {
    use std::io::BufRead;

    let re_section = regex::Regex::new(r"^\s*\[([^\]]+)\]").unwrap();
    let re_key_value = regex::Regex::new(r"^\s*([^\s=;#]+)\s*=\s*(.+?)\s*$").unwrap();

    let mut map: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut section = map.entry("".to_owned()).or_default();

    let reader = io::BufReader::new(reader);
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

struct TwineFormatter {
    map: TwineData,
}

impl fmt::Display for TwineFormatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = indenter::CodeFormatter::new(f, "    ");
        let mut all_languages = HashSet::new();

        write!(
            f,
            r#"
            #[macro_export]
            macro_rules! t {{
            "#,
        )?;
        f.indent(1);

        for (key, translations) in self.map.iter() {
            let key = Self::normalize_key(key.as_str());
            write!(
                f,
                r#"
                ({} $(, $fmt_args:expr)* => $lang:expr) => {{
                    match $lang {{
                "#,
                key,
            )?;
            f.indent(2);

            Self::generate_match_arms(&mut f, translations, &mut all_languages)?;

            f.dedent(2);
            write!(
                f,
                r#"
                }}}};
                "#,
            )?;
        }
        f.dedent(1);

        write!(
            f,
            r#"
            }}
            "#,
        )?;

        // generate the `Lang` enum and its variants
        write!(
            f,
            r#"
            #[derive(Debug, Clone, Copy, PartialEq, Hash)]
            #[allow(dead_code)]
            pub enum Lang {{
            "#,
        )?;
        f.indent(1);

        for lang in all_languages {
            write!(
                f,
                r#"
                {}(&'static str),
                "#,
                lang,
            )?;
        }

        f.dedent(1);
        write!(
            f,
            r#"
            }}
            "#,
        )?;

        Ok(())
    }
}

impl TwineFormatter {
    fn generate_match_arms<W: fmt::Write>(
        f: &mut W,
        translations: &HashMap<String, String>,
        all_languages: &mut HashSet<String>,
    ) -> fmt::Result {
        // regex that tries to parse printf's format placeholders
        // see: https://docs.microsoft.com/en-us/cpp/c-runtime-library/format-specification-syntax-printf-and-wprintf-functions?view=msvc-160
        let re_printf = Regex::new(r"%([-+#])?(\d+)?(\.\d+)?([dis@xXf])|[^%]+|%%|%$").unwrap();
        let re_lang = Regex::new(r"(\w+)(-(\w+))?").unwrap();

        let mut match_arms = Vec::new();
        let mut default_out = None;
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

            if default_out.is_none() {
                default_out = Some(out.clone());
            }

            // parse the language and region, then push the match arm
            let caps = re_lang.captures(lang.as_str()).expect("lang can be parsed");
            let lang = caps
                .get(1)
                .expect("the language is always there")
                .as_str()
                .to_camel_case();
            let region = caps.get(3).map(|x| format!("{:?}", x.as_str()));
            match_arms.push((lang.clone(), region, out));
            all_languages.insert(lang);
        }
        match_arms.sort_unstable_by_key(|(_, region, _)| region.is_none());

        for (lang, region, format) in match_arms {
            write!(
                f,
                r#"
                $crate::Lang::{}({}) => format!({:?} $(, $fmt_args)*),
                "#,
                lang,
                region.as_ref().map(|x| x.as_str()).unwrap_or("_"),
                format,
            )?;
        }

        if let Some(default_out) = default_out {
            write!(
                f,
                r#"
                _ => format!({:?} $(, $fmt_args)*),
                "#,
                default_out,
            )?;
        }

        Ok(())
    }

    // turns all the keys into snake case automatically
    fn normalize_key(key: &str) -> String {
        key.to_snake_case().replace(".", "__")
    }
}
