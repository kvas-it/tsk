use std::{env, fs, io::IsTerminal, path::{Path, PathBuf}, process::Command};

// --- Config ---

struct Config {
    dir: String,
    project: String,
    digits: usize,
    statuses: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dir: "./tsk".into(),
            project: "project.md".into(),
            digits: 3,
            statuses: vec![
                "open".into(),
                "in-progress".into(),
                "done".into(),
            ],
        }
    }
}

fn parse_config(content: &str) -> Config {
    let mut config = Config::default();
    let mut in_statuses = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            if in_statuses { in_statuses = false; }
            continue;
        }

        if in_statuses {
            if let Some(item) = trimmed.strip_prefix("- ") {
                config.statuses.push(item.trim().to_string());
                continue;
            }
            in_statuses = false;
        }

        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "dir" => config.dir = value.to_string(),
                "project" => config.project = value.to_string(),
                "digits" => {
                    if let Ok(d) = value.parse() {
                        config.digits = d;
                    }
                }
                "statuses" if value.is_empty() => {
                    in_statuses = true;
                    config.statuses.clear();
                }
                _ => {}
            }
        }
    }
    config
}

/// Walk up from the current directory looking for tsk.yaml.
/// Falls back to checking for a ./tsk/ directory.
fn find_project() -> (PathBuf, Config) {
    let mut dir = env::current_dir().expect("can't read current directory");
    loop {
        let config_path = dir.join("tsk.yaml");
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .expect("can't read tsk.yaml");
            return (dir, parse_config(&content));
        }
        if !dir.pop() {
            break;
        }
    }
    // No tsk.yaml — check for default ./tsk/ directory.
    let cwd = env::current_dir().unwrap();
    if cwd.join("tsk").is_dir() {
        return (cwd, Config::default());
    }
    eprintln!("error: no tsk project found (no tsk.yaml or ./tsk/)");
    std::process::exit(1);
}

// --- Tickets ---

struct Ticket {
    number: u32,
    title: String,
    status: String,
    parent: Option<u32>,
    path: PathBuf,
}

fn scan_tickets(dir: &Path, digits: usize) -> Vec<Ticket> {
    let mut tickets = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return tickets,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(stem) = name.strip_suffix(".md") else {
            continue;
        };
        if stem.len() != digits {
            continue;
        }
        let Ok(num) = stem.parse::<u32>() else {
            continue;
        };

        let content = fs::read_to_string(entry.path())
            .unwrap_or_default();
        let (status, title, parent) = parse_ticket(&content);

        tickets.push(Ticket {
            number: num,
            title,
            status,
            parent,
            path: entry.path(),
        });
    }
    tickets.sort_by_key(|t| t.number);
    tickets
}

/// Extract status, title, and parent from front matter and body.
fn parse_ticket(content: &str) -> (String, String, Option<u32>) {
    let mut status = "open".to_string();
    let mut title = String::new();
    let mut parent = None;
    let mut in_fm = false;
    let mut past_fm = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if in_fm {
                in_fm = false;
                past_fm = true;
            } else if !past_fm {
                in_fm = true;
            }
            continue;
        }
        if in_fm {
            if let Some(rest) = line.strip_prefix("status:") {
                status = rest.trim().to_string();
            }
            if let Some(rest) = line.strip_prefix("parent:") {
                parent = rest.trim().parse::<u32>().ok();
            }
        }
        if past_fm && title.is_empty() {
            if let Some(rest) = line.strip_prefix("# ") {
                title = rest.trim().to_string();
            }
        }
        if past_fm && !title.is_empty() {
            break;
        }
    }
    (status, title, parent)
}

fn next_number(tickets: &[Ticket]) -> u32 {
    tickets.iter().map(|t| t.number).max().unwrap_or(0) + 1
}

/// Find a ticket by number.
fn find_ticket(tickets: &[Ticket], num: u32) -> Option<&Ticket> {
    tickets.iter().find(|t| t.number == num)
}

