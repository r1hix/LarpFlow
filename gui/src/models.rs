use serde::{Deserialize, Serialize};

/// Platform-specific generated content matching LarpFlow's Python Drafts model.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Drafts {
    #[serde(default)]
    pub tweets: Vec<String>,
    #[serde(default)]
    pub reddit_subreddit: String,
    #[serde(default)]
    pub reddit_title: String,
    #[serde(default)]
    pub reddit_body: String,
    #[serde(default)]
    pub suggested_subreddits: Vec<String>,
    #[serde(default)]
    pub linkedin: String,
    #[serde(default)]
    pub facebook: String,
    #[serde(default)]
    pub discord_title: String,
    #[serde(default)]
    pub discord_body: String,
    #[serde(default)]
    pub github_readme_section: String,
    #[serde(default)]
    pub visual_asset: Option<String>,
}

/// A stored draft record retrieved from SQLite or CLI show/history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRecord {
    pub id: i64,
    pub repo: String,
    #[serde(default)]
    pub problem: Option<String>,
    #[serde(default)]
    pub solution: Option<String>,
    pub drafts: Drafts,
    pub status: String,
    pub created_at: String,
    #[serde(default)]
    pub platforms: Option<String>,
}

/// Diagnostic health status returned by `larpflow doctor --json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DoctorReport {
    pub dry_run: bool,
    pub llm_api_key: bool,
    pub llm_model: String,
    pub github_token: bool,
    pub telegram_bot_token: bool,
    pub telegram_owner_id: bool,
    pub x: bool,
    pub reddit: bool,
    pub reddit_allowed_subs: Vec<String>,
    pub linkedin: bool,
    pub facebook: bool,
    pub discord: bool,
    pub linkedin_profile: bool,
}

/// Result of posting to a single platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchResult {
    pub platform: String,
    pub state: String, // "ok", "failed", "skipped", "dry"
    pub detail: String,
}

/// Structured response from `larpflow run ... --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResponse {
    pub draft_id: i64,
    pub repo: String,
    pub drafts: Drafts,
    #[serde(default)]
    pub problem: String,
    #[serde(default)]
    pub solution: String,
    #[serde(default)]
    pub file: String,
    #[serde(default)]
    pub audit: Option<Vec<DispatchResult>>,
}

/// Structured response from `larpflow send ... --json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendResponse {
    pub draft_id: i64,
    pub results: Vec<DispatchResult>,
}

/// Generation parameters passed from the UI to the Python engine.
#[derive(Debug, Clone)]
pub struct RunParams {
    pub repo: String,
    pub commits: usize,
    pub focus_type: String,
    pub problem: String,
    pub solution: String,
    pub image_path: String,
    pub target_platforms: Vec<String>,
}

impl Default for RunParams {
    fn default() -> Self {
        Self {
            repo: ".".to_string(),
            commits: 10,
            focus_type: "auto".to_string(),
            problem: String::new(),
            solution: String::new(),
            image_path: String::new(),
            target_platforms: vec![
                "x".to_string(),
                "linkedin".to_string(),
                "reddit".to_string(),
                "discord".to_string(),
            ],
        }
    }
}
