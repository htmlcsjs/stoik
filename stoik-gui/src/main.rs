#[allow(unused_imports)]
use app::{APP_NAME, APP_NAME_FORMATTED};

mod app;
mod util;
pub use app::StoikApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use eframe::{IconData, NativeOptions};

    let options = NativeOptions {
        icon_data: Some(
            IconData::try_from_png_bytes(include_bytes!("../assets/icon-512.png")).expect("sus"),
        ),

        ..Default::default()
    };

    env_logger::init();

    match eframe::run_native(
        APP_NAME_FORMATTED,
        options,
        Box::new(|cctx| Box::new(StoikApp::new(cctx))),
    ) {
        Ok(()) => (),
        Err(e) => log::error!("Error running {APP_NAME}: {e}"), // TODO, better error handling
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let handle = wasm::WebHandle::new();
    wasm_bindgen_futures::spawn_local(async move {
        match handle.start(APP_NAME).await {
            Ok(()) => (),
            Err(e) => log::error!("{e:?}"),
        }
    });
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use crate::StoikApp;
    use eframe::WebRunner;
    use wasm_bindgen::prelude::*;

    /// Your handle to the web app from JavaScript.
    #[derive(Clone)]
    #[wasm_bindgen]
    pub struct WebHandle {
        runner: WebRunner,
    }

    #[wasm_bindgen]
    impl WebHandle {
        /// Installs a panic hook, then returns.
        #[allow(clippy::new_without_default)]
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            // Redirect [`log`] message to `console.log` and friends:
            #[cfg(debug_assertions)]
            eframe::web::WebLogger::init(log::LevelFilter::Debug).ok();
            #[cfg(not(debug_assertions))]
            eframe::web::WebLogger::init(log::LevelFilter::Warn).ok();

            Self {
                runner: WebRunner::new(),
            }
        }

        /// Call this once from JavaScript to start your app.
        #[wasm_bindgen]
        pub async fn start(&self, canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
            self.runner
                .start(
                    canvas_id,
                    eframe::WebOptions::default(),
                    Box::new(|cc| Box::new(StoikApp::new(cc))),
                )
                .await
        }

        // The following are optional:

        #[wasm_bindgen]
        pub fn destroy(&self) {
            self.runner.destroy();
        }

        /// The JavaScript can check whether or not your app has crashed:
        #[wasm_bindgen]
        pub fn has_panicked(&self) -> bool {
            self.runner.has_panicked()
        }

        #[wasm_bindgen]
        pub fn panic_message(&self) -> Option<String> {
            self.runner.panic_summary().map(|s| s.message())
        }

        #[wasm_bindgen]
        pub fn panic_callstack(&self) -> Option<String> {
            self.runner.panic_summary().map(|s| s.callstack())
        }
    }
}
