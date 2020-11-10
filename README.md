# twine

<!-- cargo-sync-readme start -->

Library for internationalization using the Twine file format.

## Usage

1.  You first need to add twine to your `[build-dependencies]` in `Cargo.toml`.

    Create (or edit) your `build.rs` file:

    ```rust
    fn main() {
        println!("cargo:rerun-if-changed=build.rs");
        twine::build_translations!("translations.ini" => "i18n.rs");
    }
    ```rust

2.  You need an INI file with your translations. Example with `translations.ini`:

    ```text
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
    [format_percentage]
        en = %.0f%
        fr = %.0f %
    ```rust

    ### Implementation Notes

    All translation keys must have all the languages of all the keys. For example, if all your
    keys have translations for `en` and `fr`, if one key has only `en`, it will fail to
    compile.

    Localized translation can be provided and will be used if available. Otherwise it will
    fallback to the default translation for that language.

3.  Now in your project you can use the macro `t!` to translate anything:

    ```rust
    // use "" if there is no localization
    let lang = Lang::Fr("be");

    // will output "Ruiner le nom d'un groupe en le traduisant en français"
    t!(app_ruin_the_band => lang);

    // using formatted arguments, this will output "73 %"
    t!(format_percentage, 73.02f32 => lang)
    ```rust

    ### Implementation Notes

    Any typo in the key will make the compilation fail. Missing format arguments will also make
    the compilation fail.

<!-- cargo-sync-readme end -->

## License

This work is dual-licensed under Apache 2.0 and MIT.
You can choose between one of them if you use this work.

`SPDX-License-Identifier: Apache-2.0 OR MIT`
