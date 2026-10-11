---
name: idr
description: Use when the user explicitly asks to create, review, critique, implement from, or update an Implementation Decision Record
---

# IDR Workflow

## Core Principle

The human owns the design. The LLM helps create the scaffold, critiques the IDR, implements from the approved IDR, verifies the work, and records what actually happened.

An IDR remains a draft through implementation and review. Questions, tentative explanations, and incomplete sections are welcome while understanding develops. Approval of the proposed solution allows implementation to begin; it does not finalize the record.

## Stage 1: Create Scaffold

When the user asks to start an IDR:

1. Resolve the command using the command resolution rules below.
2. Run the chosen command with a short title:
   ```bash
   <idr command> new "<title>"
   ```
3. Report the created path.
4. Invite the user to edit the design sections directly, starting with whatever they know; rough notes and incomplete thoughts are welcome. Offer to talk it through alongside that invitation, rather than requiring them to choose a mode. Follow any preference they have already expressed.
5. If they prefer an interactive session, help develop the sections below through dialogue. Start with what is known; do not force premature completeness.

Users can move freely between editing, discussion, and critique throughout the work.

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
6. After alignment, help fill in requested sections such as Other reading or implementation hints in Working notes when asked.
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
3. Flag contradictions or ambiguity that prevents implementation within the agreed intent. Incompleteness alone need not stop implementation.
4. If implementation reveals that the approved IDR is wrong or stale, stop and report the mismatch rather than silently implementing a different design.
5. Implement within the stated scope.
6. Add or update tests.
7. Run relevant verification.
8. Write or update docs when the change affects documented behavior.

Use `## Working notes (ephemeral)` throughout implementation and review to capture questions, experiments, discoveries, and implementation results. Humans may also write implementation hints there. Keep these notes available while review can still reshape the work and for handoff between sessions.

## Stage 4: Update And Verify IDR

Before declaring the task done:

1. Check whether the implementation serves the stated intent, as well as whether the diff and IDR agree.
2. A mismatch may require changing the implementation or revisiting the proposed solution. Identify changes to the solution; do not simply rewrite the IDR to justify whatever was built. Do not change the problem statement, goals, or non-goals without human consent.
3. Propose updates informed by Working notes and review feedback. Preserve discoveries that explain why the chosen approach prevailed, including consequential changes in understanding.

Repeat verification and revision as human feedback changes the solution.

## Stage 5: Finalization

Finalize when the work is ready to merge, meaning human or LLM review has at least mostly finished. Completing implementation alone does not trigger finalization; this stage may happen in a later session.

Finalize through manual edits:

1. Move discoveries needed to explain the outcome or changed decisions from Working notes into the relevant durable sections. Retain the reasoning without recounting every experiment.
2. Remove Working notes, guidance comments, and empty optional sections. The overview should explain the real-world need and approach taken; other sections are optional.
3. Remove the `draft` field, rather than setting it to `false`. Preserve unrelated frontmatter; remove the block if it becomes empty.

When the CLI supports `lint`, use `<idr command> lint <file>...` on the relevant IDRs to check for leftover draft material. Passing lint does not establish that the record communicates intent or that review is complete.

## Command Resolution

Do not assume `idr` is always on `PATH`. Prefer the least invasive working command:

1. `idr new "<title>"` if `idr --help` works.
2. `uv run idr new "<title>"` if the project already uses `uv` and `uv run idr --help` works.
3. `uvx --from idr-tools idr new "<title>"` if no project-local command exists and the user accepts a non-project-local tool invocation.

If none works, ask the user how to invoke `idr`. Do not install dependencies globally or add `idr-tools` as a project dependency without explicit approval.

## Context Handoff

After IDR approval, ask whether the user wants to continue in the same session or start a new LLM session manually, unless they have already specified how to proceed.

If the user wants a new session, first capture important implementation-relevant research in `## Working notes (ephemeral)`.

If the user continues in the same session, proceed from the approved IDR.
