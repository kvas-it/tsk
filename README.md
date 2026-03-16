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
