# tsk

A task management system made of markdown files in a git repository.
We dogfood tsk to track its own development in `./tsk/`.

## File format

tsk is a file format specification. Tools are implementations — any program
that reads/writes these files correctly is a valid tsk tool.

### Tickets

Files named `NNN.md` in the task directory (default: `./tsk/`).

- **Front matter** (YAML): metadata only — `status`, `created`. Optional:
  `assignee`, `priority`, `tags`, `parent`.
- **Title**: first `# H1` in the body.
- **Body**: free-form markdown after the title.

Example:

```markdown
---
status: open
created: 2026-03-16
---

# Implement the thing

Description, acceptance criteria, notes — whatever makes sense.
```

### Comments

Files named `NNN/MMM.md` — a directory per ticket, one file per comment.
Comments also have front matter (`author`, `created` at minimum).

### Sub-tasks

Separate tickets that reference a parent via `parent: NNN` in front matter.
Everything stays flat in the same directory.

### Project description

`project.md` in the task directory describes the project.

### Configuration

`tsk.yaml` at repo root. Its presence signals "this repo uses tsk."
All fields are optional — conventions provide defaults:

```yaml
dir: ./tsk          # task directory (default: ./tsk)
project: project.md # project file (default: project.md)
digits: 3           # ticket number width (default: 3)
statuses:           # allowed statuses (defaults below)
  - open
  - in-progress
  - done
```

### Next ticket number

Determined by scanning existing files. No counter to maintain.

## Viewing tickets

We use [rendar](https://github.com/kvas-it/rendar) to view tickets —
it works with directories of markdown files with YAML front matter.

## Development

- Tooling language: Rust (when we need tooling — start with just files)
- The format spec is the primary artifact; tools implement it
- When something feels like friction, that's when we build a tool for it