/// Parse a ticket number from user input. Accepts "3", "03", "003".
fn parse_ticket_number(input: &str) -> Option<u32> {
    input.parse::<u32>().ok()
}

// --- Front matter manipulation ---

/// Replace (or insert) a field in YAML front matter, preserving
/// everything else in the file byte-for-byte.
fn set_frontmatter_field(content: &str, key: &str, value: &str)
    -> String
{
    let mut result = Vec::new();
    let mut in_fm = false;
    let mut found = false;
    let prefix = format!("{key}:");

    for line in content.lines() {
        if line.trim() == "---" {
            if in_fm && !found {
                // End of front matter, field wasn't there — insert it.
                result.push(format!("{key}: {value}"));
            }
            in_fm = !in_fm;
            result.push(line.to_string());
            continue;
        }
        if in_fm && line.starts_with(&prefix) {
            result.push(format!("{key}: {value}"));
            found = true;
        } else {
            result.push(line.to_string());
        }
    }

    let mut out = result.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    out
}

// --- Filters ---

/// A status filter: either include matching or exclude matching.
enum StatusFilter {
    Include(Vec<String>),
    Exclude(Vec<String>),
}

/// Parse filter args. A leading `-` means exclude, otherwise include.
/// Supports mixing: `tsk list open in-progress` or `tsk list -done`.
fn parse_filters(args: &[String]) -> Option<StatusFilter> {
    if args.is_empty() {
        return None;
    }
    let has_negated = args.iter().any(|a| a.starts_with('-'));
    let has_positive = args.iter().any(|a| !a.starts_with('-'));

    if has_negated && has_positive {
        eprintln!(
            "error: can't mix positive and negative status filters"
        );
        std::process::exit(1);
    }

    if has_negated {
        let statuses: Vec<String> = args.iter()
            .map(|a| a.strip_prefix('-').unwrap().to_string())
            .collect();
        Some(StatusFilter::Exclude(statuses))
    } else {
        let statuses: Vec<String> = args.iter()
            .map(|a| a.to_string())
            .collect();
        Some(StatusFilter::Include(statuses))
    }
}

fn matches_filter(status: &str, filter: &Option<StatusFilter>) -> bool {
    match filter {
        None => true,
        Some(StatusFilter::Include(statuses)) => {
            statuses.iter().any(|s| s == status)
        }
        Some(StatusFilter::Exclude(statuses)) => {
            !statuses.iter().any(|s| s == status)
        }
    }
}

// --- Commands ---

/// Check if a ticket or any of its descendants match the filter.
fn subtree_matches_filter(
    ticket: &Ticket, tickets: &[Ticket],
    filter: &Option<StatusFilter>,
) -> bool {
    if matches_filter(&ticket.status, filter) {
        return true;
    }
    // Check children recursively.
    tickets.iter()
        .filter(|t| t.parent == Some(ticket.number))
        .any(|t| subtree_matches_filter(t, tickets, filter))
}

