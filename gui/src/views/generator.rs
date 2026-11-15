use egui::{Color32, RichText, Ui};

use crate::models::RunParams;

pub struct GeneratorView {
    pub params: RunParams,
    pub plat_x: bool,
    pub plat_linkedin: bool,
    pub plat_reddit: bool,
    pub plat_discord: bool,
    pub plat_facebook: bool,
    pub plat_github: bool,
}

impl Default for GeneratorView {
    fn default() -> Self {
        Self {
            params: RunParams::default(),
            plat_x: true,
            plat_linkedin: true,
            plat_reddit: true,
            plat_discord: true,
            plat_facebook: false,
            plat_github: false,
        }
    }
}

impl GeneratorView {
    pub fn sync_platforms(&mut self) {
        let mut list = Vec::new();
        if self.plat_x {
            list.push("x".to_string());
        }
        if self.plat_linkedin {
            list.push("linkedin".to_string());
        }
        if self.plat_reddit {
            list.push("reddit".to_string());
        }
        if self.plat_discord {
            list.push("discord".to_string());
        }
        if self.plat_facebook {
            list.push("facebook".to_string());
        }
        if self.plat_github {
            list.push("github".to_string());
        }
        self.params.target_platforms = list;
    }

    pub fn ui(&mut self, ui: &mut Ui, is_generating: bool) -> Option<RunParams> {
        let mut trigger_generate = false;

        ui.heading("Generate Developer Log Drafts");
        ui.label(
            "Inspect git commits, diffs, and codebase architecture to generate tailored multi-platform dev-logs.",
        );
        ui.add_space(8.0);

        egui::Grid::new("generator_grid")
            .num_columns(2)
            .spacing([12.0, 10.0])
            .show(ui, |ui| {
                // Repository row
                ui.label(RichText::new("Repository:").strong());
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.params.repo)
                            .hint_text("'.' or /path/to/local/repo or owner/repo")
                            .desired_width(380.0),
                    );
                    if ui.button("📁 Browse Local").clicked() {
                        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                            self.params.repo = folder.display().to_string();
                        }
                    }
                });
                ui.end_row();

                // Commits row
                ui.label(RichText::new("Commits to Inspect:").strong());
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.params.commits, 1..=50).text("commits"));
                });
                ui.end_row();

                // Narrative Focus Type
                ui.label(RichText::new("Narrative Focus:").strong());
                egui::ComboBox::from_id_salt("focus_type_combo")
                    .selected_text(&self.params.focus_type)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.params.focus_type, "auto".into(), "auto (infer from commits/diff)");
                        ui.selectable_value(&mut self.params.focus_type, "bug".into(), "bug (deep post-mortem & bottleneck)");
                        ui.selectable_value(&mut self.params.focus_type, "architecture".into(), "architecture (technical trade-offs)");
                        ui.selectable_value(&mut self.params.focus_type, "motivation".into(), "motivation (why this was built)");
                        ui.selectable_value(&mut self.params.focus_type, "showcase".into(), "showcase (feature demo & visuals)");
                        ui.selectable_value(&mut self.params.focus_type, "full_project".into(), "full_project (holistic project story)");
                        ui.selectable_value(&mut self.params.focus_type, "tech_stack".into(), "tech_stack (libraries & stack design)");
                    });
                ui.end_row();

                // Screenshot / Media
                ui.label(RichText::new("Media / Screenshot:").strong());
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.params.image_path)
                            .hint_text("Optional screenshot or video path")
                            .desired_width(380.0),
                    );
                    if ui.button("🖼 Select File").clicked() {
                        if let Some(file) = rfd::FileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
                            .pick_file()
                        {
                            self.params.image_path = file.display().to_string();
                        }
                    }
                });
                ui.end_row();

                // Custom Problem (Optional)
                ui.label(RichText::new("Problem Context:").strong());
                ui.add(
                    egui::TextEdit::multiline(&mut self.params.problem)
                        .hint_text("Optional: Specific bottleneck, edge case, or motivation (leave blank to auto-infer from latest commit)")
                        .desired_rows(2)
                        .desired_width(480.0),
                );
                ui.end_row();

                // Custom Solution (Optional)
                ui.label(RichText::new("Solution Context:").strong());
                ui.add(
                    egui::TextEdit::multiline(&mut self.params.solution)
                        .hint_text("Optional: Solution details, architectural trade-offs, or takeaways (leave blank to auto-infer)")
                        .desired_rows(2)
                        .desired_width(480.0),
                );
                ui.end_row();

                // Selective Platforms
                ui.label(RichText::new("Target Platforms:").strong());
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.plat_x, "X / Twitter");
                    ui.checkbox(&mut self.plat_linkedin, "LinkedIn");
                    ui.checkbox(&mut self.plat_reddit, "Reddit");
                    ui.checkbox(&mut self.plat_discord, "Discord");
                    ui.checkbox(&mut self.plat_facebook, "Facebook");
                    ui.checkbox(&mut self.plat_github, "GitHub");
                });
                ui.end_row();
            });

        self.sync_platforms();
        ui.add_space(14.0);

        ui.horizontal(|ui| {
            if is_generating {
                ui.spinner();
                ui.label(RichText::new("Inspecting repo & generating drafts with LLM...").color(Color32::from_rgb(100, 180, 255)));
            } else {
                let btn = ui.add_sized(
                    [200.0, 36.0],
                    egui::Button::new(RichText::new("🚀 Generate Drafts").strong().size(15.0)),
                );
                if btn.clicked() {
                    trigger_generate = true;
                }
            }
        });

        if trigger_generate {
            Some(self.params.clone())
        } else {
            None
        }
    }
}

