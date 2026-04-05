//! `relay diff` — Show what changed since the last handoff.

use anyhow::Result;
use std::path::Path;

#[derive(Debug, serde::Serialize)]
pub struct DiffReport {
    pub handoff_time: String,
    pub changed_files: Vec<String>,
    pub new_commits: Vec<String>,
    pub diff_stat: String,
    pub files_added: usize,
    pub files_modified: usize,
    pub files_deleted: usize,
}

/// Show what changed since the last handoff.
pub fn diff_since_handoff(project_dir: &Path) -> Result<DiffReport> {
    let relay_dir = project_dir.join(".relay");
    if !relay_dir.exists() {
        anyhow::bail!("No .relay/ directory found. Run 'relay handoff' first.");
    }

    // Find latest handoff timestamp
    let mut latest_time = String::new();
    let mut latest_modified = std::time::SystemTime::UNIX_EPOCH;

    for entry in std::fs::read_dir(&relay_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("handoff_") && name.ends_with(".md") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified > latest_modified {
                        latest_modified = modified;
                        latest_time = parse_timestamp(&name);
                    }
                }
            }
        }
    }

    if latest_time.is_empty() {
        anyhow::bail!("No handoff files found.");
    }

    // Git status
    let status_output = run_git(project_dir, &["status", "--short"]);
    let changed_files: Vec<String> = status_output
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    let mut files_added = 0;
    let mut files_modified = 0;
    let mut files_deleted = 0;
    for f in &changed_files {
        let prefix = f.trim().chars().next().unwrap_or(' ');
        match prefix {
            'A' | '?' => files_added += 1,
            'M' => files_modified += 1,
            'D' => files_deleted += 1,
            _ => files_modified += 1,
        }
    }

    // Commits since handoff
    let new_commits: Vec<String> = run_git(
        project_dir,
        &["log", "--oneline", "-10", "--since", &latest_time],
    )
    .lines()
    .filter(|l| !l.is_empty())
    .map(String::from)
    .collect();

    // Diff stat
    let diff_stat = run_git(project_dir, &["diff", "--stat"]);

    Ok(DiffReport {
        handoff_time: latest_time,
        changed_files,
        new_commits,
        diff_stat: diff_stat.trim().to_string(),
        files_added,
        files_modified,
        files_deleted,
    })
}

fn run_git(dir: &Path, args: &[&str]) -> String {
    std::process::Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

fn parse_timestamp(filename: &str) -> String {
    let ts = filename
        .strip_prefix("handoff_")
        .unwrap_or("")
        .strip_suffix(".md")
        .unwrap_or("");
    if ts.len() >= 15 {
        format!(
            "{}-{}-{} {}:{}:{}",
            &ts[0..4], &ts[4..6], &ts[6..8],
            &ts[9..11], &ts[11..13], &ts[13..15]
        )
    } else {
        ts.to_string()
    }
}