fn cmd_list(
    tickets: &[Ticket], filter: &Option<StatusFilter>,
    digits: usize,
) {
    // Determine which tickets are visible: either they match the
    // filter directly, or they're ancestors of tickets that do
    // (shown as context to preserve tree structure).
    let visible: Vec<u32> = tickets.iter()
        .filter(|t| subtree_matches_filter(t, tickets, filter))
        .map(|t| t.number)
        .collect();

    let max_status_len = tickets.iter()
        .filter(|t| visible.contains(&t.number))
        .map(|t| t.status.len())
        .max()
        .unwrap_or(4);

    // Print tickets as a tree: top-level first, then recurse.
    fn print_tree(
        parent: Option<u32>, tickets: &[Ticket],
        visible: &[u32], filter: &Option<StatusFilter>,
        digits: usize, max_status_len: usize, depth: usize,
    ) {
        let indent = "  ".repeat(depth);
        for t in tickets {
            if t.parent != parent { continue; }
            if !visible.contains(&t.number) { continue; }

            let dimmed = !matches_filter(&t.status, filter);
            let marker = if dimmed { "  .." } else { "" };

            println!(
                "{indent}{:0>w$}  {:<sw$}  {}{marker}",
                t.number, t.status, t.title,
                w = digits, sw = max_status_len,
            );

            // Print children.
            print_tree(
                Some(t.number), tickets, visible, filter,
                digits, max_status_len, depth + 1,
            );
        }
    }

    print_tree(
        None, tickets, &visible, filter,
        digits, max_status_len, 0,
    );

    // Orphans: tickets whose parent doesn't exist. Show at top level.
    let has_parent_not_in_tickets: Vec<&Ticket> = tickets.iter()
        .filter(|t| {
            if let Some(p) = t.parent {
                !tickets.iter().any(|other| other.number == p)
            } else {
                false
            }
        })
        .filter(|t| visible.contains(&t.number))
        .collect();

    for t in has_parent_not_in_tickets {
        let dimmed = !matches_filter(&t.status, filter);
        let marker = if dimmed { "  .." } else { "" };
        println!(
            "{:0>w$}  {:<sw$}  {}{marker}",
            t.number, t.status, t.title,
            w = digits, sw = max_status_len,
        );
    }
}

fn cmd_show(ticket: &Ticket, tickets: &[Ticket], digits: usize) {
    let content = fs::read_to_string(&ticket.path)
        .expect("can't read ticket file");

    // Show parent chain if this is a sub-task.
    if let Some(p) = ticket.parent {
        let mut chain = Vec::new();
        let mut current = Some(p);
        while let Some(num) = current {
            if let Some(t) = find_ticket(tickets, num) {
                chain.push(format!("{:0>w$} {}", t.number, t.title, w = digits));
                current = t.parent;
            } else {
                chain.push(format!("{:0>w$} (not found)", num, w = digits));
                break;
            }
        }
        chain.reverse();
        for (i, entry) in chain.iter().enumerate() {
            let indent = "  ".repeat(i);
            println!("{indent}{entry}");
        }
        let indent = "  ".repeat(chain.len());
        println!(
            "{indent}{:0>w$}  [{}]  {}",
            ticket.number, ticket.status, ticket.title,
            w = digits,
        );
    } else {
        println!(
            "{:0>w$}  [{}]  {}",
            ticket.number, ticket.status, ticket.title,
            w = digits,
        );
    }
    println!();

    // Print the body (everything after the title).
    let mut past_fm = false;
    let mut past_title = false;
    let mut body_started = false;
    for line in content.lines() {
        if !past_fm {
            if line.trim() == "---" {
                past_fm = !past_fm;
            }
            // Skip first ---, wait for second ---
            continue;
        }
        if !past_title {
            if line.starts_with("# ") {
                past_title = true;
                continue;
            }
            // Skip blank lines between front matter and title.
            if line.trim().is_empty() { continue; }
        }
        if past_title {
            // Skip blank lines between title and body.
            if !body_started {
                if line.trim().is_empty() { continue; }
                body_started = true;
            }
            println!("{line}");
        }
    }

    // Show children.
    let children: Vec<&Ticket> = tickets.iter()
        .filter(|t| t.parent == Some(ticket.number))
        .collect();
    if !children.is_empty() {
        if body_started { println!(); }
        println!("Sub-tasks:");
        let max_status_len = children.iter()
            .map(|t| t.status.len())
            .max()
            .unwrap_or(4);
        for child in &children {
            println!(
                "  {:0>w$}  {:<sw$}  {}",
                child.number, child.status, child.title,
                w = digits, sw = max_status_len,
            );
        }
    }
}

