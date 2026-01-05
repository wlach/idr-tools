# idr-tools

Tooling to support Implementation Decision Records (IDRs), for humans and machines.

IDRs give you structured context that explains what you're building and why, without creating documentation debt.

```bash
idr new "Add OAuth support"
# Creates: idrs/202601051430-add-oauth-support.md
```

Opens a template like this:

```markdown
# 2026-01-05: Add OAuth support

Owner: Your Name <you@example.com>

## Overview

### Problem Statement

<!-- What problem are we solving? -->

### Goals

<!-- What are we trying to achieve? -->

### Proposed Solution

<!-- High-level approach -->

## Detailed Design (as needed)

<!-- Technical details, API design, etc. -->

## Implementation (ephemeral)

<!-- What actually happened during implementation -->
```

Use this template as a starting point:

- ✏️ **Fill it in** (get an LLM to help if you like)
- 🔄 **Update as you work** (especially the implementation section)
- ✅ **Commit it with your code**
- 🚀 **Move on** after it lands

IDRs are moments in time, a record of what was done. They are not intended to be permanent documentation of the system "as it is".

## Why IDRs?

This is an experiment to try and find a middle ground between proposals, documentation, and implementation.

[Traditional design docs](https://www.industrialempathy.com/posts/design-docs-at-google/) are useful for larger decisions that require a great deal of discussion and alternatives. [ADRs](https://adr.github.io/) are great for documenting decisions but usually lack implementation detail. IDRs bridge the gap: detailed enough to guide implementation, ephemeral enough to not become stale documentation.

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
- Implementation notes (ephemeral)

The comments are useful reminders for both humans and LLMs, but you can opt to not include them by passing `--no-comments` or setting the environment variable `IDR_NO_COMMENTS=1`.

## Examples

This feature was itself implemented using an IDR—see [idrs/202601031709-init.md](idrs/202601031709-init.md) for more about the IDR format and philosophy.
