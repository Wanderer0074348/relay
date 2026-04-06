use std::path::PathBuf;

#[test]
fn capture_snapshot_works_without_claude_dir() {
    let dir = std::env::temp_dir().join("relay_test_no_claude");
    std::fs::create_dir_all(&dir).unwrap();

    // Init a git repo so git capture doesn't fail
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["init", "-q"])
        .output()
        .unwrap();
    std::fs::write(dir.join("test.txt"), "hello").unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["add", "."])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["commit", "-q", "-m", "init"])
        .output()
        .unwrap();

    let snapshot = relay::capture::capture_snapshot(&dir, None).unwrap();
    assert!(!snapshot.project_dir.is_empty());
    assert!(!snapshot.timestamp.is_empty());
    assert!(snapshot.git_state.is_some());

    let git = snapshot.git_state.unwrap();
    assert!(!git.branch.is_empty());

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn capture_snapshot_with_deadline() {
    let dir = std::env::temp_dir().join("relay_test_deadline");
    std::fs::create_dir_all(&dir).unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["init", "-q"])
        .output()
        .unwrap();
    std::fs::write(dir.join("x.txt"), "x").unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["add", "."])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["commit", "-q", "-m", "init"])
        .output()
        .unwrap();

    let snapshot = relay::capture::capture_snapshot(&dir, Some("7:00 PM")).unwrap();
    assert_eq!(snapshot.deadline.as_deref(), Some("7:00 PM"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn capture_git_state() {
    let dir = std::env::temp_dir().join("relay_test_git");
    std::fs::create_dir_all(&dir).unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["init", "-q"])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["checkout", "-b", "test-branch"])
        .output()
        .unwrap();
    std::fs::write(dir.join("file.rs"), "fn main() {}").unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["add", "."])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["commit", "-q", "-m", "test commit"])
        .output()
        .unwrap();

    let git = relay::capture::git::capture_git_state(&dir).unwrap();
    assert_eq!(git.branch, "test-branch");
    assert!(git.status_summary.contains("Clean"));
    assert!(!git.recent_commits.is_empty());
    assert!(git.recent_commits[0].contains("test commit"));

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn capture_uncommitted_files() {
    let dir = std::env::temp_dir().join("relay_test_uncommitted");
    std::fs::create_dir_all(&dir).unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["init", "-q"])
        .output()
        .unwrap();
    std::fs::write(dir.join("committed.txt"), "a").unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["add", "."])
        .output()
        .unwrap();
    std::process::Command::new("git")
        .current_dir(&dir)
        .args(["commit", "-q", "-m", "init"])
        .output()
        .unwrap();

    // Add uncommitted file
    std::fs::write(dir.join("new_file.txt"), "uncommitted").unwrap();

    let git = relay::capture::git::capture_git_state(&dir).unwrap();
    assert!(git.uncommitted_files.iter().any(|f| f.contains("new_file")));
    assert!(git.status_summary.contains("uncommitted"));

    std::fs::remove_dir_all(&dir).ok();
}