fn cmd_new(
    dir: &Path, config: &Config, tickets: &[Ticket],
    title: Option<&str>, parent: Option<u32>,
) {
    if let Some(p) = parent {
        if find_ticket(tickets, p).is_none() {
            eprintln!(
                "error: parent ticket {:0>w$} not found", p,
                w = config.digits,
            );
            std::process::exit(1);
        }
    }

    if !dir.exists() {
        fs::create_dir_all(dir).expect("can't create task directory");
    }

    let num = next_number(tickets);
    let filename = format!(
        "{:0>width$}.md", num, width = config.digits
    );
    let path = dir.join(&filename);
    let title_text = title.unwrap_or("TODO");
    let date = today();

    let parent_line = match parent {
        Some(p) => format!("parent: {p}\n"),
        None => String::new(),
    };
    let content = format!(
        "---\nstatus: open\ncreated: {date}\n{parent_line}---\n\n\
         # {title_text}\n\n"
    );
    fs::write(&path, &content).expect("can't write ticket file");

    if std::io::stdin().is_terminal() {
        if let Ok(editor) = env::var("EDITOR") {
            if !editor.is_empty() {
                Command::new(&editor).arg(&path).status().ok();
                return;
            }
        }
    }
    println!("{filename}");
}

fn cmd_spawn(
    dir: &Path, config: &Config, tickets: &[Ticket],
    template: &Ticket,
) {
    let num = next_number(tickets);
    let filename = format!(
        "{:0>width$}.md", num, width = config.digits
    );
    let path = dir.join(&filename);
    let date = today();
    let title = format!("{} ({})", template.title, date);

    let content = format!(
        "---\nstatus: open\ncreated: {date}\nparent: {}\n---\n\n\
         # {title}\n\n",
        template.number,
    );
    fs::write(&path, &content).expect("can't write ticket file");
    println!("{filename}");
}

fn cmd_status(
    ticket: &Ticket, new_status: &str, config: &Config,
) {
    if !config.statuses.contains(&new_status.to_string()) {
        eprintln!("error: unknown status '{new_status}'");
        eprintln!(
            "valid statuses: {}", config.statuses.join(", ")
        );
        std::process::exit(1);
    }

    let content = fs::read_to_string(&ticket.path)
        .expect("can't read ticket file");
    let new_content = set_frontmatter_field(
        &content, "status", new_status,
    );
    fs::write(&ticket.path, new_content)
        .expect("can't write ticket file");

    let name = ticket.path.file_name().unwrap().to_string_lossy();
    println!("{name}: {} -> {new_status}", ticket.status);
}

fn cmd_log(dir: &Path, config: &Config, days: u32) {
    // Show uncommitted changes first.
    let diff_output = Command::new("git")
        .args(["status", "--porcelain", "--"])
        .arg(dir)
        .output();
    if let Ok(output) = diff_output {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut pending: Vec<String> = Vec::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() { continue; }
            let (code, path) = line.split_at(2);
            let path = path.trim();
            let Some(stem) = extract_ticket_stem(path, config)
            else {
                continue;
            };
            let action = match code.trim() {
                "A" | "??" => "new",
                "M" | "MM" => "modified",
                "D" => "deleted",
                _ => "changed",
            };
            pending.push(format!(
                "(pending) {stem}  {:<9}  {}",
                action,
                ticket_title_from_file(
                    &dir.join(format!("{stem}.md")),
                ),
            ));
        }
        if !pending.is_empty() {
            for line in &pending {
                println!("{line}");
            }
            println!();
        }
    }

    // Parse git log for committed changes.
    let since = format!("{} days ago", days);
    let log_output = Command::new("git")
        .args([
            "log", "--since", &since,
            "--name-status", "--pretty=format:%H %ai",
            "--",
        ])
        .arg(dir)
        .output()
        .expect("can't run git log");
    let text = String::from_utf8_lossy(&log_output.stdout);

    let mut entries: Vec<String> = Vec::new();
    let mut current_date = String::new();
    let mut current_hash = String::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }

        // Hash + date: "abc123 2026-03-20 16:50:13 +0100"
        if line.len() > 50 && line.as_bytes()[40] == b' ' {
            current_hash = line[..40].to_string();
            current_date = line[41..51].to_string();
            continue;
        }

        // File status line: "A\ttsk/006.md" or "M\ttsk/006.md"
        let Some((code, path)) = line.split_once('\t') else {
            continue;
        };
        let Some(stem) = extract_ticket_stem(path, config)
        else {
            continue;
        };

        match code {
            "A" => {
                entries.push(format!(
                    "{}  {}  {:<9}  {}",
                    current_date, stem, "created",
                    ticket_title_at_commit(dir, path, &current_hash),
                ));
            }
            "M" => {
                if let Some((from, to)) =
                    status_change_in_commit(path, &current_hash)
                {
                    entries.push(format!(
                        "{}  {}  {} -> {}",
                        current_date, stem, from, to,
                    ));
                } else {
                    entries.push(format!(
                        "{}  {}  {:<9}  {}",
                        current_date, stem, "updated",
                        ticket_title_from_file(
                            &dir.join(format!("{stem}.md")),
                        ),
                    ));
                }
            }
            _ => {}
        }
    }

    for entry in &entries {
        println!("{entry}");
    }

    if entries.is_empty() {
        println!("(no ticket changes in the last {days} days)");
    }
}

