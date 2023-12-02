#[cfg(not(target_arch = "wasm32"))]
use std::io::Write;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Replaces the site body with a message telling the user to open the console and use that.
#[cfg(target_arch = "wasm32")]
pub fn add_web_nothing_to_see_msg() {
    web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.body())
        .expect("Could not get document / body.")
        .set_inner_html("<h1>Nothing to see here! Open the console!</h1>");
}