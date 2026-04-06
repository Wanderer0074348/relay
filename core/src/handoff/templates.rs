//! Handoff templates — control what context is included.

use crate::SessionSnapshot;

/// Available handoff templates.
pub enum Template {
    Full,     // Everything — conversation, git, todos, errors, decisions
    Minimal,  // Just task + error + git status
    Raw,      // Only the conversation turns, no formatting
}

impl Template {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "minimal" | "min" => Self::Minimal,
            "raw" | "conversation" => Self::Raw,
            _ => Self::Full,
        }
    }
}

/// Build handoff using the minimal template — task + error + git only.
pub fn build_minimal(snapshot: &SessionSnapshot, target: &str) -> String {
    let mut out = format!(
        "══ RELAY HANDOFF (minimal) ═════════════════\n\
        Agent: {target} | Project: {}\n\
        ═══════════════════════════════════════════\n\n",
        snapshot.project_dir
    );

    out.push_str(&format!("Task: {}\n\n", snapshot.current_task));

    if let Some(ref err) = snapshot.last_error {
        out.push_str(&format!("Error: {}\n\n", truncate(err, 300)));
    }

    if let Some(ref git) = snapshot.git_state {
        out.push_str(&format!("Branch: {} | {}\n", git.branch, git.status_summary));
        if !git.recent_commits.is_empty() {
            out.push_str(&format!("Last commit: {}\n", git.recent_commits[0]));
        }
    }

    out.push_str("\nContext restored. What would you like me to do?\n");
    out
}

/// Build handoff using raw template — just conversation turns.
pub fn build_raw(snapshot: &SessionSnapshot) -> String {
    let mut out = String::new();
    for turn in &snapshot.conversation {
        let prefix = match turn.role.as_str() {
            "user"           => "USER",
            "assistant"      => "ASSISTANT",
            "assistant_tool" => "TOOL_CALL",
            "tool_result"    => "TOOL_OUTPUT",
            _                => &turn.role,
        };
        out.push_str(&format!("[{prefix}] {}\n\n", turn.content));
    }
    if out.is_empty() {
        out.push_str("(no conversation captured)\n");
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) { end -= 1; }
    format!("{}...", &s[..end])
}
