---
name: idr
description: Use when the user explicitly asks to create, review, critique, implement from, or update an Implementation Decision Record
---

# IDR Workflow

## Core Principle

The human owns the design. The agent helps create the scaffold, critiques the IDR, implements from the approved IDR, verifies the work, and records what actually happened.

## Stage 1: Create Scaffold

When the user asks to start an IDR:

1. Resolve the command using the command resolution rules below.
2. Run the chosen command with a short title:
   ```bash
   <idr command> new "<title>"
   ```
3. Report the created path.
4. Stop and wait for the user to fill in the main sections.

The user normally owns:

- Problem Statement
- Context
- Goals
- Non-Goals
- Proposed Solution
- Alternatives considered

## Stage 2: Develop And Critique IDR

When the user asks for help developing or reviewing an IDR:

1. Read the full IDR.
2. Develop an internal critique before drafting new IDR text.
3. If the user requested a specific critique mode, use it.
4. If the user did not request a specific critique mode, perform a concise general review and ask whether they want to go deeper on a particular dimension.
5. Align with the user on the problem statement, context, goals, non-goals, and proposed solution before helping fill in other sections.
6. After alignment, help fill in requested sections such as Other reading or Implementation hints when asked.
7. Do not rewrite human-owned sections unless asked.

Useful critique modes include:

- What is missing?
- Are the goals clearly stated?
- Are the non-goals useful?
- Does the proposed solution follow from the problem?
- What implementation risks or edge cases are not addressed?
- What tests does this imply?
- Is the scope too broad?
- What would make implementation ambiguous?

## Stage 3: Implement From Approved IDR

When the user asks to implement from an approved IDR:

1. Re-read the IDR.
2. Treat it as the source of truth.
3. Flag contradictions or unresolved ambiguity before coding.
4. If implementation reveals that the approved IDR is wrong, stale, or incomplete, stop and report the mismatch rather than silently implementing a different design.
5. Implement within the stated scope.
6. Add or update tests.
7. Run relevant verification.
8. Write or update docs when the change affects documented behavior.

## Stage 4: Update And Verify IDR

Before declaring the task done:

1. Compare the final diff to the IDR.
2. Update `## Implementation (ephemeral)` with factual implementation notes.
3. Flag stale human-authored design sections instead of silently rewriting them.
4. Summarize what changed and what was verified.

The `## Implementation (ephemeral)` section is the main section the agent should expect to update after doing the work. Humans may write implementation hints there, but in the agent-assisted workflow this section is primarily where the agent records what actually happened.

## Command Resolution

Do not assume `idr` is always on `PATH`. Prefer the least invasive working command:

1. `idr new "<title>"` if `idr --help` works.
2. `uv run idr new "<title>"` if the project already uses `uv` and `uv run idr --help` works.
3. `uvx --from idr-tools idr new "<title>"` if no project-local command exists and the user accepts a non-project-local tool invocation.

If none works, ask the user how to invoke `idr`. Do not install dependencies globally or add `idr-tools` as a project dependency without explicit approval.

## Context Handoff

After IDR approval, ask whether the user wants to continue in the same session or start a new agent session manually.

If the user wants a new session, first capture important implementation-relevant research in `## Implementation (ephemeral)`.

If the user continues in the same session, proceed from the approved IDR.
