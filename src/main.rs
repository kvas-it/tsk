use std::{env, fs, path::{Path, PathBuf}, process::Command};

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
        let (status, title) = parse_ticket(&content);

        tickets.push(Ticket {
            number: num,
            title,
            status,
            path: entry.path(),
        });
    }
    tickets.sort_by_key(|t| t.number);
    tickets
}

/// Extract status from front matter and title from first H1.
fn parse_ticket(content: &str) -> (String, String) {
    let mut status = "open".to_string();
    let mut title = String::new();
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
    (status, title)
}

fn next_number(tickets: &[Ticket]) -> u32 {
    tickets.iter().map(|t| t.number).max().unwrap_or(0) + 1
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

// --- Commands ---

fn cmd_list(tickets: &[Ticket], filter: Option<&str>) {
    let max_status_len = tickets.iter()
        .map(|t| t.status.len())
        .max()
        .unwrap_or(4);

    for t in tickets {
        if let Some(s) = filter {
            if t.status != s { continue; }
        }
        println!(
            "{:0>3}  {:<width$}  {}",
            t.number, t.status, t.title,
            width = max_status_len,
        );
    }
}

fn cmd_new(
    dir: &Path, config: &Config, tickets: &[Ticket],
    title: Option<&str>,
) {
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

    let content = format!(
        "---\nstatus: open\ncreated: {date}\n---\n\n# {title_text}\n\n"
    );
    fs::write(&path, &content).expect("can't write ticket file");

    match env::var("EDITOR") {
        Ok(editor) if !editor.is_empty() => {
            Command::new(&editor).arg(&path).status().ok();
        }
        _ => println!("{filename}"),
    }
}

fn cmd_status(
    tickets: &[Ticket], num: u32, new_status: &str,
    config: &Config,
) {
    let Some(ticket) = tickets.iter().find(|t| t.number == num) else {
        eprintln!(
            "error: ticket {:0>width$} not found",
            num, width = config.digits,
        );
        std::process::exit(1);
    };

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

fn usage() {
    eprintln!("tsk — task management with markdown files\n");
    eprintln!("usage:");
    eprintln!("  tsk new [title]           Create a new ticket");
    eprintln!("  tsk list [status]         List tickets");
    eprintln!("  tsk status <N> <status>   Change ticket status");
}

// --- Main ---

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        return;
    }

    let (root, config) = find_project();
    let dir = root.join(&config.dir);
    let tickets = scan_tickets(&dir, config.digits);

    match args[1].as_str() {
        "list" | "ls" => {
            let filter = args.get(2).map(|s| s.as_str());
            cmd_list(&tickets, filter);
        }
        "new" => {
            let title = if args.len() > 2 {
                Some(args[2..].join(" "))
            } else {
                None
            };
            cmd_new(&dir, &config, &tickets, title.as_deref());
        }
        "status" | "st" => {
            if args.len() < 4 {
                eprintln!("usage: tsk status <number> <status>");
                std::process::exit(1);
            }
            let num: u32 = args[2].parse().unwrap_or_else(|_| {
                eprintln!(
                    "error: '{}' is not a valid ticket number",
                    args[2],
                );
                std::process::exit(1);
            });
            cmd_status(&tickets, num, &args[3], &config);
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
        let (status, title) = parse_ticket(
            "---\nstatus: in-progress\ncreated: 2026-03-20\n---\n\n# Do the thing\n\nBody.\n"
        );
        assert_eq!(status, "in-progress");
        assert_eq!(title, "Do the thing");
    }

    #[test]
    fn parse_ticket_no_status_defaults_to_open() {
        let (status, title) = parse_ticket(
            "---\ncreated: 2026-03-20\n---\n\n# Title\n"
        );
        assert_eq!(status, "open");
        assert_eq!(title, "Title");
    }

    #[test]
    fn parse_ticket_with_custom_fields() {
        let (status, title) = parse_ticket(
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
            Ticket { number: 1, title: String::new(), status: String::new(), path: PathBuf::new() },
            Ticket { number: 5, title: String::new(), status: String::new(), path: PathBuf::new() },
        ];
        assert_eq!(next_number(&tickets), 6);
    }
}
