use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;

use rusqlite::Connection;

use crate::models::{DispatchResult, DoctorReport, DraftRecord, Drafts, RunParams, RunResponse, SendResponse};

/// Locates the appropriate Python binary (virtualenv preferred).
pub fn find_python_binary() -> PathBuf {
    let candidates = [
        PathBuf::from("venv/bin/python"),
        PathBuf::from("../venv/bin/python"),
        PathBuf::from("venv/Scripts/python.exe"),
        PathBuf::from("../venv/Scripts/python.exe"),
        PathBuf::from(".venv/bin/python"),
        PathBuf::from("../.venv/bin/python"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    PathBuf::from("python3")
}

/// Locates the SQLite history database path.
pub fn find_storage_db() -> PathBuf {
    let candidates = [
        PathBuf::from("storage/history.sqlite"),
        PathBuf::from("../storage/history.sqlite"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    PathBuf::from("storage/history.sqlite")
}

/// Reads the root project directory.
pub fn find_project_root() -> PathBuf {
    if Path::new("pyproject.toml").exists() {
        return PathBuf::from(".");
    }
    if Path::new("../pyproject.toml").exists() {
        return PathBuf::from("..");
    }
    PathBuf::from(".")
}

/// Queries SQLite directly for ultra-fast, zero-overhead history access.
pub fn load_history_direct() -> Result<Vec<DraftRecord>, String> {
    let db_path = find_storage_db();
    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.repo, d.problem, d.solution, d.drafts_json, d.status, d.created_at,
                    (SELECT group_concat(platform) FROM posts p WHERE p.draft_id = d.id AND p.ok = 1) AS platforms
             FROM drafts d ORDER BY d.id DESC LIMIT 50",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let repo: String = row.get(1)?;
            let problem: Option<String> = row.get(2)?;
            let solution: Option<String> = row.get(3)?;
            let drafts_json: String = row.get(4)?;
            let status: String = row.get(5)?;
            let created_at: String = row.get(6)?;
            let platforms: Option<String> = row.get(7)?;

            let drafts: Drafts = serde_json::from_str(&drafts_json).unwrap_or_default();

            Ok(DraftRecord {
                id,
                repo,
                problem,
                solution,
                drafts,
                status,
                created_at,
                platforms,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut results = Vec::new();
    for r in rows {
        if let Ok(record) = r {
            results.push(record);
        }
    }

    Ok(results)
}

/// Directly updates the drafts JSON in SQLite when the user edits in the UI.
pub fn save_draft_edits(draft_id: i64, drafts: &Drafts) -> Result<(), String> {
    let db_path = find_storage_db();
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    let json_str = serde_json::to_string(drafts).map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE drafts SET drafts_json = ? WHERE id = ?",
        rusqlite::params![json_str, draft_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Asynchronously runs `larpflow doctor --json`.
pub fn run_doctor_async(tx: mpsc::Sender<Result<DoctorReport, String>>) {
    thread::spawn(move || {
        let py = find_python_binary();
        let root = find_project_root();

        let output = Command::new(&py)
            .current_dir(&root)
            .args(["-m", "larpflow.core.cli", "doctor", "--json"])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    match serde_json::from_str::<DoctorReport>(&text) {
                        Ok(report) => {
                            let _ = tx.send(Ok(report));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(format!("JSON Parse error: {e}\nRaw output: {text}")));
                        }
                    }
                } else {
                    let err = String::from_utf8_lossy(&out.stderr);
                    let _ = tx.send(Err(format!("Doctor command failed: {err}")));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(format!("Failed to execute python ({:?}): {e}", py)));
            }
        }
    });
}

/// Asynchronously generates drafts using the Python narrative engine.
pub fn run_generate_async(params: RunParams, tx: mpsc::Sender<Result<RunResponse, String>>) {
    thread::spawn(move || {
        let py = find_python_binary();
        let root = find_project_root();

        let mut args = vec![
            "-m".to_string(),
            "larpflow.core.cli".to_string(),
            "run".to_string(),
            params.repo,
            "--commits".to_string(),
            params.commits.to_string(),
            "--type".to_string(),
            params.focus_type,
            "--json".to_string(),
            "--no-open".to_string(),
        ];

        if !params.problem.trim().is_empty() {
            args.push("--problem".to_string());
            args.push(params.problem);
        }

        if !params.solution.trim().is_empty() {
            args.push("--solution".to_string());
            args.push(params.solution);
        }

        if !params.image_path.trim().is_empty() {
            args.push("--image".to_string());
            args.push(params.image_path);
        }

        if !params.target_platforms.is_empty() {
            args.push("--platforms".to_string());
            args.push(params.target_platforms.join(","));
        }

        let output = Command::new(&py).current_dir(&root).args(&args).output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if out.status.success() {
                    // In case of any leading logs, find the first '{' and last '}'
                    if let (Some(start), Some(end)) = (stdout.find('{'), stdout.rfind('}')) {
                        let json_slice = &stdout[start..=end];
                        match serde_json::from_str::<RunResponse>(json_slice) {
                            Ok(res) => {
                                let _ = tx.send(Ok(res));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(format!("Failed to parse response: {e}\nOutput: {stdout}")));
                            }
                        }
                    } else {
                        let _ = tx.send(Err(format!("No JSON returned in stdout:\n{stdout}\nStderr: {stderr}")));
                    }
                } else {
                    let _ = tx.send(Err(format!("Generation failed (code {:?}):\n{stderr}\n{stdout}", out.status.code())));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(format!("Failed to spawn Python process ({:?}): {e}", py)));
            }
        }
    });
}

/// Asynchronously dispatches an approved draft to social platforms.
pub fn run_dispatch_async(
    draft_id: i64,
    platforms: Vec<String>,
    image_path: String,
    tx: mpsc::Sender<Result<Vec<DispatchResult>, String>>,
) {
    thread::spawn(move || {
        let py = find_python_binary();
        let root = find_project_root();

        let mut args = vec![
            "-m".to_string(),
            "larpflow.core.cli".to_string(),
            "send".to_string(),
            draft_id.to_string(),
            "--yes".to_string(),
            "--json".to_string(),
        ];

        if !platforms.is_empty() {
            args.push("--platforms".to_string());
            args.push(platforms.join(","));
        }

        if !image_path.trim().is_empty() {
            args.push("--image".to_string());
            args.push(image_path);
        }

        let output = Command::new(&py).current_dir(&root).args(&args).output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);

                if out.status.success() {
                    if let (Some(start), Some(end)) = (stdout.find('{'), stdout.rfind('}')) {
                        let json_slice = &stdout[start..=end];
                        match serde_json::from_str::<SendResponse>(json_slice) {
                            Ok(res) => {
                                let _ = tx.send(Ok(res.results));
                            }
                            Err(e) => {
                                let _ = tx.send(Err(format!("Failed to parse send response: {e}\nOutput: {stdout}")));
                            }
                        }
                    } else {
                        let _ = tx.send(Err(format!("No JSON returned in send output:\n{stdout}\n{stderr}")));
                    }
                } else {
                    let _ = tx.send(Err(format!("Send failed: {stderr}\n{stdout}")));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(format!("Failed to spawn Python ({:?}): {e}", py)));
            }
        }
    });
}
