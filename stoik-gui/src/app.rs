use std::fmt::Display;

use eframe::{App, CreationContext, Frame};
use egui::{
    widgets, CentralPanel, Context, RichText, SidePanel, Slider, TextEdit, TextStyle,
    TopBottomPanel, Ui,
};
use stoik::{formula::Molecule, StoikError};
use strum::{EnumIter, IntoEnumIterator};

use crate::util::trace;

#[allow(unused)]
pub const APP_NAME: &str = "stoik-gui";
pub const APP_NAME_FORMATTED: &str = "Stoik GUI";

#[derive(Default)]
pub struct StoikApp {
    settings_open: bool,
    display_mode: Mode,
    mode_data: ModeData,
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
        log::info!("started");
        TopBottomPanel::top(id("top")).show(ctx, |ui| {
            trace!(ui);
            ui.horizontal(|ui| {
                ui.visuals_mut().button_frame = false;
                self.top_bar(ui, frame);
            });
        });

        SidePanel::left(id("settings")).show_animated(ctx, self.settings_open, |ui| {
            trace!(ui);

            ui.vertical_centered(|ui| {
                ui.heading("⚙ Settings");
            });
            ui.separator();
            self.settings_panel(ui, frame);
        });

        CentralPanel::default().show(ctx, |ui| self.main_win(ui, frame, self.display_mode));

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

        ui.separator();

        for i in Mode::iter() {
            if ui
                .selectable_value(&mut self.display_mode, i, i.to_string())
                .clicked()
            {
                self.mode_data.changed = false;
                // TODO: RESET STATE DEPENDING ON MODE
            };
        }

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                widgets::global_dark_light_mode_switch(ui);
                ui.separator();
                egui::warn_if_debug_build(ui);
            });
        });
    }

    fn settings_panel(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        trace!(ui);

        {
            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.checkbox(&mut debug_on_hover, "Debug on hover")
                .on_hover_text("Show structure of the ui when you hover with the mouse");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        }

        let mut style = (*ui.ctx().style()).clone();
        let mut text_size = style.text_styles.get(&TextStyle::Monospace).unwrap().size;

        ui.label("Text size");
        let response = ui.add(Slider::new(&mut text_size, 1.0..=36.0).clamp_to_range(false));
        if response.drag_released() || response.lost_focus() {
            let mut text_styles = egui::style::default_text_styles();
            if text_size < 1.0 {
                text_size = 14.0;
            }
            for (_, font_id) in text_styles.iter_mut() {
                font_id.size *= text_size / 12.0
            }

            style.text_styles = text_styles;
            ui.ctx().set_style(style);
        }
    }

    fn main_win(&mut self, ui: &mut Ui, _frame: &mut Frame, mode: Mode) {
        match mode {
            Mode::Text => {
                trace!(ui);
                ui.heading(RichText::new("Formula"));
                let mut input = self.mode_data.text_input.clone();
                let res = ui.add(
                    TextEdit::singleline(&mut input)
                        .font(TextStyle::Monospace)
                        .desired_width(f32::INFINITY),
                );

                // TODO: custom deliminators
                if res.changed() {
                    self.mode_data.text_input = input.replace("=>", "->");
                    self.mode_data.changed = true;
                }

                if self.mode_data.changed && res.lost_focus() {
                    if !self.mode_data.text_input.contains("->") {
                        if !self.mode_data.text_input.is_empty() {
                            self.mode_data.error_msg = Some(
                                "No RHS is given, use -> or => to indicate between them"
                                    .to_string(),
                            );
                        } else {
                            self.mode_data.error_msg = None;
                            self.mode_data.lhs.clear();
                            self.mode_data.rhs.clear();
                        }
                    } else {
                        self.mode_data.error_msg = None;
                        let (mut lhs_str, mut rhs_str) =
                            self.mode_data.text_input.split_once("->").unwrap();

                        lhs_str = lhs_str.trim();
                        rhs_str = rhs_str.trim();

                        self.mode_data.lhs.clear();
                        for formula in lhs_str.split('+').map(|x| x.trim()) {
                            match Molecule::from_formula(formula) {
                                Ok(mol) => self.mode_data.lhs.push((mol, formula.to_string())),
                                Err(e) => {
                                    self.mode_data.error_msg = Some(generate_error_msg(e, formula));
                                    break;
                                }
                            }
                        }

                        self.mode_data.rhs.clear();
                        if self.mode_data.error_msg.is_none() {
                            for formula in rhs_str.split('+').map(|x| x.trim()) {
                                match Molecule::from_formula(formula) {
                                    Ok(mol) => self.mode_data.rhs.push((mol, formula.to_string())),
                                    Err(e) => {
                                        self.mode_data.error_msg =
                                            Some(generate_error_msg(e, formula))
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Mode::List => {
                trace!(ui);
            }
        }

        trace!(ui);

        if let Some(s) = &self.mode_data.error_msg {
            ui.label(
                RichText::new(s)
                    .color(ui.visuals().error_fg_color)
                    .monospace(),
            );
        } else if !self.mode_data.lhs.is_empty() && !self.mode_data.rhs.is_empty() {
            ui.heading("LHS");
            for (mol, formula) in &self.mode_data.lhs {
                let mut pairs = mol.get_map().into_iter().collect::<Vec<_>>();
                pairs.sort_by(|(ak, _), (bk, _)| ak.cmp(bk));
                let mut mapped = Vec::new();
                for (atom, count) in pairs {
                    mapped.push(format!("{atom}: {count}"))
                }
                ui.label(format!("{formula}: {}", mapped.join(", ")));
            }

            ui.heading("RHS");
            for (mol, formula) in &self.mode_data.rhs {
                let mut pairs = mol.get_map().into_iter().collect::<Vec<_>>();
                pairs.sort_by(|(ak, _), (bk, _)| ak.cmp(bk));
                let mut mapped = Vec::new();
                for (atom, count) in pairs {
                    mapped.push(format!("{atom}: {count}"))
                }
                ui.label(format!("{formula}: {}", mapped.join(", ")));
            }
        }
    }
}

pub fn id(id: &str) -> String {
    format!("{APP_NAME}-{id}")
}

fn generate_error_msg(e: StoikError, formula: &str) -> String {
    match e {
        StoikError::InvalidToken(loc) => {
            loc.format_msg(formula, "Malformed formula", "Illegal token")
        }
        StoikError::NumberFirst(loc) => loc.format_msg(
            formula,
            "Malformed formula",
            "Compound groups cannot start with numbers",
        ),
        StoikError::UnpairedParenthesis(loc) => {
            loc.format_msg(formula, "Malformed formula", "Unpaired parenthesis")
        }
        StoikError::UnpairedBracket(loc) => {
            loc.format_msg(formula, "Malformed formula", "Unpaired bracket")
        }
        e => e.to_string(),
    }
}

#[derive(Debug, Default, PartialEq, Eq, EnumIter, Clone, Copy)]
enum Mode {
    #[default]
    Text,
    List,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => write!(f, "Text"),
            Self::List => write!(f, "List"),
        }
    }
}

#[derive(Debug, Default)]
struct ModeData {
    text_input: String,
    error_msg: Option<String>,
    lhs: Vec<(Molecule, String)>,
    rhs: Vec<(Molecule, String)>,
    changed: bool,
}
