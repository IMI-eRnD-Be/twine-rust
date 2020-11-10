include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

fn main() {
    for lang in vec![Lang::Fr(""), Lang::En(""), Lang::En("gb")] {
        println!("{}", t!(app_ruin_the_band => lang));
        println!("{}", t!(band_rage_against_the_machine => lang));
        println!("{}", t!(format_string, "Hello", "World" => lang));
        println!("{}", t!(format_percentage, 73.02f32 => lang));
        println!("{}", t!(format_hexadecimal, 0xBAD_CAFE => lang));
    }
}
