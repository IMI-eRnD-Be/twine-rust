#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]

mod my_module;

include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

fn main() {
    my_module::my_fn();
    my_module::test_serde();
}
