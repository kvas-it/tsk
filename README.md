# tsk

Task management with markdown files in a git repository.

Tasks are files. Structure is directories. History is git.
No server, no database, no API. Just files.

## What a task looks like

```markdown
---
status: open
created: 2026-03-16
---

# Fix the login bug

The login form accepts empty passwords. Add validation.
```

YAML front matter for metadata, `# title` for the title, markdown body
for everything else. Add any fields you want — `priority`, `tags`,
`sprint`, whatever. It's just YAML.

## Getting started

Add a `tsk/` directory to your repo. Create `001.md`. Done.

Optionally add `tsk.yaml` at the repo root for configuration:

```yaml
dir: ./tsk          # where tickets live (default: ./tsk)
digits: 3           # ticket number width (default: 3)
statuses:           # allowed statuses (defaults below)
  - open
  - in-progress
  - done
```

### CLI

```
cargo install --git https://github.com/kvas-it/tsk
```

```
tsk new "Fix the login bug"    # create ticket, open $EDITOR
tsk list                       # all tickets
tsk list -done                 # everything except done
tsk show 42                    # show a ticket
tsk status 42 done             # change status
tsk log                        # recent activity
```

### Claude Code

tsk ships with a [Claude Code skill](skills/tsk/SKILL.md). Copy the
`skills/` directory into your project and Claude can manage tickets
directly — or just describe the format in your CLAUDE.md.

## Project structure

```
your-repo/
├── tsk.yaml              # config (optional)
└── tsk/
    ├── project.md         # project description
    ├── 001.md             # tickets
    ├── 002.md
    ├── 002/
    │   └── 001.md         # comments on ticket 002
    └── ...
```

Comments live in `NNN/MMM.md`. Sub-tasks are separate tickets with
`parent: NNN` in front matter.

## Format spec

tsk is a file format. Any tool that reads/writes these files correctly
is a valid implementation. See [SPEC.md](SPEC.md) for the full
definition.
