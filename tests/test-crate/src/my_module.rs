use crate::t;

pub fn my_fn() {
    for lang in vec![crate::Lang::Fr(""), crate::Lang::En(""), crate::Lang::En("gb")] {
        println!("{}", t!(app_ruin_the_band => lang));
        println!("{}", t!(band_rage_against_the_machine => lang));
        println!("{}", t!(format_string, "Hello", "World" => lang));
        println!("{}", t!(format_percentage, 73.02f32 => lang));
        println!("{}", t!(format_hexadecimal, 0xBAD_CAFE => lang));
        println!("{}", t!(fallback_to_default_lang => lang));
    }
}
