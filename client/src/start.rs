// Since this project gets loaded into the browser as a dynamic library we need our entry point
// to be part of the library rather than in the main.rs file

use crate::complex::ComplexGrapher;
use crate::engine::Engine;
use tracing::info;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // Initialize the logger
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        }
        else {
            tracing_subscriber::fmt::init();
        }
    }

    info!("Starting up program");

    let (mut engine, event_loop) = Engine::new().await;
    let complex = ComplexGrapher::_new(&engine);
    engine.add_engine_object(Box::new(complex));
    engine.run(event_loop);
}
