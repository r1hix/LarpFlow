use egui::{Color32, RichText, Ui};

use crate::backend;
use crate::models::DoctorReport;

pub struct SettingsView {
    pub report: Option<DoctorReport>,
    pub is_checking: bool,
    pub error_message: Option<String>,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self {
            report: None,
            is_checking: false,
            error_message: None,
        }
    }
}

impl SettingsView {
    pub fn ui(&mut self, ui: &mut Ui) -> bool {
        let mut trigger_check = false;

        ui.horizontal(|ui| {
            ui.heading("System Diagnostics & Settings");
            ui.add_space(20.0);
            if self.is_checking {
                ui.spinner();
                ui.label("Running larpflow doctor...");
            } else if ui.button("🩺 Run Diagnostics").clicked() {
                trigger_check = true;
            }
        });

        if let Some(err) = &self.error_message {
            ui.add_space(4.0);
            ui.colored_label(Color32::from_rgb(239, 83, 80), err);
        }

        ui.add_space(10.0);

        if let Some(r) = &self.report {
            ui.group(|ui| {
                ui.label(RichText::new("Execution Mode & Core AI").strong().size(15.0));
                ui.horizontal(|ui| {
                    ui.label("Dry Run Mode:");
                    if r.dry_run {
                        ui.colored_label(Color32::from_rgb(255, 179, 0), RichText::new("ENABLED (Simulation)").strong());
                    } else {
                        ui.colored_label(Color32::from_rgb(76, 175, 80), RichText::new("LIVE (Real Dispatches)").strong());
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("LLM Model:");
                    ui.label(RichText::new(&r.llm_model).strong());
                    ui.add_space(10.0);
                    ui.label("API Key:");
                    self.badge(ui, r.llm_api_key);
                });

                ui.horizontal(|ui| {
                    ui.label("GitHub Token:");
                    self.badge(ui, r.github_token);
                    ui.label("(optional for public repos)");
                });

                ui.horizontal(|ui| {
                    ui.label("LinkedIn Profile:");
                    self.badge(ui, r.linkedin_profile);
                    ui.label("(config/linkedin_profile.md)");
                });
            });

            ui.add_space(8.0);

            ui.group(|ui| {
                ui.label(RichText::new("Social Platform Integrations").strong().size(15.0));
                egui::Grid::new("platforms_grid").num_columns(2).spacing([20.0, 6.0]).show(ui, |ui| {
                    ui.label("X / Twitter (API v2):");
                    self.badge(ui, r.x);
                    ui.end_row();

                    ui.label("LinkedIn API:");
                    self.badge(ui, r.linkedin);
                    ui.end_row();

                    ui.label("Reddit (PRAW):");
                    ui.horizontal(|ui| {
                        self.badge(ui, r.reddit);
                        ui.label(format!("allowed subs: {:?}", r.reddit_allowed_subs));
                    });
                    ui.end_row();

                    ui.label("Discord (Webhook):");
                    self.badge(ui, r.discord);
                    ui.end_row();

                    ui.label("Facebook Graph API:");
                    self.badge(ui, r.facebook);
                    ui.end_row();

                    ui.label("Telegram Bot:");
                    self.badge(ui, r.telegram_bot_token && r.telegram_owner_id);
                    ui.end_row();
                });
            });
        } else if !self.is_checking {
            ui.label("Click 'Run Diagnostics' to check your .env configuration and connected platforms.");
        }

        ui.add_space(14.0);
        ui.group(|ui| {
            ui.label(RichText::new("Environment & Binary Paths").strong().size(15.0));
            ui.label(format!("Python Binary: {}", backend::find_python_binary().display()));
            ui.label(format!("Project Root:  {}", backend::find_project_root().display()));
            ui.label(format!("Storage DB:    {}", backend::find_storage_db().display()));
        });

        trigger_check
    }

    fn badge(&self, ui: &mut Ui, ok: bool) {
        if ok {
            ui.colored_label(Color32::from_rgb(76, 175, 80), RichText::new("✓ CONFIGURED").strong());
        } else {
            ui.colored_label(Color32::from_rgb(239, 83, 80), RichText::new("✗ MISSING").strong());
        }
    }
}
