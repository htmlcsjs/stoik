use app::{StoikApp, APP_NAME};
use eframe::{IconData, NativeOptions};

mod app;

include! {concat!(env!("OUT_DIR"), "/icon.rs")}

fn main() {
    let options = NativeOptions {
        icon_data: Some(IconData {
            rgba: Vec::from(ICON_BYTES),
            width: ICON_SIZE.0,
            height: ICON_SIZE.1,
        }),

        ..Default::default()
    };
    match eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cctx| Box::new(StoikApp::new(cctx))),
    ) {
        Ok(()) => (),
        Err(e) => println!("Error running stoik-gui: {e}"), // TODO, better error handling
    }
}
