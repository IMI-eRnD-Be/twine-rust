# twine

![Rust](https://github.com/IMI-eRnD-Be/twine/workflows/Rust/badge.svg)
[![Latest Version](https://img.shields.io/crates/v/twine.svg)](https://crates.io/crates/twine)
[![Docs.rs](https://docs.rs/twine/badge.svg)](https://docs.rs/twine)
[![LOC](https://tokei.rs/b1/github/IMI-eRnD-Be/twine)](https://github.com/IMI-eRnD-Be/twine)
[![Dependency Status](https://deps.rs/repo/github/IMI-eRnD-Be/twine/status.svg)](https://deps.rs/repo/github/IMI-eRnD-Be/twine)

Library for internationalization using the [Twine](https://github.com/scelis/twine) file
format.

## Usage

1.  You need to add twine to your `[build-dependencies]` in `Cargo.toml`.

2.  Create (or edit) your `build.rs` file:

```rust
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    twine::build_translations(&["translations.ini"], "i18n.rs");
}
```

3.  You need an INI file with your translations. Example with `translations.ini`:

```
[app_ruin_the_band]
    en = Ruin a band name by translating it in French
    fr = Ruiner le nom d'un groupe en le traduisant en français
[band_tool]
    en = Tool
    fr = Outil
[band_the_doors]
    en = The Doors
    fr = Les portes
[band_rage_against_the_machine]
    en = Rage Against the Machine
    en-gb = Wrath Against the Machine
    fr = Colère contre la machine
[band_the_jackson_5]
    en = The Jackson 5
    fr = Les 5 fils de Jack
[format_string]
    en = %s, %@!
    fr = %s, %@ !
[format_percentage]
    en = %.0f%
    fr = %.0f %
[format_hexadecimal]
    en = %x
    fr = %#X
```

4.  Now in your project you can use the macro `t!` to translate anything:

```rust
// you need to include the generated file somewhere
include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

fn main() {
    // use "" if there is no localization
    let lang = Lang::Fr("be");

    // will output "Ruiner le nom d'un groupe en le traduisant en français"
    t!(app_ruin_the_band => lang);

    // using formatted arguments, this will output "73 %"
    t!(format_percentage, 73.02f32 => lang);
}
```

## Implementation Notes

All translation keys must have all the languages of all the keys. For example, if all your keys
have translations for `en` and `fr`, if one key has only `en`, it will fail to compile.

Localized translation can be provided and will be used if available. Otherwise it will
fallback to the default translation for that language.

Any typo in the key will make the compilation fail. Missing format arguments will also make
the compilation fail.

## Features

 *  `serde`: when this feature is activated you will need to add `serde` to your dependencies
    and the `Lang` enum generated implements `Serialize` and `Deserialize`.

## License

This work is dual-licensed under Apache 2.0 and MIT.
You can choose between one of them if you use this work.

License: MIT OR Apache-2.0
