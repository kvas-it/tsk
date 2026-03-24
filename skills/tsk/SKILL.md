---
name: tsk
description: Manage tsk tickets — create, update, list, and comment on markdown task files. Use when the user asks to create tickets, change ticket status, list tasks, or work with the tsk task management system.
argument-hint: "[new|list|status|show] [args]"
---

# tsk — markdown task management

You manage tasks stored as markdown files with YAML front matter.
Read `tsk.yaml` (if present) to find the task directory and configuration.
Defaults: dir `./tsk/`, digits `3`, statuses `open/in-progress/done`.

## Quick reference

**Create a ticket:** find the highest NNN.md, increment, write the file:

```markdown
---
status: open
created: YYYY-MM-DD
---

# Ticket title

Description.
```

**List tickets:** scan NNN.md files, extract status and title (first `# `).

**Change status:** edit the `status:` line in front matter. Preserve all
other fields exactly as they are.

**Add a comment:** create `NNN/MMM.md` (e.g. `003/001.md`) with front
matter including `created` and `author`.

**Sub-tasks:** create a separate ticket with `parent: NNN` in front matter.

## Rules

- Title is the first `# H1` in the body, not in front matter.
- Custom front matter fields are allowed. ALWAYS preserve fields you
  don't recognize when modifying a file.
- Ticket numbers are zero-padded to the configured digit width (default 3).
- Next ticket number = highest existing number + 1.
- Default status is `open` when omitted.

## CLI

If the `tsk` binary is available, prefer it over direct file manipulation.
Run `tsk` with no arguments to see available commands.

Fall back to reading/writing files directly if the CLI is not available.
