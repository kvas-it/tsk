# tsk

A task management system made of markdown files in a git repository.

## The idea

Software interaction exists on a spectrum: UI → API → filesystem → stream
of bytes. Each level is simpler and easier to implement, but somewhat more
limited in expressiveness. For task management, the filesystem level hits a
sweet spot — it's simple enough that any tool (including AIs) can interact
with it natively, yet expressive enough to capture real project structure.

tsk is built on this insight:
- **Tasks are markdown files.** Human-readable, AI-readable, diffable.
- **Structure is directories.** Group, nest, and organize however you want.
- **History is git.** Every change is tracked, branchable, mergeable.

No server, no database, no API to learn. Just files.

## Design principles

- **AI-friendly first.** The filesystem is the interface. Any tool that can
  read and write files can be a full participant.
- **Human-friendly always.** Markdown is for humans too. You can manage your
  tasks with nothing but a text editor and `ls`.
- **Convention over structure.** Light conventions, not rigid schemas. The
  system should be useful before you've read any documentation.
- **Git-native.** History, collaboration, and branching come free.

## What a task looks like

```markdown
---
status: open
created: 2026-03-16
priority: high
---

# Make the thing work

Description goes here. Free-form markdown.
```

Metadata lives in YAML front matter. The title is the first `# heading`.
The body is whatever you need it to be.

The core fields are `status` and `created`. Beyond that — add whatever
you want. Need `priority`? Just add it. Need `sprint`, `estimate`,
`blocked-by`? Type it in. No admin panel, no custom field configuration,
no screen schemes. Your tools will see it because it's just YAML.

## Getting started

### Just files

Create a `tsk.yaml` in your repo root (it can be empty) and a `tsk/`
directory. Start adding `001.md`, `002.md`, etc. That's it.

### With the CLI

```
cargo install --git https://github.com/kvas-it/tsk
```

```
tsk new "Fix the login bug"    # creates the next ticket, opens $EDITOR
tsk list                       # all tickets
tsk list open                  # just open ones
tsk status 42 done             # mark ticket 42 as done
```

### With Claude Code

tsk ships with a Claude Code skill. Clone the repo (or copy
`.claude/skills/tsk/` into your project) and Claude can create, list,
and modify tickets directly.

## Project structure

```
your-repo/
├── tsk.yaml          # config (optional, presence signals "this is a tsk project")
└── tsk/
    ├── project.md    # project description
    ├── 001.md        # tickets
    ├── 002.md
    ├── 002/
    │   └── 001.md    # comments on ticket 002
    └── ...
```

## Format specification

See [SPEC.md](SPEC.md) for the full format definition. tsk is a file
format — any tool that reads and writes these files correctly is a valid
implementation.
