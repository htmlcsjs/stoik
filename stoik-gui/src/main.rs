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
