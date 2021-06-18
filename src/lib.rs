#![allow(clippy::needless_doctest_main)]
//! ![Rust](https://github.com/IMI-eRnD-Be/twine/workflows/Rust/badge.svg)
//! [![Latest Version](https://img.shields.io/crates/v/twine.svg)](https://crates.io/crates/twine)
//! [![Docs.rs](https://docs.rs/twine/badge.svg)](https://docs.rs/twine)
//! [![LOC](https://tokei.rs/b1/github/IMI-eRnD-Be/twine)](https://github.com/IMI-eRnD-Be/twine)
//! [![Dependency Status](https://deps.rs/repo/github/IMI-eRnD-Be/twine/status.svg)](https://deps.rs/repo/github/IMI-eRnD-Be/twine)
//!
//! Library for internationalization using the [Twine](https://github.com/scelis/twine) file
//! format.
//!
//! # Usage
//!
//! 1.  You need to add twine to your `[build-dependencies]` in `Cargo.toml`.
//!
//! 2.  Create (or edit) your `build.rs` file:
//!
//! ```no_run
//! fn main() {
//!     println!("cargo:rerun-if-changed=build.rs");
//!     twine::build_translations(&["translations.ini"], "i18n.rs");
//! }
//! ```
//!
//! 3.  You need an INI file with your translations.
//!     Language translations are matched by `two lowercase letter` code (eg: `en`).
//!     Localized language translations are identified by `two lowercase letter` code,
//!     plus `hyphen`, plus `to letter localization` code (eg: `en-gb`).
//!
//!     The next paragraph is an example `translations.ini` file:
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
//! 4.  Now in your project you can use the macro `t!` to translate anything:
//!
//! ```ignore
//! # /// define valid language varients
//! # enum Lang { Fr(&'static str) }
//! # /// desugar the procedural macro call
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
//! # Features
//!
//!  *  `serde`: when this feature is activated you will need to add `serde` to your dependencies
//!     and the `Lang` enum generated implements `Serialize` and `Deserialize`.
//!
//! # License
//!
//! This work is dual-licensed under Apache 2.0 and MIT.
//! You can choose between one of them if you use this work.

use heck::{CamelCase, SnakeCase};
use indenter::CodeFormatter;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::Path;

// regex that tries to parse printf's format placeholders
// see: https://docs.microsoft.com/en-us/cpp/c-runtime-library/format-specification-syntax-printf-and-wprintf-functions?view=msvc-160
static RE_PRINTF: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"%([-+#])?(\d+)?(\.\d+)?([dis@xXf])|[^%]+|%%|%$").unwrap());
static RE_LANG: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\w+)(-(\w+))?").unwrap());
static RE_SECTION: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*\[([^\]]+)\]").unwrap());
static RE_KEY_VALUE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*([^\s=;#]+)\s*=\s*(.+?)\s*$").unwrap());

type TwineData = HashMap<String, Vec<(String, String)>>;

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
    let mut readers = strs.iter().map(io::Cursor::new).collect::<Vec<_>>();

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

    let mut map: TwineData = HashMap::new();
    let mut section = None;

    let reader = io::BufReader::new(reader);
    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if let Some(caps) = RE_SECTION.captures(line.as_str()) {
            section = Some(
                map.entry(caps.get(1).unwrap().as_str().to_owned())
                    .or_default(),
            );
        }
        if let Some(caps) = RE_KEY_VALUE.captures(line.as_str()) {
            if let Some(section) = section.as_mut() {
                section.push((
                    caps.get(1).unwrap().as_str().to_owned(),
                    caps.get(2).unwrap().as_str().to_owned(),
                ));
            } else {
                panic!("key-value outside section at line {}", i + 1);
            }
        }
    }

    Ok(map)
}

struct TwineFormatter {
    map: TwineData,
}

