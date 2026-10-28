//! The settings window (tray → Settings). Runs as its own process (`Mousemeeter.exe --settings`)
//! so its event loop never competes with the input hooks of the tray process.

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use eframe::egui::{self, RichText};

use crate::config::Config;

pub const TITLE: &str = "Mousemeeter Settings";
/// Command-line argument that opens the settings window instead of starting the tray app.
pub const ARG: &str = "--settings";

/// Shows the window and returns whether the config was saved.
pub fn run(dir: &Path) -> bool {
    let saved = Rc::new(Cell::new(false));
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(TITLE)
        .with_inner_size([440.0, 620.0])
        .with_min_inner_size([380.0, 400.0]);
    if let Some(icon) = icon() {
        viewport = viewport.with_icon(Arc::new(icon));
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let window = SettingsWindow::new(dir, saved.clone());
    let result = eframe::run_native(TITLE, options, Box::new(|_| Ok(Box::new(window))));
    result.is_ok() && saved.get()
}

fn icon() -> Option<egui::IconData> {
    let image = image::load_from_memory_with_format(
        include_bytes!("../../../assets/icon.ico"),
        image::ImageFormat::Ico,
    )
    .ok()?
    .into_rgba8();
    Some(egui::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}

struct SettingsWindow {
    path: PathBuf,
    config: Config,
    /// Profile XML files next to the exe.
    profile_files: Vec<String>,
    /// Empty means the default install location.
    voicemeeter_path: String,
    new_title: String,
    error: Option<String>,
    saved: Rc<Cell<bool>>,
}

impl SettingsWindow {
    fn new(dir: &Path, saved: Rc<Cell<bool>>) -> Self {
        let path = dir.join("config.ini");
        let config = Config::load(&path);
        SettingsWindow {
            voicemeeter_path: config
                .voicemeeter_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            profile_files: xml_files(dir),
            path,
            config,
            new_title: String::new(),
            error: None,
            saved,
        }
    }

    fn save(&mut self) -> Result<(), String> {
        let path = self.voicemeeter_path.trim();
        self.config.voicemeeter_path = (!path.is_empty()).then(|| PathBuf::from(path));
        self.config
            .deactivate_on_window
            .retain(|title| !title.trim().is_empty());
        self.config
            .save(&self.path)
            .map_err(|e| format!("Could not save {}: {e}", self.path.display()))
    }

    fn mouse_section(&mut self, ui: &mut egui::Ui) {
        section(ui, "Volume", |ui| {
            egui::Grid::new("volume").num_columns(2).show(ui, |ui| {
                let outputs = &mut self.config.outputs;
                for (label, strip) in [
                    ("Backward + wheel", &mut outputs.output1),
                    ("Forward + wheel", &mut outputs.output2),
                    ("Both + wheel", &mut outputs.output3),
                ] {
                    ui.label(label);
                    ui.add(egui::DragValue::new(strip).range(0..=7).prefix("Strip "));
                    ui.end_row();
                }
                ui.label("Step per wheel notch");
                ui.add(
                    egui::DragValue::new(&mut self.config.volume_change_amount)
                        .range(0.1..=12.0)
                        .speed(0.1)
                        .fixed_decimals(1)
                        .suffix(" dB"),
                );
                ui.end_row();
            });
            hint(
                ui,
                "Voicemeeter Potato strips: 0–4 hardware inputs, 5 VAIO, 6 AUX, 7 VAIO3.",
            );
        });
    }

    fn profiles_section(&mut self, ui: &mut egui::Ui) {
        section(ui, "Profiles", |ui| {
            egui::Grid::new("profiles").num_columns(2).show(ui, |ui| {
                let files = &self.profile_files;
                for (label, value) in [
                    ("Default", &mut self.config.default_file),
                    ("F24", &mut self.config.profile1_file),
                    ("Double F24", &mut self.config.profile2_file),
                ] {
                    ui.label(label);
                    profile_picker(ui, label, value, files);
                    ui.end_row();
                }
            });
            hint(
                ui,
                "Profiles are Voicemeeter XML files saved next to Mousemeeter.exe. \
                 Pressing a profile's key again goes back to the default.",
            );
        });
    }

    fn windows_section(&mut self, ui: &mut egui::Ui) {
        section(ui, "Disable on windows", |ui| {
            hint(
                ui,
                "The mouse gestures are ignored while one of these windows is in front.",
            );
            let mut remove = None;
            for (i, title) in self.config.deactivate_on_window.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(title).desired_width(300.0));
                    if ui.small_button("✖").on_hover_text("Remove").clicked() {
                        remove = Some(i);
                    }
                });
            }
            if let Some(i) = remove {
                self.config.deactivate_on_window.remove(i);
            }
            ui.horizontal(|ui| {
                let edit = ui.add(
                    egui::TextEdit::singleline(&mut self.new_title)
                        .hint_text("Window title")
                        .desired_width(300.0),
                );
                let entered = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (ui.button("Add").clicked() || entered) && !self.new_title.trim().is_empty() {
                    let title = std::mem::take(&mut self.new_title);
                    self.config
                        .deactivate_on_window
                        .push(title.trim().to_string());
                }
            });
            ui.horizontal(|ui| {
                ui.label("Match");
                egui::ComboBox::from_id_salt("match_mode")
                    .selected_text(match_mode_label(self.config.title_match_mode))
                    .show_ui(ui, |ui| {
                        for mode in 1..=3 {
                            ui.selectable_value(
                                &mut self.config.title_match_mode,
                                mode,
                                match_mode_label(mode),
                            );
                        }
                    });
            });
        });
    }

    fn startup_section(&mut self, ui: &mut egui::Ui) {
        section(ui, "Startup", |ui| {
            hint(
                ui,
                "Applied the next time Mousemeeter starts (tray → Reload).",
            );
            let config = &mut self.config;
            ui.checkbox(
                &mut config.reset_on_startup,
                "Load the default profile on start",
            );
            ui.checkbox(&mut config.run_as_admin, "Run as administrator");
            ui.checkbox(&mut config.set_affinity, "Raise Voicemeeter's priority");
            ui.checkbox(
                &mut config.set_crackling_fix,
                "Crackling fix (pin audiodg to one core, needs administrator)",
            );
            ui.label("Voicemeeter folder");
            ui.add(
                egui::TextEdit::singleline(&mut self.voicemeeter_path)
                    .hint_text(r"Default (C:\Program Files (x86)\VB\Voicemeeter)")
                    .desired_width(f32::INFINITY),
            );
        });
    }
}

