use crate::t;
use crate::Lang;

pub fn basic() {
    for lang in &[Lang::Fr(""), Lang::En(""), Lang::En("gb")] {
        println!("{}", t!(app_ruin_the_band => lang));
        println!("{}", t!(band_rage_against_the_machine => lang));
        println!("{}", t!(format_string, "Hello", "World" => lang));
        println!("{}", t!(format_percentage, 73.02f32 => lang));
        println!("{}", t!(format_hexadecimal, 0xBAD_CAFE => lang));
        println!("{}", t!(fallback_to_default_lang => lang));
        println!("{}", t!(name_with_dot_in_it => lang));
        println!("{}", t!(stuff_with_escaped_sequences_and_double_quotes => lang));
    }

    assert_eq!(
        Lang::all_languages(),
        &[&Lang::En(""), &Lang::En("gb"), &Lang::Fr("")]
    );

    let lang = Lang::En("gb");
    assert_eq!(lang.to_string(), "en_gb");
    assert_eq!(lang.language(), "en");
    assert_eq!(lang.region(), "gb");
    let lang = Lang::En("");
    assert_eq!(lang.to_string(), "en");
    assert_eq!(lang.language(), "en");
    assert_eq!(lang.region(), "");
}

pub fn serde() {
    let lang: Lang = serde_json::from_str("\"en_GB\"").unwrap();
    assert_eq!(serde_json::to_string(&lang).unwrap(), "\"en_gb\"");
    let lang: Lang = serde_json::from_str("\"en\"").unwrap();
    assert_eq!(serde_json::to_string(&lang).unwrap(), "\"en\"");
    let lang: Lang = serde_json::from_str("\"fr\"").unwrap();
    assert_eq!(serde_json::to_string(&lang).unwrap(), "\"fr\"");
}
