use egui::{Color32, RichText, ScrollArea, Ui};

use crate::models::{DispatchResult, Drafts};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlatformTab {
    X,
    LinkedIn,
    Reddit,
    Discord,
    Facebook,
    GitHub,
}

pub struct PreviewView {
    pub active_tab: PlatformTab,
    pub active_draft_id: Option<i64>,
    pub repo_name: String,
    pub status: String,
    pub drafts: Drafts,
    pub visual_asset: String,
    pub dispatch_x: bool,
    pub dispatch_linkedin: bool,
    pub dispatch_reddit: bool,
    pub dispatch_discord: bool,
    pub dispatch_facebook: bool,
    pub dispatch_github: bool,
    pub audit_results: Vec<DispatchResult>,
    pub status_message: Option<String>,
}

impl Default for PreviewView {
    fn default() -> Self {
        Self {
            active_tab: PlatformTab::LinkedIn,
            active_draft_id: None,
            repo_name: String::new(),
            status: "pending".to_string(),
            drafts: Drafts::default(),
            visual_asset: String::new(),
            dispatch_x: true,
            dispatch_linkedin: true,
            dispatch_reddit: true,
            dispatch_discord: true,
            dispatch_facebook: false,
            dispatch_github: false,
            audit_results: Vec::new(),
            status_message: None,
        }
    }
}

pub enum PreviewAction {
    SaveEdits(i64, Drafts),
    Dispatch(i64, Vec<String>, String),
}

impl PreviewView {
    pub fn load_draft(&mut self, draft_id: i64, repo: String, drafts: Drafts, status: String) {
        self.active_draft_id = Some(draft_id);
        self.repo_name = repo;
        self.status = status;
        self.visual_asset = drafts.visual_asset.clone().unwrap_or_default();
        self.drafts = drafts;
        self.audit_results.clear();
        self.status_message = None;
    }