impl eframe::App for SettingsWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("buttons").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    match self.save() {
                        Ok(()) => {
                            self.saved.set(true);
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                if ui.button("Cancel").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if let Some(error) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            });
            ui.add_space(6.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.mouse_section(ui);
                self.profiles_section(ui);
                self.windows_section(ui);
                self.startup_section(ui);
            });
        });
    }
}

fn section(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(4.0);
    ui.heading(title);
    ui.add_space(2.0);
    add_contents(ui);
    ui.add_space(4.0);
    ui.separator();
}

fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(text).weak().small());
}

fn match_mode_label(mode: u8) -> &'static str {
    match mode {
        1 => "Title starts with",
        2 => "Title contains",
        _ => "Exact title",
    }
}

fn profile_picker(ui: &mut egui::Ui, id: &str, value: &mut String, files: &[String]) {
    let missing = !files.contains(value);
    let selected = if missing {
        format!("{value} (not found)")
    } else {
        value.clone()
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected)
        .width(220.0)
        .show_ui(ui, |ui| {
            for file in files {
                ui.selectable_value(value, file.clone(), file);
            }
            if files.is_empty() {
                ui.label("No .xml files next to Mousemeeter.exe");
            }
        });
}

fn xml_files(dir: &Path) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| name.to_ascii_lowercase().ends_with(".xml"))
        .collect();
    files.sort_by_key(|name| name.to_lowercase());
    files
}
