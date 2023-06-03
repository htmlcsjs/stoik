use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Display,
};

use eframe::{App, CreationContext, Frame};
use egui::{
    widgets, CentralPanel, Context, RichText, SidePanel, Slider, TextEdit, TextStyle,
    TopBottomPanel, Ui,
};
use egui_extras::{Column, TableBody, TableBuilder};
use stoik::{formula::Molecule, StoikError};
use strum::{EnumIter, IntoEnumIterator};

use crate::macros::trace;

#[allow(unused)]
pub const APP_NAME: &str = "stoik-gui";
pub const APP_NAME_FORMATTED: &str = "Stoik GUI";

#[derive(Default)]
pub struct StoikApp {
    settings_open: bool,
    display_mode: Mode,
    mode_data: ModeData,
    all_atoms: bool,
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
                self.mode_data.changed = true;
                if self.display_mode == Mode::Text {
                    self.mode_data.text_input = self
                        .mode_data
                        .lhs_mols
                        .iter()
                        .cloned()
                        .map(|(_, formula)| formula)
                        .collect::<Vec<_>>()
                        .join(" + ")
                        + " -> "
                        + &self
                            .mode_data
                            .rhs_mols
                            .iter()
                            .cloned()
                            .map(|(_, formula)| formula)
                            .collect::<Vec<_>>()
                            .join(" + ");
                }
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

        #[cfg(debug_assertions)]
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

        ui.separator();

