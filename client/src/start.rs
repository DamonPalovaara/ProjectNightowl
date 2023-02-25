use crate::{complex::ComplexGrapher, Application};
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

    let mut app = Application::new().await;
    let complex = ComplexGrapher::new(app.surface());
    app.add_render_object(Box::new(complex));
    app.run();
}
