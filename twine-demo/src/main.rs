use std::env;
use std::process;
//use substring::Substring;
use std::time::SystemTime;
use tracing::{error, trace, Level};
use tracing_subscriber::fmt;

// get the macro (t!) accessing the internationalization strings
include!(concat!(env!("OUT_DIR"), "/i18n.rs"));

pub mod services;

use crate::services::imports::twine_import;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize the tracing subsystem
    let span = tracing::span!(Level::TRACE, "wip_twine");
    let _enter = span.enter();
    let collector = fmt::Subscriber::builder()
        //.with_env_filter("trace")
        .with_max_level(tracing::Level::TRACE)
        .finish();

    // start tracing thread
    tracing::subscriber::with_default(collector, || {
        // get system environment
        //let mut lang = env::var("LANG").unwrap_or_else(|_| "en".to_string());
        //let lang_culture = lang.substring(3,4).to_string();   // "de_DE.UTF-8" -> "DE"
        //let lang_family = lang.substring(0,2).to_string();   // "de_DE.UTF-8" -> "de"
        //lang = format!{"Lang::{}(\"{}\")", &lang_culture, &lang_family};
        //println!{"lang: {}", &lang};

        // include localization strings
        let lang = Lang::De("de");
        let mut state = t!(state_started => lang);
        let mut res = t!(lang_code => lang);

        // localized feedback
        let time_start = SystemTime::now();
        trace!(target: "twine-demo", state = ?state,
               res = ? res, time = ?time_start);

        let mut import_path = concat!(env!("OUT_DIR"), "/i18n.rs").to_string();
        match twine_import::import(&mut import_path, &lang) {
            Ok(duration) => {
                trace!(target: "twine-demo",  import_path = ?import_path,
                       duration = ?&duration);
            }
            Err(err) => {
                error!("error running import: {}", err);
                process::exit(1);
            }
        };

        state = t!(state_finished => lang);
        res = t!(err_user_not_found => lang);

        // localized feedback
        let time_end = SystemTime::now();
        let duration = time_end.duration_since(time_start);
        trace!(target: "twine-demo", process = ?res, state = ?state, duration = ?duration);
    });

    Ok(())
}