/// Extract ticket stem (e.g. "006") from a path like "tsk/006.md".
fn extract_ticket_stem(path: &str, config: &Config) -> Option<String> {
    let filename = path.rsplit('/').next()?;
    let stem = filename.strip_suffix(".md")?;
    if stem.len() != config.digits { return None; }
    stem.parse::<u32>().ok()?;
    Some(stem.to_string())
}

/// Get a ticket title by reading the file (for current state).
fn ticket_title_from_file(path: &Path) -> String {
    let content = fs::read_to_string(path).unwrap_or_default();
    let (_, title, _) = parse_ticket(&content);
    title
}

/// Get a ticket title from a specific commit.
fn ticket_title_at_commit(
    _dir: &Path, path: &str, rev: &str,
) -> String {
    let output = Command::new("git")
        .args(["show", &format!("{rev}:{path}")])
        .output();
    match output {
        Ok(out) => {
            let content = String::from_utf8_lossy(&out.stdout);
            let (_, title, _) = parse_ticket(&content);
            title
        }
        Err(_) => String::new(),
    }
}

/// Check if a specific commit changed the status field.
fn status_change_in_commit(
    path: &str, hash: &str,
) -> Option<(String, String)> {
    let output = Command::new("git")
        .args(["diff", &format!("{hash}^"), hash, "--", path])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);

    let mut old_status = None;
    let mut new_status = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("-status:") {
            old_status = Some(rest.trim().to_string());
        }
        if let Some(rest) = line.strip_prefix("+status:") {
            new_status = Some(rest.trim().to_string());
        }
    }
    match (old_status, new_status) {
        (Some(from), Some(to)) if from != to => Some((from, to)),
        _ => None,
    }
}

// --- Helpers ---

fn today() -> String {
    let output = Command::new("date")
        .arg("+%Y-%m-%d")
        .output()
        .expect("can't run `date`");
    String::from_utf8(output.stdout)
        .expect("invalid date output")
        .trim()
        .to_string()
}

fn require_ticket<'a>(
    tickets: &'a [Ticket], input: &str, digits: usize,
) -> &'a Ticket {
    let Some(num) = parse_ticket_number(input) else {
        eprintln!("error: '{input}' is not a valid ticket number");
        std::process::exit(1);
    };
    let Some(ticket) = find_ticket(tickets, num) else {
        eprintln!(
            "error: ticket {:0>w$} not found", num, w = digits,
        );
        std::process::exit(1);
    };
    ticket
}

