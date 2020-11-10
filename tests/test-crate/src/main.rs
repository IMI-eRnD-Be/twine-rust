include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

fn main() {
    let lang = Lang::Fr("be");

    t!(app_ruin_the_band => lang);
    t!(format_percentage, 73.02f32 => lang);
}
