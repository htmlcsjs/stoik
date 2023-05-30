#[allow(unused_imports)]
use app::{APP_NAME, APP_NAME_FORMATTED};

include! {"lib.rs"}

#[cfg(not(target_arch = "wasm32"))]
include! {concat!(env!("OUT_DIR"), "/icon.rs")}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use eframe::{IconData, NativeOptions};

    let options = NativeOptions {
        icon_data: Some(IconData {
            rgba: Vec::from(ICON_BYTES),
            width: ICON_SIZE.0,
            height: ICON_SIZE.1,
        }),

        ..Default::default()
    };

    match eframe::run_native(
        APP_NAME_FORMATTED,
        options,
        Box::new(|cctx| Box::new(StoikApp::new(cctx))),
    ) {
        Ok(()) => (),
        Err(e) => println!("Error running {APP_NAME}: {e}"), // TODO, better error handling
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            APP_NAME, // hardcode it
            web_options,
            Box::new(|cc| Box::new(StoikApp::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });
}
