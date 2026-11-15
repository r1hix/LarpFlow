use std::sync::mpsc;
use egui::{CentralPanel, Color32, RichText, TopBottomPanel};

use crate::backend;
use crate::models::{DispatchResult, DoctorReport, RunResponse};
use crate::views::generator::GeneratorView;
use crate::views::history::{HistoryAction, HistoryView};
use crate::views::preview::{PreviewAction, PreviewView};
use crate::views::settings::SettingsView;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppTab {
    Generator,
    Review,
    History,
    Settings,
}

pub struct LarpFlowApp {
    active_tab: AppTab,
    generator: GeneratorView,
    preview: PreviewView,
    history: HistoryView,
    settings: SettingsView,

    // Async task communication
    generate_rx: Option<mpsc::Receiver<Result<RunResponse, String>>>,
    dispatch_rx: Option<mpsc::Receiver<Result<Vec<DispatchResult>, String>>>,
    doctor_rx: Option<mpsc::Receiver<Result<DoctorReport, String>>>,

    is_generating: bool,
    is_dispatching: bool,
    dry_run_cached: bool,
}

impl Default for LarpFlowApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        backend::run_doctor_async(tx);

        Self {
            active_tab: AppTab::Generator,
            generator: GeneratorView::default(),
            preview: PreviewView::default(),
            history: HistoryView::default(),
            settings: SettingsView {
                report: None,
                is_checking: true,
                error_message: None,
            },
            generate_rx: None,
            dispatch_rx: None,
            doctor_rx: Some(rx),
            is_generating: false,
            is_dispatching: false,
            dry_run_cached: false,
        }
    }
}

impl eframe::App for LarpFlowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll background async tasks
        self.poll_channels(ctx);

        // Header Panel
        TopBottomPanel::top("header_panel").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("⚡ LarpFlow").strong().size(20.0).color(Color32::from_rgb(100, 180, 255)));
                ui.label(RichText::new("Autonomous Dev-Log Engine").color(Color32::GRAY));

                ui.add_space(20.0);
                ui.selectable_value(&mut self.active_tab, AppTab::Generator, "🚀 Generator");
                ui.selectable_value(&mut self.active_tab, AppTab::Review, "📝 Review & Gatekeeper");
                ui.selectable_value(&mut self.active_tab, AppTab::History, "📜 History");
                ui.selectable_value(&mut self.active_tab, AppTab::Settings, "⚙ Settings & Doctor");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.dry_run_cached {
                        ui.colored_label(
                            Color32::from_rgb(255, 179, 0),
                            RichText::new("⚠️ DRY RUN").strong(),
                        );
                    } else {
                        ui.colored_label(
                            Color32::from_rgb(76, 175, 80),
                            RichText::new("● LIVE DISPATCH").strong(),
                        );
                    }
                });
            });
            ui.add_space(6.0);
        });

        // Bottom Status Bar
        TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let py = backend::find_python_binary();
                ui.label(format!("Python: {}", py.display()));
                ui.separator();
                let db = backend::find_storage_db();
                ui.label(format!("Database: {}", db.display()));
            });
        });

        // Central Content
        CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                AppTab::Generator => {
                    if let Some(params) = self.generator.ui(ui, self.is_generating) {
                        self.is_generating = true;
                        let (tx, rx) = mpsc::channel();
                        self.generate_rx = Some(rx);
                        backend::run_generate_async(params, tx);
                    }
                }

                AppTab::Review => {
                    let action = self.preview.ui(ui, self.is_dispatching, self.dry_run_cached);
                    match action {
                        Some(PreviewAction::SaveEdits(draft_id, drafts)) => {
                            match backend::save_draft_edits(draft_id, &drafts) {
                                Ok(_) => {
                                    self.preview.status_message = Some("✓ Saved edits to SQLite".into());
                                    self.history.refresh();
                                }
                                Err(e) => {
                                    self.preview.status_message = Some(format!("✗ Save failed: {e}"));
                                }
                            }
                        }
                        Some(PreviewAction::Dispatch(draft_id, platforms, image)) => {
                            self.is_dispatching = true;
                            let (tx, rx) = mpsc::channel();
                            self.dispatch_rx = Some(rx);
                            backend::run_dispatch_async(draft_id, platforms, image, tx);
                        }
                        None => {}
                    }
                }

                AppTab::History => {
                    if let Some(action) = self.history.ui(ui) {
                        match action {
                            HistoryAction::LoadDraft(record) => {
                                self.preview.load_draft(
                                    record.id,
                                    record.repo,
                                    record.drafts,
                                    record.status,
                                );
                                self.active_tab = AppTab::Review;
                            }
                        }
                    }
                }

                AppTab::Settings => {
                    if self.settings.ui(ui) {
                        self.settings.is_checking = true;
                        let (tx, rx) = mpsc::channel();
                        self.doctor_rx = Some(rx);
                        backend::run_doctor_async(tx);
                    }
                }
            }
        });
    }
}

impl LarpFlowApp {
    fn poll_channels(&mut self, ctx: &egui::Context) {
        // Poll Generator output
        if let Some(rx) = &self.generate_rx {
            if let Ok(res) = rx.try_recv() {
                self.is_generating = false;
                match res {
                    Ok(run_resp) => {
                        self.preview.load_draft(
                            run_resp.draft_id,
                            run_resp.repo,
                            run_resp.drafts,
                            "pending".to_string(),
                        );
                        self.preview.status_message = Some(format!("✓ Draft #{} generated successfully", run_resp.draft_id));
                        self.history.refresh();
                        self.active_tab = AppTab::Review;
                    }
                    Err(e) => {
                        self.preview.status_message = Some(format!("✗ Generation error: {e}"));
                    }
                }
                ctx.request_repaint();
            }
        }

        // Poll Dispatch output
        if let Some(rx) = &self.dispatch_rx {
            if let Ok(res) = rx.try_recv() {
                self.is_dispatching = false;
                match res {
                    Ok(results) => {
                        let has_ok = results.iter().any(|r| r.state == "ok");
                        let has_failed = results.iter().any(|r| r.state == "failed");
                        self.preview.status = if self.dry_run_cached {
                            "dry_run".to_string()
                        } else if has_failed {
                            if has_ok { "partial".to_string() } else { "failed".to_string() }
                        } else {
                            "posted".to_string()
                        };
                        self.preview.audit_results = results;
                        self.preview.status_message = Some("✓ Dispatch complete (see audit below)".into());
                        self.history.refresh();
                    }
                    Err(e) => {
                        self.preview.status_message = Some(format!("✗ Dispatch failed: {e}"));
                    }
                }
                ctx.request_repaint();
            }
        }

        // Poll Doctor output
        if let Some(rx) = &self.doctor_rx {
            if let Ok(res) = rx.try_recv() {
                self.settings.is_checking = false;
                match res {
                    Ok(rep) => {
                        self.dry_run_cached = rep.dry_run;
                        self.settings.report = Some(rep);
                        self.settings.error_message = None;
                    }
                    Err(e) => {
                        self.settings.error_message = Some(e);
                    }
                }
                ctx.request_repaint();
            }
        }
    }
}
