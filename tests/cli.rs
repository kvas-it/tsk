use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a temp directory with tsk.yaml and a tsk/ subdir.
/// Each call gets a unique directory.
fn setup_project() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir()
        .join(format!("tsk-test-{}-{}", std::process::id(), n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("tsk")).unwrap();
    fs::write(dir.join("tsk.yaml"), "").unwrap();
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn tsk_bin() -> PathBuf {
    // Cargo puts the binary in target/debug/tsk during `cargo test`.
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove `deps`
    path.push("tsk");
    path
}

fn run(dir: &Path, args: &[&str]) -> (String, String, bool) {
    let output = Command::new(tsk_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run tsk");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

fn write_ticket(dir: &Path, num: u32, content: &str) {
    let path = dir.join("tsk").join(format!("{:03}.md", num));
    fs::write(path, content).unwrap();
}

// --- tsk new ---

#[test]
fn new_creates_ticket() {
    let dir = setup_project();
    let (stdout, _, ok) = run(&dir, &["new", "First ticket"]);
    assert!(ok);
    assert_eq!(stdout.trim(), "001.md");

    let content = fs::read_to_string(dir.join("tsk/001.md")).unwrap();
    assert!(content.contains("status: open"));
    assert!(content.contains("# First ticket"));

    cleanup(&dir);
}

#[test]
fn new_increments_number() {
    let dir = setup_project();
    write_ticket(&dir, 3, "---\nstatus: open\n---\n\n# Three\n");
    let (stdout, _, ok) = run(&dir, &["new", "Four"]);
    assert!(ok);
    assert_eq!(stdout.trim(), "004.md");
    cleanup(&dir);
}

#[test]
fn new_with_parent() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Parent\n");
    let (stdout, _, ok) = run(&dir, &["new", "--parent", "1", "Child"]);
    assert!(ok);
    assert_eq!(stdout.trim(), "002.md");

    let content = fs::read_to_string(dir.join("tsk/002.md")).unwrap();
    assert!(content.contains("parent: 1"));
    assert!(content.contains("# Child"));

    cleanup(&dir);
}

#[test]
fn new_with_parent_short_flag() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Parent\n");
    let (stdout, _, ok) = run(&dir, &["new", "-p", "1", "Child"]);
    assert!(ok);
    assert_eq!(stdout.trim(), "002.md");
    cleanup(&dir);
}

// --- tsk spawn ---

#[test]
fn spawn_creates_child_with_date_in_title() {
    let dir = setup_project();
    write_ticket(
        &dir, 1,
        "---\nstatus: template\n---\n\n# Weekly audit\n",
    );
    let (stdout, _, ok) = run(&dir, &["spawn", "1"]);
    assert!(ok);
    assert_eq!(stdout.trim(), "002.md");

    let content = fs::read_to_string(dir.join("tsk/002.md")).unwrap();
    assert!(content.contains("parent: 1"));
    assert!(content.contains("status: open"));
    // Title should contain the template title and a date.
    assert!(content.contains("Weekly audit ("));
    // Check it has a date-like pattern (YYYY-MM-DD).
    assert!(content.contains("202"));

    cleanup(&dir);
}

#[test]
fn spawn_nonexistent_ticket_fails() {
    let dir = setup_project();
    let (_, stderr, ok) = run(&dir, &["spawn", "99"]);
    assert!(!ok);
    assert!(stderr.contains("not found"));
    cleanup(&dir);
}

#[test]
fn new_with_nonexistent_parent_fails() {
    let dir = setup_project();
    let (_, stderr, ok) = run(&dir, &["new", "--parent", "99", "Orphan"]);
    assert!(!ok);
    assert!(stderr.contains("not found"));
    cleanup(&dir);
}

// --- tsk list ---

#[test]
fn list_flat() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Alpha\n");
    write_ticket(&dir, 2, "---\nstatus: done\n---\n\n# Beta\n");

    let (stdout, _, ok) = run(&dir, &["list"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("001"));
    assert!(lines[0].contains("open"));
    assert!(lines[0].contains("Alpha"));
    assert!(lines[1].contains("002"));
    assert!(lines[1].contains("done"));
    assert!(lines[1].contains("Beta"));

    cleanup(&dir);
}

#[test]
fn list_with_status_filter() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Alpha\n");
    write_ticket(&dir, 2, "---\nstatus: done\n---\n\n# Beta\n");

    let (stdout, _, ok) = run(&dir, &["list", "open"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("Alpha"));

    cleanup(&dir);
}

#[test]
fn list_with_negative_filter() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Alpha\n");
    write_ticket(&dir, 2, "---\nstatus: done\n---\n\n# Beta\n");

    let (stdout, _, ok) = run(&dir, &["list", "-done"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("Alpha"));

    cleanup(&dir);
}

#[test]
fn list_tree_view() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Parent\n");
    write_ticket(
        &dir, 2,
        "---\nstatus: open\nparent: 1\n---\n\n# Child\n",
    );
    write_ticket(
        &dir, 3,
        "---\nstatus: open\nparent: 1\n---\n\n# Child 2\n",
    );

    let (stdout, _, ok) = run(&dir, &["list"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    // Parent at top level (no indent).
    assert!(lines[0].starts_with("001"));
    // Children indented.
    assert!(lines[1].starts_with("  002"));
    assert!(lines[2].starts_with("  003"));

    cleanup(&dir);
}

#[test]
fn list_deep_nesting() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Root\n");
    write_ticket(
        &dir, 2,
        "---\nstatus: open\nparent: 1\n---\n\n# Level 1\n",
    );
    write_ticket(
        &dir, 3,
        "---\nstatus: open\nparent: 2\n---\n\n# Level 2\n",
    );

    let (stdout, _, ok) = run(&dir, &["list"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("001"));
    assert!(lines[1].starts_with("  002"));
    assert!(lines[2].starts_with("    003"));

    cleanup(&dir);
}

#[test]
fn list_filtered_ancestor_shown_with_marker() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: done\n---\n\n# Parent\n");
    write_ticket(
        &dir, 2,
        "---\nstatus: open\nparent: 1\n---\n\n# Child\n",
    );

    let (stdout, _, ok) = run(&dir, &["list", "open"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2);
    // Parent shown with ".." marker.
    assert!(lines[0].contains("001"));
    assert!(lines[0].contains(".."));
    // Child shown normally.
    assert!(lines[1].contains("002"));
    assert!(!lines[1].contains(".."));

    cleanup(&dir);
}

#[test]
fn list_orphan_at_top_level() {
    let dir = setup_project();
    // Child references parent 99 which doesn't exist.
    write_ticket(
        &dir, 1,
        "---\nstatus: open\nparent: 99\n---\n\n# Orphan\n",
    );

    let (stdout, _, ok) = run(&dir, &["list"]);
    assert!(ok);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    // Shown at top level, no indent.
    assert!(lines[0].starts_with("001"));

    cleanup(&dir);
}

// --- tsk show ---

#[test]
fn show_basic() {
    let dir = setup_project();
    write_ticket(
        &dir, 1,
        "---\nstatus: open\n---\n\n# My Ticket\n\nSome body text.\n",
    );

    let (stdout, _, ok) = run(&dir, &["show", "1"]);
    assert!(ok);
    assert!(stdout.contains("[open]"));
    assert!(stdout.contains("My Ticket"));
    assert!(stdout.contains("Some body text."));

    cleanup(&dir);
}

#[test]
fn show_displays_children() {
    let dir = setup_project();
    write_ticket(
        &dir, 1,
        "---\nstatus: open\n---\n\n# Parent\n\nBody.\n",
    );
    write_ticket(
        &dir, 2,
        "---\nstatus: open\nparent: 1\n---\n\n# Child A\n",
    );
    write_ticket(
        &dir, 3,
        "---\nstatus: done\nparent: 1\n---\n\n# Child B\n",
    );

    let (stdout, _, ok) = run(&dir, &["show", "1"]);
    assert!(ok);
    assert!(stdout.contains("Sub-tasks:"));
    assert!(stdout.contains("Child A"));
    assert!(stdout.contains("Child B"));

    cleanup(&dir);
}

#[test]
fn show_displays_parent_chain() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Root\n");
    write_ticket(
        &dir, 2,
        "---\nstatus: open\nparent: 1\n---\n\n# Child\n",
    );

    let (stdout, _, ok) = run(&dir, &["show", "2"]);
    assert!(ok);
    // Parent chain should show root above the child.
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].contains("001"));
    assert!(lines[0].contains("Root"));
    assert!(lines[1].contains("002"));
    assert!(lines[1].contains("[open]"));
    assert!(lines[1].contains("Child"));

    cleanup(&dir);
}

