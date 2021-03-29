use std::{
    error::Error,
    process::Command,
    time::{Duration, SystemTime},
};
use tracing::trace;

use crate::Lang;

/// test import
pub fn import(p: &mut String, lang: &Lang) -> Result<Duration, Box<dyn Error>> {
    use std::path::Path;

    let mut res = t!(import_started => lang);
    let mut state = t!(state_started => lang);

    let time_start = SystemTime::now();
    trace!(target: "import", process = ?res, state = ?state,
           date_start = ?time_start);

    let path = Path::new(p);
    assert_eq!(path.is_file(), true);
    trace!(target: "import", path = ?path);

    let mut child = Command::new("sleep").arg("5").spawn().unwrap();
    let _result = child.wait().unwrap();

    let mut time_end = SystemTime::now();
    let duration = time_end
        .duration_since(time_start)
        .expect("Clock may have gone backwards");
    trace!(target: "import", duration = ?duration);

    state = t!(state_finished => lang);
    res = t!(import_finished => lang);

    time_end = SystemTime::now();
    trace!(target: "import", process = ?res, state = ?state,
           date_stop = ?time_end);

    Ok(duration)
}
