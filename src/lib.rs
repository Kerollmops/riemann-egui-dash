#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod event;
mod views;

pub use app::RiemannDashApp;
#[cfg(target_arch = "wasm32")] // When compiling for web
use eframe::wasm_bindgen::{self, prelude::*};
use url::Url;

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> std::result::Result<(), eframe::wasm_bindgen::JsValue> {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let app = RiemannDashApp::default();
    eframe::start_web(canvas_id, Box::new(app))
}

pub fn websocket_url(url: &Url, subscribe: bool, query: &str) -> Url {
    use url::form_urlencoded::Serializer;

    let query = Serializer::new(String::new())
        .append_pair("subscribe", &subscribe.to_string())
        .append_pair("query", query)
        .finish();
    let mut url = url.clone().join("index/").unwrap();
    url.set_query(Some(&query));
    url
}

pub fn base_url(mut url: Url) -> Result<Url, url::ParseError> {
    match url.path_segments_mut() {
        Ok(mut path) => path.clear(),
        Err(_) => return Err(url::ParseError::RelativeUrlWithCannotBeABaseBase),
    };
    url.set_query(None);
    Ok(url)
}
