use egui::{Color32, RichText, ScrollArea, Ui};

use crate::backend;
use crate::models::DraftRecord;

pub struct HistoryView {
    pub records: Vec<DraftRecord>,
    pub filter_text: String,
    pub status_filter: String,
    pub error_message: Option<String>,
}

impl Default for HistoryView {
    fn default() -> Self {
        let mut view = Self {
            records: Vec::new(),
            filter_text: String::new(),
            status_filter: "all".to_string(),
            error_message: None,
        };
        view.refresh();
        view
    }
}

pub enum HistoryAction {
    LoadDraft(DraftRecord),
}

impl HistoryView {
    pub fn refresh(&mut self) {
        match backend::load_history_direct() {
            Ok(records) => {
                self.records = records;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load SQLite history: {e}"));
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui) -> Option<HistoryAction> {
        let mut action = None;

        ui.horizontal(|ui| {
            ui.heading("Draft History (storage/history.sqlite)");
            ui.add_space(20.0);
            if ui.button("🔄 Refresh").clicked() {
                self.refresh();
            }
        });

        if let Some(err) = &self.error_message {
            ui.colored_label(Color32::from_rgb(239, 83, 80), err);
        }

        ui.add_space(6.0);

        // Filter Bar
        ui.horizontal(|ui| {
            ui.label("Search repo:");
            ui.add(egui::TextEdit::singleline(&mut self.filter_text).hint_text("Filter by repo name..."));

            ui.add_space(10.0);
            ui.label("Status:");
            egui::ComboBox::from_id_salt("history_status_filter")
                .selected_text(&self.status_filter)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.status_filter, "all".into(), "All");
                    ui.selectable_value(&mut self.status_filter, "posted".into(), "Posted");
                    ui.selectable_value(&mut self.status_filter, "pending".into(), "Pending");
                    ui.selectable_value(&mut self.status_filter, "partial".into(), "Partial");
                    ui.selectable_value(&mut self.status_filter, "dry_run".into(), "Dry Run");
                });
        });

        ui.add_space(8.0);
        ui.separator();

        let filtered_records: Vec<&DraftRecord> = self
            .records
            .iter()
            .filter(|r| {
                if !self.filter_text.trim().is_empty()
                    && !r.repo.to_lowercase().contains(&self.filter_text.to_lowercase())
                {
                    return false;
                }
                if self.status_filter != "all" && r.status.to_lowercase() != self.status_filter {
                    return false;
                }
                true
            })
            .collect();

        if filtered_records.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(RichText::new("No drafts matching filter.").color(Color32::GRAY));
            });
            return None;
        }

        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("history_table")
                .striped(true)
                .spacing([14.0, 8.0])
                .show(ui, |ui| {
                    // Header row
                    ui.label(RichText::new("ID").strong());
                    ui.label(RichText::new("Date / Time").strong());
                    ui.label(RichText::new("Repository").strong());
                    ui.label(RichText::new("Status").strong());
                    ui.label(RichText::new("Dispatched To").strong());
                    ui.label(RichText::new("Actions").strong());
                    ui.end_row();

                    for record in filtered_records {
                        ui.label(format!("#{}", record.id));
                        ui.label(&record.created_at);
                        ui.label(RichText::new(&record.repo).strong());

                        let (color, text) = match record.status.as_str() {
                            "posted" => (Color32::from_rgb(76, 175, 80), "posted"),
                            "partial" => (Color32::from_rgb(255, 167, 38), "partial"),
                            "dry_run" => (Color32::from_rgb(255, 214, 0), "dry_run"),
                            _ => (Color32::GRAY, record.status.as_str()),
                        };
                        ui.colored_label(color, text);

                        ui.label(record.platforms.as_deref().unwrap_or("-"));

                        if ui.button("🔍 Load into Review").clicked() {
                            action = Some(HistoryAction::LoadDraft(record.clone()));
                        }
                        ui.end_row();
                    }
                });
        });

        action
    }
}