impl fmt::Display for TwineFormatter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = CodeFormatter::new(f, "    ");
        let mut all_languages = HashSet::new();

        write!(
            f,
            r#"

            "#,
        )?;

        write!(
            f,
            r#"
            // i18n.rs

            /// Create translation strings for supported language varients.
            #[macro_export]
            macro_rules! t {{
            "#,
        )?;
        f.indent(1);

        let mut sorted: Vec<_> = self.map.iter().collect();
        sorted.sort_unstable_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

        for (key, translations) in sorted {
            let key = Self::normalize_key(key.as_str());
            write!(
                f,
                r#"
                ({} $(, $fmt_args:expr)* => $lang:expr) => {{{{
                    match $lang {{
                "#,
                key,
            )?;
            f.indent(2);

            self.generate_match_arms(&mut f, translations, &mut all_languages)?;

            f.dedent(2);
            write!(
                f,
                r#"
                    }}
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

            /// Valid language variants.
            #[derive(Clone, Copy, Hash, Debug, PartialEq)]
            #[allow(dead_code)]
            pub enum Lang {{
            "#,
        )?;
        f.indent(1);

        let lang_variants: HashSet<_> = all_languages
            .iter()
            .map(|(lang, _)| lang.as_str())
            .collect();
        let mut lang_variants: Vec<_> = lang_variants.into_iter().collect();
        lang_variants.sort_unstable();

        for lang in lang_variants.iter() {
            write!(
                f,
                r#"
                /// variant {}
                {}(&'static str),
                "#,
                lang, lang
            )?;
        }

        f.dedent(1);
        write!(
            f,
            r#"
            }}

            impl Lang {{
                /// Array with known language identifier.
                pub fn all_languages() -> &'static [&'static Lang] {{
                    &[
            "#,
        )?;
        f.indent(3);

        let mut sorted_languages: Vec<_> = all_languages.iter().collect();
        sorted_languages.sort_unstable();

        for (lang, region) in sorted_languages {
            write!(
                f,
                r#"
                &Lang::{}({:?}),
                "#,
                lang,
                region.as_deref().unwrap_or(""),
            )?;
        }

        f.dedent(3);
        write!(
            f,
            r#"
                    ]
                }}
            }}
            "#,
        )?;

        // implent default for `Lang`
        // the fist in the sorted list should be fine
        write!(
            f,
            r#"

            impl Default for Lang {{
            "#,
        )?;
        f.indent(1);

        let mut sorted_languages: Vec<_> = all_languages.iter().collect();
        sorted_languages.sort_unstable();

        let (default_lang, default_region) = sorted_languages[0];
        write!(
            f,
            r#"
            fn default() -> Self {{ Lang::{}({:?}) }}
            "#,
            default_lang,
            default_region.as_deref().unwrap_or(""),
        )?;
        f.dedent(1);

        write!(
            f,
            r#"
            }}
            "#,
        )?;

        #[cfg(feature = "serde")]
        {
            let mut all_regions: Vec<_> = all_languages
                .iter()
                .filter_map(|(_, region)| region.as_deref())
                .collect();
            all_regions.sort_unstable_by(|a, b| a.cmp(b).reverse());
            Self::generate_serde(&mut f, &lang_variants, &all_regions)?;
        }

        Ok(())
    }
}

impl TwineFormatter {
    #[allow(clippy::single_char_add_str)]
    fn generate_match_arms(
        &self,
        f: &mut CodeFormatter<fmt::Formatter>,
        translations: &[(String, String)],
        all_languages: &mut HashSet<(String, Option<String>)>,
    ) -> fmt::Result {
        let mut match_arms = Vec::new();
        let mut default_out = None;
        for (lang, text) in translations {
            // transform all printf's format placeholder to Rust's format
            let mut out = String::new();
            for caps in RE_PRINTF.captures_iter(text.as_str()) {
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
            let caps = RE_LANG.captures(lang.as_str()).expect("lang can be parsed");
            let lang = caps
                .get(1)
                .expect("the language is always there")
                .as_str()
                .to_camel_case();
            let region = caps.get(3);
            all_languages.insert((lang.clone(), region.map(|x| x.as_str().to_string())));
            match_arms.push((lang, region.map(|x| format!("{:?}", x.as_str())), out));
        }
        match_arms.sort_unstable_by(|(a_lang, a_region, _), (b_lang, b_region, _)| {
            a_lang
                .cmp(b_lang)
                .then(a_region.is_none().cmp(&b_region.is_none()))
        });

        for (lang, region, format) in match_arms {
            write!(
                f,
                r#"
                $crate::Lang::{}({}) => format!({:?} $(, $fmt_args)*),
                "#,
                lang,
                region.as_deref().unwrap_or("_"),
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

    #[cfg(feature = "serde")]
    fn generate_serde(
        f: &mut CodeFormatter<fmt::Formatter>,
        all_languages: &[&str],
        all_regions: &[&str],
    ) -> fmt::Result {
        write!(
            f,
            r#"

            impl<'de> serde::Deserialize<'de> for Lang {{
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {{
                    use serde::de;
                    use std::fmt;

                    struct LangVisitor;

                    impl<'de> de::Visitor<'de> for LangVisitor {{
                        type Value = Lang;

                        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {{
                            formatter.write_str("expected string")
                        }}

                        fn visit_str<E>(self, value: &str) -> Result<Lang, E>
                        where
                            E: de::Error,
                        {{
                            let mut it = value.splitn(2, '_');
                            let lang = it.next().unwrap();
                            let region = it.next().unwrap_or("");

                            let region = match region.to_lowercase().as_str() {{
            "#,
        )?;
        f.indent(5);

        for region in all_regions {
            write!(
                f,
                r#"
                {region:?} => {region:?},
                "#,
                region = region,
            )?;
        }

        f.dedent(1);
        write!(
            f,
            r#"
                "" => "",
                _ => {{
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Str(region),
                        &"existing region",
                    ));
                }}
            }};

            match lang {{
            "#,
        )?;
        f.indent(1);

        for lang in all_languages {
            write!(
                f,
                r#"
                {:?} => Ok(Lang::{}(region)),
                "#,
                lang.to_snake_case(),
                lang,
            )?;
        }

        f.dedent(5);
        write!(
            f,
            r#"
                                _ => {{
                                    return Err(de::Error::invalid_value(
                                        de::Unexpected::Str(region),
                                        &"existing language",
                                    ));
                                }}
                            }}
                        }}
                    }}

                    deserializer.deserialize_str(LangVisitor)
                }}
            }}

            impl serde::Serialize for Lang {{
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::ser::Serializer,
                {{
                    match self {{
            "#,
        )?;
        f.indent(3);

        for lang in all_languages {
            write!(
                f,
                r#"
                Lang::{variant}(region) if region.is_empty() => serializer.serialize_str({lang:?}),
                Lang::{variant}(region) => serializer.serialize_str(
                    &format!("{{}}_{{}}", {lang:?}, region),
                ),
                "#,
                variant = lang,
                lang = lang.to_snake_case(),
            )?;
        }

        f.dedent(3);
        write!(
            f,
            r#"
                    }}
                }}
            }}
            "#,
        )?;

        Ok(())
    }
}