// --- tsk status ---

#[test]
fn status_change() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Ticket\n");

    let (stdout, _, ok) = run(&dir, &["status", "1", "done"]);
    assert!(ok);
    assert!(stdout.contains("open -> done"));

    let content = fs::read_to_string(dir.join("tsk/001.md")).unwrap();
    assert!(content.contains("status: done"));

    cleanup(&dir);
}

#[test]
fn status_invalid_fails() {
    let dir = setup_project();
    write_ticket(&dir, 1, "---\nstatus: open\n---\n\n# Ticket\n");

    let (_, stderr, ok) = run(&dir, &["status", "1", "bogus"]);
    assert!(!ok);
    assert!(stderr.contains("unknown status"));

    cleanup(&dir);
}

// --- tsk init ---

#[test]
fn init_creates_project() {
    let dir = std::env::temp_dir()
        .join(format!("tsk-init-{}", COUNTER.fetch_add(1, Ordering::SeqCst)));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let (stdout, _, ok) = run(&dir, &["init"]);
    assert!(ok);
    assert!(stdout.contains("Initialized"));
    assert!(dir.join("tsk.yaml").exists());
    assert!(dir.join("tsk").is_dir());

    cleanup(&dir);
}

#[test]
fn init_idempotent_with_yaml() {
    let dir = setup_project();
    let (_, stderr, ok) = run(&dir, &["init"]);
    assert!(ok);
    assert!(stderr.contains("already initialized"));
    cleanup(&dir);
}

#[test]
fn init_idempotent_with_dir_only() {
    let dir = std::env::temp_dir()
        .join(format!("tsk-init-{}", COUNTER.fetch_add(1, Ordering::SeqCst)));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("tsk")).unwrap();
    // No tsk.yaml, just the directory.

    let (_, stderr, ok) = run(&dir, &["init"]);
    assert!(ok);
    assert!(stderr.contains("already initialized"));

    cleanup(&dir);
}

// --- tsk (no args) ---

#[test]
fn no_args_shows_usage() {
    let dir = setup_project();
    let (_, stderr, ok) = run(&dir, &[]);
    assert!(ok);
    assert!(stderr.contains("usage:"));
    cleanup(&dir);
}

// --- tsk list (empty) ---

#[test]
fn list_empty_project() {
    let dir = setup_project();
    let (stdout, _, ok) = run(&dir, &["list"]);
    assert!(ok);
    assert!(stdout.is_empty());
    cleanup(&dir);
}
