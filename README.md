# idr-tools

Tooling to support Implementation Decision Records (IDRs), for humans and machines.

IDRs give you structured context that explains what you're building and why, without creating documentation debt.

```bash
idr new "Add OAuth support"
# Creates: idrs/202601051430-add-oauth-support.md
```

Opens a template like this:

```markdown
---
draft: true
---

# 2026-01-05: Add OAuth support

Owner: Your Name <you@example.com>

## Overview

### Problem Statement

<!-- What problem are we solving? -->

### Goals

<!-- What are we trying to achieve? -->

### Proposed Solution

<!-- High-level approach -->

## Detailed Design

<!-- Technical details, API design, etc. -->

## Working notes (ephemeral)

<!-- Questions, experiments, discoveries, and implementation details -->
```

Use this template as a starting point:

- ✏️ **Start with what you know**—edit directly or talk it through with an LLM
- 🔄 **Update through implementation and review**, keeping discoveries in Working notes
- 📝 **Finalize once review is mostly finished**, preserving reasoning and removing draft material
- ✅ **Commit it with your code**
- 🚀 **Move on** after it lands

IDRs are moments in time, a record of the thinking behind a solution to a problem and why it was chosen.
They are not intended to be permanent documentation of a system "as it is".

## Why IDRs?

This is an experiment to try and find a middle ground between proposals, documentation, and implementation.

[Traditional design docs](https://www.industrialempathy.com/posts/design-docs-at-google/) are useful for larger decisions that require a great deal of discussion and alternatives. [ADRs](https://adr.github.io/) are great for documenting decisions but lack detail.
IDRs bridge the gap: detailed enough to guide implementation, ephemeral enough to not become stale documentation.

In some ways an IDR is analogous to a very large commit message. However, it is generally easier to scan,
parse, and search as a permanent historical record.

## Usage

Create a new IDR:

```bash
idr new "Your Feature Title"
```

This will create a new markdown file in the `idrs/` directory with:

- A UTC timestamp prefix (e.g., `202512301600`)
- A slugified version of your title
- Pre-filled template with your git author name

Example:

```bash
idr new "Add last modified metadata"
# Creates: idrs/202512301600-add-last-modified-metadata.md
```

The IDR template includes sections for:

- Problem Statement
- Context
- Goals and Non-Goals
- Proposed Solution
- Detailed Design
- Working notes (ephemeral)

The comments are useful reminders for both humans and LLMs, but you can opt to not include them by passing `--no-comments` or setting the environment variable `IDR_NO_COMMENTS=1`.

### Drafts and finalization

New IDRs start with `draft: true`, including when comments are omitted. Rough
notes, questions, and incomplete sections are welcome. Approval of the proposed
solution lets implementation begin; the IDR stays a draft through implementation
and review. Only the overview is required; other sections are optional.

Once human or LLM review has at least mostly finished, finalize through deliberate
edits, either yourself or with an LLM's help:

1. Preserve discoveries that explain the chosen approach in the relevant durable
   sections, including consequential changes in understanding.
2. Remove Working notes, guidance comments, and empty optional sections.
3. Remove the `draft` field, rather than setting it to `false`. Keep unrelated
   frontmatter; remove the block if it becomes empty.

### Check finalization

```bash
idr lint idrs/202601051430-add-oauth-support.md
# Multiple explicit paths are also supported:
idr lint idrs/first.md idrs/second.md
```

Lint checks only the named files, without modifying them or scanning the repository.
Findings include a path, one-based line number, stable code, and message:

```text
idrs/example.md:2: IDR001 Draft metadata remains; remove the `draft` field when finalizing
```

| Code | Meaning |
| --- | --- |
| `IDR001` | A top-level `draft` field remains in YAML frontmatter, regardless of its value |
| `IDR002` | A Markdown/HTML comment remains |
| `IDR003` | A Working notes heading remains, with or without `(ephemeral)` |

Code examples in fenced blocks, indented blocks, and inline code do not trigger
findings. Working notes headings are matched without regard to case or heading
level. Lint exits with `0` when all files pass, `1` for findings, and `2` for usage,
read, or frontmatter parsing errors. It continues checking other named files if
one cannot be read or parsed.

Passing lint checks for leftover draft material; it does not establish that the
IDR communicates intent or that review is complete.

This repository's PR check, **Lint added IDRs**, runs lint on Markdown files
newly added under `idrs/` by the PR. Existing records are not required to pass.
Draft IDRs will fail the check until finalized after review.

## Agent skill

This repository includes an optional `idr` skill for LLM collaboration. It invites
direct editing and also supports developing an IDR through conversation. You can
move between editing, discussion, and critique as needed. The human owns the
design; the LLM can scaffold, critique, implement from an approved IDR, and keep
Working notes. Verification checks whether the implementation serves the stated
intent as well as whether the diff and IDR agree. Finalization is a separate step
after review is mostly finished, and can happen in a later session.

To install it locally, symlink `skills/idr` into your agent's skills directory:

```bash
# Codex
ln -s /path/to/idr-tools/skills/idr ~/.agents/skills/idr

# Claude Code
ln -s /path/to/idr-tools/skills/idr ~/.claude/skills/idr
```

## Examples

This feature was itself implemented using an IDR—see [idrs/202601031709-init.md](idrs/202601031709-init.md) for more about the IDR format and philosophy.
