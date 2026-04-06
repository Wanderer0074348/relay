use relay::{SessionSnapshot, TodoItem, GitState, ConversationTurn};

fn make_snapshot() -> SessionSnapshot {
    SessionSnapshot {
        current_task: "Fix the WebSocket handler".into(),
        todos: vec![
            TodoItem { content: "Database schema".into(), status: "completed".into() },
            TodoItem { content: "WebSocket handler".into(), status: "in_progress".into() },
            TodoItem { content: "Frontend charts".into(), status: "pending".into() },
        ],
        decisions: vec!["Using Socket.io".into(), "Redis pub/sub".into()],
        last_error: Some("error[E0499]: cannot borrow as mutable".into()),
        last_output: Some("cargo check failed".into()),
        git_state: Some(GitState {
            branch: "feature/websocket".into(),
            status_summary: "3 uncommitted changes".into(),
            recent_commits: vec!["abc1234 Add ws skeleton".into()],
            diff_summary: "3 files changed".into(),
            uncommitted_files: vec!["M src/ws.rs".into(), "M Cargo.toml".into()],
        }),
        project_dir: "/tmp/test-project".into(),
        recent_files: vec!["M src/ws.rs".into()],
        timestamp: "2026-04-05 18:20:00".into(),
        deadline: Some("7:00 PM".into()),
        conversation: vec![
            ConversationTurn { role: "user".into(), content: "fix the websocket".into() },
            ConversationTurn { role: "assistant".into(), content: "I'll wrap state in Arc".into() },
            ConversationTurn { role: "assistant_tool".into(), content: "[Edit] src/ws.rs".into() },
            ConversationTurn { role: "tool_result".into(), content: "File updated".into() },
        ],
    }
}

#[test]
fn handoff_contains_task() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("Fix the WebSocket handler"));
}

#[test]
fn handoff_contains_error() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("E0499"));
}

#[test]
fn handoff_contains_git() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("feature/websocket"));
    assert!(handoff.contains("abc1234"));
}

#[test]
fn handoff_contains_todos() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("Database schema"));
    assert!(handoff.contains("WebSocket handler"));
}

#[test]
fn handoff_contains_conversation() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("fix the websocket"));
    assert!(handoff.contains("Arc"));
}

#[test]
fn handoff_contains_deadline() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("DEADLINE") || handoff.contains("7:00 PM"));
}

#[test]
fn handoff_contains_decisions() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("Socket.io"));
}

#[test]
fn handoff_does_not_auto_execute() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    assert!(handoff.contains("DO NOT immediately start"));
    assert!(handoff.contains("WAIT for the user"));
}

#[test]
fn handoff_truncates_to_budget() {
    let snapshot = make_snapshot();
    let handoff = relay::handoff::build_handoff(&snapshot, "codex", 100).unwrap();
    // 100 tokens ~= 350 chars. Should be truncated.
    assert!(handoff.len() < 1000);
}

#[test]
fn handoff_save_creates_file() {
    let dir = std::env::temp_dir().join("relay_test_save");
    std::fs::create_dir_all(&dir).unwrap();

    let path = relay::handoff::save_handoff("test handoff content", &dir).unwrap();
    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "test handoff content");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn minimal_template_is_short() {
    let snapshot = make_snapshot();
    let full = relay::handoff::build_handoff(&snapshot, "codex", 8000).unwrap();
    let minimal = relay::handoff::templates::build_minimal(&snapshot, "codex");
    assert!(minimal.len() < full.len());
    assert!(minimal.contains("Fix the WebSocket"));
}

#[test]
fn raw_template_is_conversation_only() {
    let snapshot = make_snapshot();
    let raw = relay::handoff::templates::build_raw(&snapshot);
    assert!(raw.contains("[USER]"));
    assert!(raw.contains("[ASSISTANT]"));
    assert!(!raw.contains("## GIT STATE"));
}