/// Parse `tsk new` arguments: extract --parent N and the remaining
/// words as the title.
fn parse_new_args(args: &[String]) -> (Option<String>, Option<u32>) {
    let mut parent = None;
    let mut title_parts = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--parent" || args[i] == "-p" {
            i += 1;
            if i < args.len() {
                parent = args[i].parse::<u32>().ok();
                if parent.is_none() {
                    eprintln!(
                        "error: '{}' is not a valid ticket number",
                        args[i],
                    );
                    std::process::exit(1);
                }
            } else {
                eprintln!("error: --parent requires a ticket number");
                std::process::exit(1);
            }
        } else {
            title_parts.push(args[i].as_str());
        }
        i += 1;
    }
    let title = if title_parts.is_empty() {
        None
    } else {
        Some(title_parts.join(" "))
    };
    (title, parent)
}

fn cmd_init() {
    let cwd = env::current_dir().unwrap();

    // Check if project already exists.
    if cwd.join("tsk.yaml").exists() {
        eprintln!("tsk project already initialized (tsk.yaml exists)");
        return;
    }
    if cwd.join("tsk").is_dir() {
        eprintln!("tsk project already initialized (tsk/ exists)");
        return;
    }

    fs::create_dir_all(cwd.join("tsk"))
        .expect("can't create tsk/ directory");
    fs::write(cwd.join("tsk.yaml"), "")
        .expect("can't write tsk.yaml");
    println!("Initialized tsk project (tsk.yaml + tsk/)");
}

fn usage() {
    eprintln!("tsk — task management with markdown files\n");
    eprintln!("usage:");
    eprintln!("  tsk init                       Initialize a new project");
    eprintln!("  tsk new [--parent N] [title]   Create a new ticket");
    eprintln!("  tsk spawn <N>                  Create instance of a recurring ticket");
    eprintln!("  tsk list [status...]           List tickets (tree view)");
    eprintln!("  tsk list -<status>             Exclude a status");
    eprintln!("  tsk show <N>                   Show a ticket");
    eprintln!("  tsk status <N> <status>        Change ticket status");
    eprintln!("  tsk log [days]                 Recent activity (default: 7 days)");
}

