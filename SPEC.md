# tsk format specification

Version: 0.1 (draft)

## Overview

tsk is a file format for managing tasks as markdown files in a directory,
typically inside a git repository. This document defines the format
precisely enough that any tool can implement it.

## Definitions

- **tsk project**: a directory containing task files and optionally a
  project description file.
- **ticket**: a markdown file with YAML front matter representing a task.
- **comment**: a markdown file with YAML front matter associated with
  a ticket.

## Project discovery

A tsk project is identified by the presence of `tsk.yaml` in an ancestor
directory. If no `tsk.yaml` is found, a tool MAY fall back to looking for
a `./tsk/` directory.

## Configuration: `tsk.yaml`

Located at the root of the repository (or wherever the project is anchored).
All fields are optional. Defaults are shown below:

```yaml
dir: ./tsk          # path to the task directory, relative to tsk.yaml
project: project.md # project description file, relative to dir
digits: 3           # zero-padded width for ticket numbers
statuses:           # allowed status values
  - open
  - in-progress
  - done
```

An empty `tsk.yaml` is valid — all defaults apply.

## Task directory

The task directory (default: `./tsk/`) contains:
- Ticket files (`NNN.md`)
- Comment directories (`NNN/`)
- A project description file (default: `project.md`)

No other structure is imposed. Tools SHOULD ignore files that don't match
these patterns.

## Tickets

### Filename

Tickets are named `NNN.md` where `NNN` is a zero-padded decimal number.
The number of digits is configured by `digits` in `tsk.yaml` (default: 3).

Examples: `001.md`, `042.md`, `999.md`.

### Ticket numbering

The next ticket number is determined by scanning existing ticket files and
incrementing the highest number found. If no tickets exist, the first
ticket is `001.md` (or the equivalent for the configured digit width).

### Structure

A ticket file consists of:

1. **YAML front matter** — delimited by `---` lines.
2. **Title** — the first `# ` (H1) heading in the body.
3. **Body** — free-form markdown after the title.

### Front matter

#### Core fields

| Field     | Type   | Required | Default  | Description                    |
|-----------|--------|----------|----------|--------------------------------|
| `status`  | string | no       | `open`   | One of the configured statuses |
| `created` | date   | no       | —        | Creation date (YYYY-MM-DD)     |

#### Custom fields

Any additional YAML fields are allowed. Tools MUST preserve fields they
don't understand when reading, modifying, and writing ticket files.

There is no distinction between "standard optional" and "custom" fields
at the format level. A tool may choose to understand `tags`, `assignee`,
`priority`, `parent`, or any other field — but the format does not require
it.

### Example

```markdown
---
status: open
created: 2026-03-16
---

# Implement the thing

Description, acceptance criteria, whatever you need.
```

## Comments

### Directory

Comments for ticket `NNN` live in a directory named `NNN/` (same directory
as the ticket file, without the `.md` extension).

### Filename

Comments are named `MMM.md` where `MMM` follows the same zero-padded
convention as tickets.

### Structure

Same as tickets: YAML front matter, H1 title (optional for comments),
and a markdown body.

### Front matter

| Field     | Type   | Required | Default | Description            |
|-----------|--------|----------|---------|------------------------|
| `created` | date   | no       | —       | Creation date          |
| `author`  | string | no       | —       | Who wrote the comment  |

Custom fields are allowed, same rules as tickets.

### Example

```markdown
---
created: 2026-03-16
author: vasily
---

# Reconsidered the approach

Actually, let's do it the other way.
```

## Project description

The project description file (default: `project.md`) is a markdown file
with optional YAML front matter. It describes the project that the tickets
belong to. Its format is not strictly defined — it's free-form markdown.

## Tool requirements

Tools that implement the tsk format MUST:
- Preserve unknown front matter fields when modifying files.
- Respect the `tsk.yaml` configuration when present.
- Use the configured digit width for new ticket/comment numbers.

Tools that implement the tsk format SHOULD:
- Default to `open` when `status` is omitted.
- Ignore files in the task directory that don't match ticket/comment
  naming patterns.
- Treat the first `# ` heading as the ticket title.
