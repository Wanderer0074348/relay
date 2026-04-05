//! `relay resume` — When Claude comes back, show what happened during the handoff.
//! Reads the latest handoff file + git diff since handoff time → generates a
//! "welcome back" prompt so Claude (or any agent) can pick up with full awareness
//! of what the fallback agent did.

use anyhow::{Context, Result};
use std::path::Path;

#[derive(Debug, serde::Serialize)]
pub struct ResumeReport {
    pub handoff_file: String,
    pub handoff_time: String,
    pub original_task: String,
    pub changes_since: Vec<String>,
    pub new_commits: Vec<String>,
    pub diff_stat: String,
    pub resume_prompt: String,
}

/// Build a resume report from the latest handoff.
pub fn build_resume(project_dir: &Path) -> Result<ResumeReport> {
    let relay_dir = project_dir.join(".relay");
    if !relay_dir.exists() {
        anyhow::bail!("No .relay/ directory found. Run 'relay handoff' first.");
    }

    // Find latest handoff file
    let handoff_file = find_latest_handoff(&relay_dir)?;
    let handoff_content = std::fs::read_to_string(&handoff_file)
        .context("Failed to read handoff file")?;

    // Extract timestamp from filename: handoff_20260405_141328.md
    let filename = handoff_file
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let handoff_time = parse_handoff_timestamp(&filename);

    // Extract original task from handoff content
    let original_task = extract_section(&handoff_content, "## CURRENT TASK")
        .unwrap_or_else(|| "Unknown".into());

    // Git changes since handoff
    let changes_since = git_changed_files(project_dir, &handoff_time);
    let new_commits = git_commits_since(project_dir, &handoff_time);
    let diff_stat = git_diff_stat(project_dir, &handoff_time);

    // Build resume prompt
    let resume_prompt = build_resume_prompt(
        &original_task,
        &changes_since,
        &new_commits,
        &diff_stat,
        &handoff_time,
    );

    Ok(ResumeReport {
        handoff_file: handoff_file.to_string_lossy().to_string(),
        handoff_time,
        original_task,
        changes_since,
        new_commits,
        diff_stat,
        resume_prompt,
    })
}

fn find_latest_handoff(relay_dir: &Path) -> Result<std::path::PathBuf> {
    let mut latest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
    for entry in std::fs::read_dir(relay_dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with("handoff_") && name.ends_with(".md") {
            if let Ok(meta) = path.metadata() {
                if let Ok(modified) = meta.modified() {
                    if latest.as_ref().map_or(true, |(_, t)| modified > *t) {
                        latest = Some((path, modified));
                    }
                }
            }
        }
    }
    latest
        .map(|(p, _)| p)
        .ok_or_else(|| anyhow::anyhow!("No handoff files found in .relay/"))
}

fn parse_handoff_timestamp(filename: &str) -> String {
    // handoff_20260405_141328.md → 2026-04-05 14:13:28
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
        "unknown".into()
    }
}

fn extract_section(content: &str, header: &str) -> Option<String> {
    let start = content.find(header)?;
    let after = &content[start + header.len()..];
    let end = after.find("\n## ").unwrap_or(after.len());
    let section = after[..end].trim();
    if section.is_empty() { None } else { Some(section.to_string()) }
}

fn git_changed_files(dir: &Path, _since: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .current_dir(dir)
        .args(["diff", "--name-only", "HEAD"])
        .output()
        .ok();
    output
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn git_commits_since(dir: &Path, since: &str) -> Vec<String> {
    let output = std::process::Command::new("git")
        .current_dir(dir)
        .args(["log", "--oneline", "-10", "--since", since])
        .output()
        .ok();
    output
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn git_diff_stat(dir: &Path, _since: &str) -> String {
    let output = std::process::Command::new("git")
        .current_dir(dir)
        .args(["diff", "--stat", "HEAD"])
        .output()
        .ok();
    output
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn build_resume_prompt(
    task: &str,
    changed: &[String],
    commits: &[String],
    diff_stat: &str,
    handoff_time: &str,
) -> String {
    let mut prompt = format!(
        "══ RELAY RESUME ══════════════════════════════\n\
        You are resuming a session that was handed off at {handoff_time}.\n\
        A fallback agent continued the work. Here's what happened:\n\
        ══════════════════════════════════════════════\n\n\
        ## ORIGINAL TASK\n\n{task}\n"
    );

    if !commits.is_empty() {
        prompt.push_str("\n## COMMITS MADE DURING HANDOFF\n\n");
        for c in commits {
            prompt.push_str(&format!("  {c}\n"));
        }
    }

    if !changed.is_empty() {
        prompt.push_str(&format!("\n## FILES CHANGED ({})\n\n", changed.len()));
        for f in changed.iter().take(20) {
            prompt.push_str(&format!("  {f}\n"));
        }
    }

    if !diff_stat.is_empty() {
        prompt.push_str(&format!("\n## DIFF SUMMARY\n\n{diff_stat}\n"));
    }

    prompt.push_str("\n## INSTRUCTIONS\n\n\
        Review what the fallback agent did above.\n\
        Continue from the current state. Check if the original task is complete.\n\
        If not, finish it.\n");

    prompt
}
