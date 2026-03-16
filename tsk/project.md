---
created: 2026-03-16
---

# tsk

A task management system made of markdown files in a git repository.

## Vision

tsk is a file format, not a product. The spec defines how task files are
organized; any tool that reads/writes these files correctly is a valid
implementation. The filesystem is the API.

## Current phase

Bootstrapping. We're defining the format by using it — dogfooding tsk to
track its own development. No tooling yet; we'll build tools when we feel
friction.

## Key decisions

- Tickets are `NNN.md` with YAML front matter and H1 title
- Comments are `NNN/MMM.md`
- Sub-tasks are separate tickets with `parent: NNN`
- Config in `tsk.yaml` at repo root
- Default statuses: open, in-progress, done
- Title lives in the body (first `# H1`), not front matter
- Next ticket number: scan the directory
- Tooling in Rust, when needed
- rendar for viewing