    pub fn ui(&mut self, ui: &mut Ui, is_dispatching: bool, dry_run: bool) -> Option<PreviewAction> {
        let mut action = None;

        if self.active_draft_id.is_none() {
            ui.vertical_centered(|ui| {
                ui.add_space(60.0);
                ui.label(RichText::new("No draft selected").size(18.0).color(Color32::GRAY));
                ui.label("Generate a new draft from the 'Generator' tab or select one from 'History'.");
            });
            return None;
        }

        let draft_id = self.active_draft_id.unwrap();

        // Header info
        ui.horizontal(|ui| {
            ui.heading(format!("Draft #{} - {}", draft_id, self.repo_name));
            ui.add_space(10.0);

            let (status_color, status_bg) = match self.status.as_str() {
                "posted" => (Color32::WHITE, Color32::from_rgb(46, 125, 50)),
                "partial" => (Color32::WHITE, Color32::from_rgb(230, 81, 0)),
                "dry_run" => (Color32::BLACK, Color32::from_rgb(255, 214, 0)),
                _ => (Color32::WHITE, Color32::from_rgb(66, 66, 66)),
            };

            ui.colored_label(status_color, RichText::new(format!(" [{}] ", self.status.to_uppercase())).background_color(status_bg).strong());

            if dry_run {
                ui.colored_label(
                    Color32::from_rgb(255, 179, 0),
                    RichText::new("⚠️ DRY RUN ENABLED (no live posts)").strong(),
                );
            }
        });

        if let Some(msg) = &self.status_message {
            ui.add_space(4.0);
            ui.label(RichText::new(msg).color(Color32::from_rgb(129, 199, 132)));
        }

        ui.add_space(8.0);

        // Platform Tab Bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.active_tab, PlatformTab::LinkedIn, "💼 LinkedIn");
            ui.selectable_value(&mut self.active_tab, PlatformTab::X, "🐦 X (Twitter Thread)");
            ui.selectable_value(&mut self.active_tab, PlatformTab::Reddit, "🤖 Reddit");
            ui.selectable_value(&mut self.active_tab, PlatformTab::Discord, "💬 Discord");
            ui.selectable_value(&mut self.active_tab, PlatformTab::Facebook, "👥 Facebook");
            ui.selectable_value(&mut self.active_tab, PlatformTab::GitHub, "🐙 GitHub README");
        });

        ui.separator();

        // Platform Content Editor
        ScrollArea::vertical()
            .max_height(340.0)
            .show(ui, |ui| match self.active_tab {
                PlatformTab::LinkedIn => {
                    let word_count = self.drafts.linkedin.split_whitespace().count();
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("LinkedIn Post (140 - 350 words recommended):").strong());
                        let count_color = if (140..=350).contains(&word_count) {
                            Color32::from_rgb(76, 175, 80)
                        } else {
                            Color32::from_rgb(255, 167, 38)
                        };
                        ui.colored_label(count_color, format!("{} words", word_count));
                    });
                    ui.add(
                        egui::TextEdit::multiline(&mut self.drafts.linkedin)
                            .desired_width(ui.available_width())
                            .desired_rows(12),
                    );
                }

                PlatformTab::X => {
                    ui.label(RichText::new("X (Twitter) Thread (max 275 chars/tweet):").strong());
                    let mut remove_idx = None;
                    let total_tweets = self.drafts.tweets.len();
                    for (i, tweet) in self.drafts.tweets.iter_mut().enumerate() {
                        let char_count = tweet.chars().count();
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("Tweet {}/{}", i + 1, total_tweets)).strong());
                                let color = if char_count <= 275 {
                                    Color32::from_rgb(76, 175, 80)
                                } else {
                                    Color32::from_rgb(239, 83, 80)
                                };
                                ui.colored_label(color, format!("{}/275 chars", char_count));
                                if total_tweets > 1 && ui.button("🗑 Remove").clicked() {
                                    remove_idx = Some(i);
                                }
                            });
                            ui.add(
                                egui::TextEdit::multiline(tweet)
                                    .desired_width(ui.available_width())
                                    .desired_rows(3),
                            );
                        });
                    }
                    if let Some(idx) = remove_idx {
                        self.drafts.tweets.remove(idx);
                    }
                    if ui.button("➕ Add Tweet to Thread").clicked() {
                        self.drafts.tweets.push(String::new());
                    }
                }

                PlatformTab::Reddit => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Subreddit:").strong());
                        ui.text_edit_singleline(&mut self.drafts.reddit_subreddit);
                        if !self.drafts.suggested_subreddits.is_empty() {
                            ui.label("Suggestions:");
                            for sub in &self.drafts.suggested_subreddits {
                                if ui.button(sub).clicked() {
                                    self.drafts.reddit_subreddit = sub.clone();
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Title:").strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut self.drafts.reddit_title)
                                .desired_width(ui.available_width() - 80.0),
                        );
                    });
                    ui.label(RichText::new("Post Body (Markdown):").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.drafts.reddit_body)
                            .desired_width(ui.available_width())
                            .desired_rows(10),
                    );
                }

                PlatformTab::Discord => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Embed Title:").strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut self.drafts.discord_title)
                                .desired_width(ui.available_width() - 100.0),
                        );
                    });
                    ui.label(RichText::new("Embed Description:").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.drafts.discord_body)
                            .desired_width(ui.available_width())
                            .desired_rows(10),
                    );
                }

                PlatformTab::Facebook => {
                    ui.label(RichText::new("Facebook Showcase Post:").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.drafts.facebook)
                            .desired_width(ui.available_width())
                            .desired_rows(10),
                    );
                }

                PlatformTab::GitHub => {
                    ui.label(RichText::new("GitHub README Architecture Section:").strong());
                    ui.add(
                        egui::TextEdit::multiline(&mut self.drafts.github_readme_section)
                            .desired_width(ui.available_width())
                            .desired_rows(10),
                    );
                }
            });

        ui.add_space(10.0);

        // Visual Asset Attached
        ui.horizontal(|ui| {
            ui.label(RichText::new("🖼 Media Attachment:").strong());
            ui.add(
                egui::TextEdit::singleline(&mut self.visual_asset)
                    .hint_text("None attached")
                    .desired_width(360.0),
            );
            if ui.button("Browse...").clicked() {
                if let Some(file) = rfd::FileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
                    .pick_file()
                {
                    self.visual_asset = file.display().to_string();
                    self.drafts.visual_asset = Some(self.visual_asset.clone());
                }
            }
        });

        ui.separator();

        // Review Gate & Dispatch Section
        ui.label(RichText::new("Gatekeeper & Social Dispatch").strong().size(15.0));
        ui.horizontal(|ui| {
            ui.label("Dispatch to:");
            ui.checkbox(&mut self.dispatch_linkedin, "LinkedIn");
            ui.checkbox(&mut self.dispatch_x, "X");
            ui.checkbox(&mut self.dispatch_reddit, "Reddit");
            ui.checkbox(&mut self.dispatch_discord, "Discord");
            ui.checkbox(&mut self.dispatch_facebook, "Facebook");
            ui.checkbox(&mut self.dispatch_github, "GitHub");
        });

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button(RichText::new("💾 Save Edits to SQLite").strong()).clicked() {
                self.drafts.visual_asset = if self.visual_asset.trim().is_empty() {
                    None
                } else {
                    Some(self.visual_asset.clone())
                };
                action = Some(PreviewAction::SaveEdits(draft_id, self.drafts.clone()));
            }

            ui.add_space(12.0);

            if is_dispatching {
                ui.spinner();
                ui.label(RichText::new("Dispatching to social networks...").color(Color32::from_rgb(100, 180, 255)));
            } else {
                let btn_text = if dry_run {
                    "🧪 Dry Run Dispatch"
                } else {
                    "🚀 Dispatch to Socials (Live)"
                };
                let btn = ui.add_sized(
                    [220.0, 32.0],
                    egui::Button::new(RichText::new(btn_text).strong().size(14.0)),
                );
                if btn.clicked() {
                    let mut targets = Vec::new();
                    if self.dispatch_linkedin {
                        targets.push("linkedin".to_string());
                    }
                    if self.dispatch_x {
                        targets.push("x".to_string());
                    }
                    if self.dispatch_reddit {
                        targets.push("reddit".to_string());
                    }
                    if self.dispatch_discord {
                        targets.push("discord".to_string());
                    }
                    if self.dispatch_facebook {
                        targets.push("facebook".to_string());
                    }
                    if self.dispatch_github {
                        targets.push("github".to_string());
                    }

                    action = Some(PreviewAction::Dispatch(
                        draft_id,
                        targets,
                        self.visual_asset.clone(),
                    ));
                }
            }
        });

        // Audit Log Cards
        if !self.audit_results.is_empty() {
            ui.add_space(10.0);
            ui.label(RichText::new("Audit Log:").strong());
            ui.group(|ui| {
                for item in &self.audit_results {
                    ui.horizontal(|ui| {
                        let (badge_color, badge_text) = match item.state.as_str() {
                            "ok" => (Color32::from_rgb(76, 175, 80), "OK"),
                            "failed" => (Color32::from_rgb(239, 83, 80), "FAILED"),
                            "skipped" => (Color32::from_rgb(158, 158, 158), "SKIPPED"),
                            "dry" => (Color32::from_rgb(255, 179, 0), "DRY RUN"),
                            _ => (Color32::GRAY, item.state.as_str()),
                        };
                        ui.colored_label(badge_color, RichText::new(format!("[{badge_text}]")).strong());
                        ui.label(RichText::new(&item.platform).strong());
                        ui.label(&item.detail);
                    });
                }
            });
        }

        action
    }
}