// --- Main ---

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        return;
    }

    // Handle init before find_project() since there's no project yet.
    if args[1] == "init" {
        cmd_init();
        return;
    }

    let (root, config) = find_project();
    let dir = root.join(&config.dir);
    let tickets = scan_tickets(&dir, config.digits);

    match args[1].as_str() {
        "list" | "ls" => {
            let filter = parse_filters(&args[2..]);
            cmd_list(&tickets, &filter, config.digits);
        }
        "show" => {
            if args.len() < 3 {
                eprintln!("usage: tsk show <number>");
                std::process::exit(1);
            }
            let ticket = require_ticket(
                &tickets, &args[2], config.digits,
            );
            cmd_show(ticket, &tickets, config.digits);
        }
        "new" => {
            let (title, parent) = parse_new_args(&args[2..]);
            cmd_new(
                &dir, &config, &tickets, title.as_deref(), parent,
            );
        }
        "spawn" => {
            if args.len() < 3 {
                eprintln!("usage: tsk spawn <number>");
                std::process::exit(1);
            }
            let template = require_ticket(
                &tickets, &args[2], config.digits,
            );
            cmd_spawn(&dir, &config, &tickets, template);
        }
        "log" => {
            let days: u32 = args.get(2)
                .and_then(|s| s.parse().ok())
                .unwrap_or(7);
            cmd_log(&dir, &config, days);
        }
        "status" | "st" => {
            if args.len() < 4 {
                eprintln!("usage: tsk status <number> <status>");
                std::process::exit(1);
            }
            let ticket = require_ticket(
                &tickets, &args[2], config.digits,
            );
            cmd_status(ticket, &args[3], &config);
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            usage();
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Config parsing ---

    #[test]
    fn parse_config_defaults() {
        let config = parse_config("");
        assert_eq!(config.dir, "./tsk");
        assert_eq!(config.project, "project.md");
        assert_eq!(config.digits, 3);
        assert_eq!(
            config.statuses,
            vec!["open", "in-progress", "done"],
        );
    }

    #[test]
    fn parse_config_custom() {
        let config = parse_config(
            "dir: ./tasks\ndigits: 4\nstatuses:\n  - todo\n  - done\n"
        );
        assert_eq!(config.dir, "./tasks");
        assert_eq!(config.digits, 4);
        assert_eq!(config.statuses, vec!["todo", "done"]);
    }

    // --- Ticket parsing ---

    #[test]
    fn parse_ticket_full() {
        let (status, title, parent) = parse_ticket(
            "---\nstatus: in-progress\ncreated: 2026-03-20\n---\n\n# Do the thing\n\nBody.\n"
        );
        assert_eq!(status, "in-progress");
        assert_eq!(title, "Do the thing");
        assert_eq!(parent, None);
    }

    #[test]
    fn parse_ticket_no_status_defaults_to_open() {
        let (status, title, _) = parse_ticket(
            "---\ncreated: 2026-03-20\n---\n\n# Title\n"
        );
        assert_eq!(status, "open");
        assert_eq!(title, "Title");
    }

    #[test]
    fn parse_ticket_with_parent() {
        let (status, title, parent) = parse_ticket(
            "---\nstatus: open\nparent: 5\n---\n\n# Sub-task\n"
        );
        assert_eq!(status, "open");
        assert_eq!(title, "Sub-task");
        assert_eq!(parent, Some(5));
    }

    #[test]
    fn parse_ticket_with_custom_fields() {
        let (status, title, _) = parse_ticket(
            "---\nstatus: done\npriority: high\ntags: [a, b]\n---\n\n# Thing\n"
        );
        assert_eq!(status, "done");
        assert_eq!(title, "Thing");
    }

    // --- Front matter manipulation ---

    #[test]
    fn set_field_replaces_existing() {
        let input = "---\nstatus: open\ncreated: 2026-03-20\n---\n\n# Title\n";
        let result = set_frontmatter_field(input, "status", "done");
        assert_eq!(
            result,
            "---\nstatus: done\ncreated: 2026-03-20\n---\n\n# Title\n"
        );
    }

    #[test]
    fn set_field_inserts_when_missing() {
        let input = "---\ncreated: 2026-03-20\n---\n\n# Title\n";
        let result = set_frontmatter_field(input, "status", "open");
        assert_eq!(
            result,
            "---\ncreated: 2026-03-20\nstatus: open\n---\n\n# Title\n"
        );
    }

    #[test]
    fn set_field_preserves_custom_fields() {
        let input = "---\nstatus: open\npriority: high\ntags: [a]\n---\n\n# T\n";
        let result = set_frontmatter_field(input, "status", "done");
        assert_eq!(
            result,
            "---\nstatus: done\npriority: high\ntags: [a]\n---\n\n# T\n"
        );
    }

    // --- Ticket numbering ---

    #[test]
    fn next_number_empty() {
        assert_eq!(next_number(&[]), 1);
    }

    #[test]
    fn next_number_with_gap() {
        let tickets = vec![
            Ticket { number: 1, title: String::new(), status: String::new(), parent: None, path: PathBuf::new() },
            Ticket { number: 5, title: String::new(), status: String::new(), parent: None, path: PathBuf::new() },
        ];
        assert_eq!(next_number(&tickets), 6);
    }

    // --- Flexible ticket number input ---

    #[test]
    fn parse_ticket_number_bare() {
        assert_eq!(parse_ticket_number("3"), Some(3));
    }

    #[test]
    fn parse_ticket_number_padded() {
        assert_eq!(parse_ticket_number("003"), Some(3));
    }

    #[test]
    fn parse_ticket_number_invalid() {
        assert_eq!(parse_ticket_number("abc"), None);
    }

    // --- Status filters ---

    #[test]
    fn filter_include_single() {
        let f = parse_filters(&["open".into()]);
        assert!(matches_filter("open", &f));
        assert!(!matches_filter("done", &f));
    }

    #[test]
    fn filter_include_multiple() {
        let f = parse_filters(&["open".into(), "in-progress".into()]);
        assert!(matches_filter("open", &f));
        assert!(matches_filter("in-progress", &f));
        assert!(!matches_filter("done", &f));
    }

    #[test]
    fn filter_exclude() {
        let f = parse_filters(&["-done".into()]);
        assert!(matches_filter("open", &f));
        assert!(!matches_filter("done", &f));
    }

    #[test]
    fn filter_none() {
        let f = parse_filters(&[]);
        assert!(matches_filter("open", &f));
        assert!(matches_filter("done", &f));
    }

    // --- New args parsing ---

    #[test]
    fn parse_new_args_title_only() {
        let args: Vec<String> = vec!["Hello".into(), "world".into()];
        let (title, parent) = parse_new_args(&args);
        assert_eq!(title.as_deref(), Some("Hello world"));
        assert_eq!(parent, None);
    }

    #[test]
    fn parse_new_args_with_parent() {
        let args: Vec<String> = vec![
            "--parent".into(), "5".into(), "Sub-task".into(),
        ];
        let (title, parent) = parse_new_args(&args);
        assert_eq!(title.as_deref(), Some("Sub-task"));
        assert_eq!(parent, Some(5));
    }

    #[test]
    fn parse_new_args_parent_short_flag() {
        let args: Vec<String> = vec![
            "-p".into(), "3".into(), "Child".into(),
        ];
        let (title, parent) = parse_new_args(&args);
        assert_eq!(title.as_deref(), Some("Child"));
        assert_eq!(parent, Some(3));
    }

    #[test]
    fn parse_new_args_parent_at_end() {
        let args: Vec<String> = vec![
            "Title".into(), "--parent".into(), "7".into(),
        ];
        let (title, parent) = parse_new_args(&args);
        assert_eq!(title.as_deref(), Some("Title"));
        assert_eq!(parent, Some(7));
    }

    #[test]
    fn parse_new_args_empty() {
        let args: Vec<String> = vec![];
        let (title, parent) = parse_new_args(&args);
        assert_eq!(title, None);
        assert_eq!(parent, None);
    }

    // --- Tree structure ---

    fn make_ticket(
        number: u32, status: &str, title: &str, parent: Option<u32>,
    ) -> Ticket {
        Ticket {
            number, status: status.into(), title: title.into(),
            parent, path: PathBuf::new(),
        }
    }

    #[test]
    fn subtree_matches_filter_direct() {
        let tickets = vec![
            make_ticket(1, "open", "Parent", None),
            make_ticket(2, "done", "Child", Some(1)),
        ];
        let f = Some(StatusFilter::Include(vec!["open".into()]));
        assert!(subtree_matches_filter(&tickets[0], &tickets, &f));
        assert!(!subtree_matches_filter(&tickets[1], &tickets, &f));
    }

    #[test]
    fn subtree_matches_filter_via_child() {
        let tickets = vec![
            make_ticket(1, "done", "Parent", None),
            make_ticket(2, "open", "Child", Some(1)),
        ];
        let f = Some(StatusFilter::Include(vec!["open".into()]));
        // Parent is "done" but child is "open" — parent subtree matches.
        assert!(subtree_matches_filter(&tickets[0], &tickets, &f));
    }

    #[test]
    fn subtree_matches_filter_deep_nesting() {
        let tickets = vec![
            make_ticket(1, "done", "Grandparent", None),
            make_ticket(2, "done", "Parent", Some(1)),
            make_ticket(3, "open", "Child", Some(2)),
        ];
        let f = Some(StatusFilter::Include(vec!["open".into()]));
        assert!(subtree_matches_filter(&tickets[0], &tickets, &f));
        assert!(subtree_matches_filter(&tickets[1], &tickets, &f));
        assert!(subtree_matches_filter(&tickets[2], &tickets, &f));
    }
}
