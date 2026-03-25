# Changelog

## 2026-03-25 — `tsk init` command

- Added `tsk init` — creates `tsk.yaml` and `tsk/` in the current directory.
  Safe to run twice (detects existing project).
- Updated skill to use `tsk init` instead of manual `mkdir`.
- Ticket 017

## 2026-03-25 — Integration tests & CI

- Added 19 integration tests (`tests/cli.rs`) — run the binary against
  temp projects and assert on output. Covers: new, list (flat, tree,
  filters, orphans, ancestor markers), show (basic, children, parent
  chain), status changes, error cases.
- Added GitHub Actions CI (`.github/workflows/ci.yml`) — runs
  `cargo test` on push to main and on PRs.
- 45 tests total (26 unit + 19 integration)
- Tickets 015, 016

## 2026-03-25 — Task nesting

- Added `parent` as a documented optional field in SPEC.md
- `tsk new --parent N` (or `-p N`) creates sub-tasks
- `tsk list` shows tree view — children indented under parents
- Filtered parents shown with `..` marker to preserve tree context
- `tsk show` displays parent chain for sub-tasks, lists children
- Orphan tickets (parent doesn't exist) shown at top level
- 26 tests (up from 18)
- Ticket 014

## 2026-03-20 — Show formatting fix

- Fixed `tsk show` printing two blank lines between header and body (now one)
- Closed ticket 013

## 2026-03-20 — Documentation

- Rewrote README — shorter, scannable, optimized for quick understanding
- Added contribution workflow to CLAUDE.md: tickets first, track progress,
  definition of done
- Closed ticket 011

## 2026-03-20 — Activity log

- Added `tsk log [days]` — shows recent ticket activity from git history
- Detects ticket creation, status changes, and updates
- Shows uncommitted changes at the top
- Defaults to last 7 days
- Closed ticket 010

## 2026-03-20 — CLI improvements

- Fixed digit width bug in `tsk list` (was hardcoded to 3)
- Added `tsk show <N>` — formatted ticket display
- Added negative/multi-status filters: `tsk list -done`,
  `tsk list open in-progress`
- Flexible ticket number input: `tsk show 3` and `tsk show 003` both work
- 17 tests (up from 10)
- Closed tickets 006–009

## 2026-03-20 — MVP ready

- Created Claude Code skill (`.claude/skills/tsk/SKILL.md`) — Claude
  can create, list, and modify tickets. Uses CLI when available, falls
  back to direct file operations
- Finalized front matter schema decisions: only `status` and `created`
  are core, everything else is custom
- Added getting started, CLI usage, and project structure to README
- Closed tickets 003 and 005
- All initial tickets (001–005) are done

## 2026-03-20 — Rust CLI v0.1

- Built the `tsk` CLI — zero dependencies, ~300 lines of Rust
- Commands: `tsk new`, `tsk list`, `tsk status`
- Hand-rolled YAML parsing for tsk.yaml and front matter
- Front matter manipulation preserves unknown fields
- 10 unit tests covering core logic
- Closed ticket 004

## 2026-03-20 — Tooling decisions

- Decided on two complementary tools: Rust CLI for humans, Claude Code
  skill for AI agents — both operating on the same files
- Rust CLI commands: `tsk new`, `tsk list`, `tsk status`
- Closed ticket 002
- Opened tickets:
  - 004: Build the tsk Rust CLI
  - 005: Create Claude Code skill for tsk

## 2026-03-16 — Format spec (v0.1 draft)

- Wrote `SPEC.md` — standalone format specification covering tickets,
  comments, front matter, tsk.yaml, and tool requirements
- Key decisions codified: `status` defaults to `open`, custom fields
  are always allowed, tools must preserve unknown fields
- Added task example and custom fields pitch to README
- Closed ticket 001

## 2026-03-16 — Project bootstrap

- Created the repository
- Defined the tsk file format: tickets as `NNN.md` with YAML front matter,
  title in first H1, comments as `NNN/MMM.md`, config in `tsk.yaml`
- Set up dogfooding: tsk tracks its own development in `./tsk/`
- Created `CLAUDE.md`, `tsk.yaml`, `tsk/project.md`
- Opened initial tickets:
  - 001: Write the format specification
  - 002: Figure out the tooling story
  - 003: Define front matter schema
