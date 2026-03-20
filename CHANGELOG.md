# Changelog

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