        ui.checkbox(&mut self.all_atoms, "Show balanced atoms in summery");
    }

    fn main_win(&mut self, ui: &mut Ui, _frame: &mut Frame, mode: Mode) {
        trace!(ui);

        match mode {
            Mode::Text => self.ui_text(ui),
            Mode::List => self.ui_list(ui),
        }

        if let Some(s) = &self.mode_data.error_msg {
            ui.label(
                RichText::new(s)
                    .color(ui.visuals().error_fg_color)
                    .monospace(),
            );
        } else if !self.mode_data.lhs_mols.is_empty() && !self.mode_data.rhs_mols.is_empty() {
            self.show_balance_summery(ui);
        }
    }

    fn ui_text(&mut self, ui: &mut Ui) {
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
                    self.mode_data.error_msg =
                        Some("No RHS is given, use -> or => to indicate between them".to_string());
                } else {
                    self.mode_data.error_msg = None;
                    self.mode_data.lhs_mols.clear();
                    self.mode_data.rhs_mols.clear();
                }
            } else {
                self.mode_data.error_msg = None;
                let (mut lhs_str, mut rhs_str) =
                    self.mode_data.text_input.split_once("->").unwrap();

                lhs_str = lhs_str.trim();
                rhs_str = rhs_str.trim();

                self.mode_data.lhs_mols.clear();
                for formula in lhs_str.split('+').map(|x| x.trim()) {
                    match Molecule::from_formula(formula) {
                        Ok(mol) => self.mode_data.lhs_mols.push((mol, formula.to_string())),
                        Err(e) => {
                            self.mode_data.error_msg = Some(generate_error_msg(e, formula));
                            break;
                        }
                    }
                }

                self.mode_data.rhs_mols.clear();
                if self.mode_data.error_msg.is_none() {
                    for formula in rhs_str.split('+').map(|x| x.trim()) {
                        match Molecule::from_formula(formula) {
                            Ok(mol) => self.mode_data.rhs_mols.push((mol, formula.to_string())),
                            Err(e) => {
                                self.mode_data.error_msg = Some(generate_error_msg(e, formula))
                            }
                        }
                    }
                }
            }
        }
    }

    fn ui_list(&mut self, ui: &mut Ui) {
        trace!(ui);

        ui.heading("LHS");
        let mut to_del = None;
        for (i, (_, formula)) in self.mode_data.lhs_mols.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    to_del = Some(i);
                }
                ui.label(format_formula(formula));
            });
        }
        if let Some(i) = to_del {
            self.mode_data.lhs_mols.remove(i);
            self.mode_data.changed = true;
        }

        let mut lost_focus = false;
        ui.horizontal(|ui| {
            lost_focus = ui
                .text_edit_singleline(&mut self.mode_data.new_lhs)
                .lost_focus()
                || ui.button("+").clicked();
        });
        if lost_focus && !self.mode_data.new_lhs.is_empty() && self.mode_data.error_msg.is_none() {
            match Molecule::from_formula(&self.mode_data.new_lhs) {
                Ok(mol) => {
                    self.mode_data
                        .lhs_mols
                        .push((mol, self.mode_data.new_lhs.to_string()));
                    self.mode_data.new_lhs.clear();
                    self.mode_data.error_msg = None;
                    self.mode_data.changed = true;
                }
                Err(e) => {
                    self.mode_data.error_msg = Some(generate_error_msg(e, &self.mode_data.new_lhs));
                }
            }
        }

        ui.heading("RHS");
        let mut to_del = None;
        for (i, (_, formula)) in self.mode_data.rhs_mols.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.button("-").clicked() {
                    to_del = Some(i);
                }
                ui.label(format_formula(formula));
            });
        }
        if let Some(i) = to_del {
            self.mode_data.rhs_mols.remove(i);
            self.mode_data.changed = true;
        }

        let mut lost_focus = false;
        ui.horizontal(|ui| {
            lost_focus = ui
                .text_edit_singleline(&mut self.mode_data.new_rhs)
                .lost_focus()
                || ui.button("+").clicked();
        });
        if lost_focus && !self.mode_data.new_rhs.is_empty() && self.mode_data.error_msg.is_none() {
            match Molecule::from_formula(&self.mode_data.new_rhs) {
                Ok(mol) => {
                    self.mode_data
                        .rhs_mols
                        .push((mol, self.mode_data.new_rhs.to_string()));
                    self.mode_data.new_rhs.clear();
                    self.mode_data.error_msg = None;
                    self.mode_data.changed = true;
                }
                Err(e) => {
                    self.mode_data.error_msg = Some(generate_error_msg(e, &self.mode_data.new_rhs));
                }
            }
        }
    }

    fn table_body_contents(&mut self, mut body: TableBody) {
        let height = body
            .ui_mut()
            .style()
            .text_styles
            .get(&TextStyle::Monospace)
            .unwrap()
            .size;
        for (key, val) in &self.mode_data.balanced {
            if self.all_atoms || !val {
                body.row(height, |mut row| {
                    row.col(|ui| {
                        ui.monospace(format!(
                            "{key}: {}",
                            self.mode_data.lhs.get(key).unwrap_or(&0)
                        ));
                    });
                    row.col(|ui| {
                        ui.monospace(format!(
                            "{key}: {}",
                            self.mode_data.rhs.get(key).unwrap_or(&0)
                        ));
                    });
                    if self.all_atoms {
                        row.col(|ui| {
                            if *val {
                                ui.label("✅");
                            } else {
                                ui.label("❌");
                            }
                        });
                    }
                })
            }
        }
    }

    fn show_balance_summery(&mut self, ui: &mut Ui) {
        if self.mode_data.changed {
            self.mode_data.lhs.clear();
            self.mode_data.rhs.clear();
            self.mode_data.balanced.clear();

            for (mol, _) in &self.mode_data.lhs_mols {
                extend_mol_map(&mut self.mode_data.lhs, mol.get_map());
            }

            for (mol, _) in &self.mode_data.rhs_mols {
                extend_mol_map(&mut self.mode_data.rhs, mol.get_map());
            }

            let mut keys = self.mode_data.lhs.keys().collect::<Vec<_>>();
            keys.extend(self.mode_data.rhs.keys());
            keys.dedup();

            for key in keys {
                self.mode_data.balanced.insert(
                    key.to_string(),
                    self.mode_data.lhs.get(key) == self.mode_data.rhs.get(key),
                );
            }
            self.mode_data.changed = false;
        }

        let balanced = self.mode_data.balanced.values().all(|x| *x);

        if balanced {
            ui.heading("Your equasion is balanced");
        } else {
            ui.heading("Your equasion is not balanced");
        }

        if self.all_atoms || !balanced {
            let table = TableBuilder::new(ui)
                .striped(true)
                .columns(Column::auto(), if self.all_atoms { 3 } else { 2 })
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

            table
                .header(24.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Reactants");
                    });
                    header.col(|ui| {
                        ui.label("Products");
                    });
                    if self.all_atoms {
                        header.col(|ui| {
                            ui.label("Balanced");
                        });
                    }
                })
                .body(|body| self.table_body_contents(body));
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

fn format_formula(formula: &str) -> String {
    let mut chars = formula.trim().chars().peekable();
    let mut new = String::new();

    while chars.peek().unwrap_or(&'a').is_ascii_digit() {
        new.push(chars.next().unwrap());
    }

    for i in chars {
        if i.is_ascii_digit() {
            let num = i.to_digit(10).unwrap();
            new.push(char::from_u32(0x2080 + num).unwrap());
        } else {
            new.push(i)
        }
    }

    new
}

// maybe make generic
fn extend_mol_map(main: &mut HashMap<String, i64>, mol: HashMap<String, i64>) {
    for (key, mol_val) in mol {
        if let Entry::Occupied(mut entry) = main.entry(key.clone()) {
            *entry.get_mut() += mol_val;
        } else {
            main.insert(key, mol_val);
        }
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
    lhs_mols: Vec<(Molecule, String)>,
    rhs_mols: Vec<(Molecule, String)>,
    lhs: HashMap<String, i64>,
    rhs: HashMap<String, i64>,
    balanced: HashMap<String, bool>,
    changed: bool,
    new_lhs: String,
    new_rhs: String,
}
