use eframe::{App, CreationContext, Frame};
use egui::{widgets, CentralPanel, Context, SidePanel, Slider, TextStyle, TopBottomPanel, Ui};

#[allow(unused)]
pub const APP_NAME: &str = "stoik-gui";
pub const APP_NAME_FORMATTED: &str = "Stoik GUI";

#[derive(Default)]
pub struct StoikApp {
    settings_open: bool,
}

impl StoikApp {
    pub fn new(cctx: &CreationContext) -> Self {
        // customise eframe here using cctx
        let mut style = (*cctx.egui_ctx.style()).clone();
        for (_, font_id) in style.text_styles.iter_mut() {
            font_id.size *= 14.0 / 12.0
        }

        cctx.egui_ctx.set_style(style);
        Self::default()
    }
}

// TODO: persistance
impl App for StoikApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        TopBottomPanel::top(id("top")).show(ctx, |ui| {
            egui::trace!(ui);
            ui.horizontal(|ui| {
                ui.visuals_mut().button_frame = false;
                self.top_bar(ui, frame);
            });
        });

        SidePanel::left(id("settings")).show_animated(ctx, self.settings_open, |ui| {
            egui::trace!(ui);

            ui.vertical_centered(|ui| {
                ui.heading("⚙ Settings");
            });
            ui.separator();
            self.settings_panel(ui, frame);
        });

        CentralPanel::default().show(ctx, |ui| {
            egui::trace!(ui);
        });

        if !frame.is_web() {
            egui::gui_zoom::zoom_with_keyboard_shortcuts(ctx, frame.info().native_pixels_per_point);
        }
    }
}

impl StoikApp {
    fn top_bar(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        ui.label(APP_NAME_FORMATTED);

        ui.separator();

        ui.toggle_value(&mut self.settings_open, "⚙");
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                widgets::global_dark_light_mode_switch(ui);
                ui.separator();
            });
        });
    }

    fn settings_panel(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        egui::trace!(ui);

        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }

        let mut style = (*ui.ctx().style()).clone();
        let mut text_size = style.text_styles.get(&TextStyle::Monospace).unwrap().size;

        ui.label("Text size");
        if ui
            .add(Slider::new(&mut text_size, 1.0..=36.0).clamp_to_range(false))
            .changed()
        {
            let mut text_styles = egui::style::default_text_styles();
            for (_, font_id) in text_styles.iter_mut() {
                font_id.size *= text_size / 12.0
            }

            style.text_styles = text_styles;
            ui.ctx().set_style(style);
        }
    }
}

pub fn id(id: &str) -> String {
    format!("{APP_NAME}-{id}")
}
