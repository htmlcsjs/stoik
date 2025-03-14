#[allow(unused_imports)]
use app::{APP_NAME, APP_NAME_FORMATTED};

mod app;
pub use app::StoikApp;

#[cfg(not(target_arch = "wasm32"))]
fn init_icon(builder: egui::ViewportBuilder) -> egui::ViewportBuilder {
    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-512.png"));
    if let Ok(data) = icon_data {
        builder.with_icon(data)
    } else {
        builder
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use eframe::NativeOptions;

    let options = NativeOptions {
        window_builder: Some(Box::new(init_icon)),
        ..Default::default()
    };

    env_logger::init();

    match eframe::run_native(
        APP_NAME_FORMATTED,
        options,
        Box::new(|cctx| Ok(Box::new(StoikApp::new(cctx)))),
    ) {
        Ok(()) => (),
        Err(e) => log::error!("Error running {APP_NAME}: {e}"), // TODO, better error handling
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("stoik-gui")
            .expect("Failed to find stoik-gui")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("stoik-gui was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(StoikApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
