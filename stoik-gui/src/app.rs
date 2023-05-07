use eframe::{App, CreationContext, Frame};
use egui::{CentralPanel, Context};

pub const APP_NAME: &str = "stoik-gui";

#[derive(Default)]
pub struct StoikApp {}

impl StoikApp {
    pub fn new(_cctx: &CreationContext) -> Self {
        // customise eframe here using ctx
        Self::default()
    }
}

impl App for StoikApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello there");
            ui.label("Amog us");
        });
    }
}
