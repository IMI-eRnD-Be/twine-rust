#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]

use std::env;
use std::time::SystemTime;
use tracing::{trace, Level};
use tracing_subscriber::fmt;

// get the macro (t!) accessing the internationalization strings
include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

pub mod services;

use crate::services::imports::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize the tracing subsystem
    let span = tracing::span!(Level::TRACE, "wip_twine");
    let _enter = span.enter();
    let collector = fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    // start tracing thread
    tracing::subscriber::with_default(collector, || {
        // include localization strings
        let lang = Lang::De("de");
        let mut state = t!(state_started => lang);
        let mut res = t!(lang_code => lang);

        // localized feedback
        let time_start = SystemTime::now();
        trace!(target: "twine-demo", state = ?state,
               res = ? res, time = ?time_start);

        match json_import::read_colours("data/demo.json") {
            Ok(colours) => {
                println!("We got {} colour elements", colours.colour.iter().count());
                for (pos, e) in colours.colour.iter().enumerate() {
                    println!("Element {}: colour name={:?}, rgba-value={:?}", pos, e.colour_name, e.code.rgba);
                }
            },
            Err(e) => {
                println!("Error: {}!", e);
                res = t!(err_import_colours => lang);
                println!("Error: {}!", res);
            },
        }

        state = t!(state_finished => lang);
        res = t!(import_colours => lang);

        // localized feedback
        let time_end = SystemTime::now();
        let duration = time_end.duration_since(time_start);
        trace!(target: "twine-demo", process = ?res, state = ?state, duration = ?duration);
    });

    Ok(())
}
