#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) type DateTime = i64;
const SECS_1904_TO_1970: u64 = 2082844800;

/// Jan 1, 1904
fn font_epoch() -> SystemTime {
    UNIX_EPOCH - Duration::from_secs(SECS_1904_TO_1970)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now() -> DateTime {
    SystemTime::now()
        .duration_since(font_epoch())
        .expect("Time went backwards")
        .as_secs() as i64
    // this u64 -> i64 conversion should be safe
    // until the year 292279027113
}


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen] extern "C" {
    #[wasm_bindgen(js_namespace = Date, js_name = now)]
    fn date_now() -> f64;
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn now() -> DateTime {
    let timestamp_millis = date_now();
    let timestamp_secs = (timestamp_millis / 1000.0) as u64;
    (timestamp_secs + SECS_1904_TO_1970) as i64
}
