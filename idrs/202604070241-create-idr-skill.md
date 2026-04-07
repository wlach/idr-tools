# 2026-04-07: Create idr skill

Owner: Will Lachance <wlach@protonmail.com>

## Overview

### Problem Statement

Create an agent skill for working with Implementation Decision Records (IDRs). The skill should support a human-led workflow where the operator owns the problem framing, goals, non-goals, and tradeoffs, while the agent helps scaffold, critique, implement, test, and record what actually happened.

### Context (as needed)

`idr-tools` provides an `idr` CLI for creating markdown IDRs in a repository. IDRs are meant to capture implementation intent and execution context without becoming permanent system documentation.

The skill is partly inspired by the [Superpowers] skill model: reusable, triggerable process guidance for coding agents. However, this skill should not adopt the full Superpowers premise that the agent should usually drive design discovery. The desired workflow is more opinionated and human-owned:

1. Create an IDR scaffold from a human task description.
2. Have the human manually fill in the main design sections.
3. Use the agent to help develop requested sections and critique the IDR according to the user's requested review mode.
4. Iterate until the human is happy with the IDR.
5. Implement from the approved IDR.
6. Add tests, run verification, write docs as needed, and update the IDR.

Strong models are useful in this workflow because they are good at finding ambiguity, missing risks, unclear goals, and mismatches between a proposed solution and implementation constraints. The agent should act as a critic and executor, not as the owner of the design.

### Goals

- Add a reusable `idr` skill to this repository.
- Make the skill explicitly human-led: the user owns the strategic sections of the IDR unless they ask the agent to write them.
- Support staged use: create scaffold, wait for human edits, develop and critique, implement, verify, update implementation notes.
- Make command invocation adaptable across environments where `idr` may be on `PATH`, available via `uv run idr`, or available via `uvx --from idr-tools idr` when a project-local installation is not present.
- Prefer deterministic explicit activation over broad automatic activation.

### Non-Goals

- Force IDRs for every small change.
- Automatically install `idr-tools` into a project or add it as a dependency.
- Build a hard state machine; skills are agent instructions, not a workflow engine.
- Replace human judgment about design, tradeoffs, or scope.

### Proposed Solution

Create a top-level skill at `skills/idr/SKILL.md`.

The skill should use a narrow trigger, for example:

```yaml
name: idr
description: Use when the user explicitly asks to create, review, critique, implement from, or update an Implementation Decision Record
```

The skill should define an explicit staged workflow with stop points. It should create an IDR scaffold when requested, then stop and ask the user to manually fill in the main design sections. When the user asks for help developing or critiquing the IDR, it should work in the requested mode. When the user asks for implementation, it should treat the approved IDR as source of truth, implement within scope, add tests, verify, write docs as needed, and update `## Implementation (ephemeral)` with factual notes.

The `## Implementation (ephemeral)` section is the main section the agent should expect to update after doing the work. Humans can write implementation hints there if useful, but in the agent-assisted workflow the section is primarily where the agent records what actually happened.

## Detailed Design (as needed)

### Location

Use:

```text
skills/idr/SKILL.md
```

For local use, the skill can be symlinked into agent-specific skill directories:

```sh
ln -s /path/to/idr-tools/skills/idr ~/.agents/skills/idr
ln -s /path/to/idr-tools/skills/idr ~/.claude/skills/idr
```

### Skill workflow

The skill should define four stages:

1. **Create scaffold**
   - Resolve an appropriate `idr` invocation.
   - Run `idr new "<title>"`.
   - Report the file path.
   - Stop and wait for the human to fill in the main sections.

2. **Develop and critique IDR**
   - Read the full IDR.
   - Develop an internal critique before drafting new IDR text.
   - If the user requested a specific critique mode, use it.
   - If the user did not request a specific critique mode, perform a concise general review and ask whether they want to go deeper on a particular dimension.
   - Align with the user on the problem statement, context, goals, non-goals, and proposed solution before helping fill in other sections.
   - After alignment, help fill in requested sections such as Other reading or Implementation hints when asked.
   - Support review modes such as "what's missing?", "are the goals clear?", "what are the implementation risks?", "what tests does this imply?", and "is the scope too broad?"
   - Do not rewrite human-owned sections unless asked.

3. **Implement from approved IDR**
   - Re-read the IDR.
   - Treat it as the source of truth.
   - Flag contradictions or unresolved ambiguity before coding.
   - If implementation reveals that the approved IDR is wrong, stale, or incomplete, stop and report the mismatch rather than silently implementing a different design.
   - Implement within the stated scope.
   - Add or update tests.
   - Run relevant verification.
   - Write or update docs when the change affects documented behavior.

4. **Update and verify IDR**
   - Compare the final diff to the IDR.
   - Update `## Implementation (ephemeral)` with factual implementation notes.
   - Flag stale design sections instead of silently rewriting human-owned sections.
   - Summarize verification.

### Authorship boundary

The human owns:

- Problem Statement
- Context
- Goals
- Non-Goals
- Proposed Solution
- Alternatives considered

The agent may:

- Create the file scaffold.
- Ask clarifying questions.
- Identify ambiguity, contradictions, missing risks, test gaps, or scope creep.
- Suggest edits for human approval.
- Implement from the approved IDR.
- Update `## Implementation (ephemeral)` with facts from the work, especially after agent-assisted implementation.

The agent should not silently invent strategic rationale as if it came from the human.

### Command resolution

The skill should not assume `idr` is always on `PATH`. It should prefer the least invasive working command:

1. `idr new "<title>"` if `idr` is available.
2. `uv run idr new "<title>"` if the project already uses `uv` and `uv run idr --help` works.
3. `uvx --from idr-tools idr new "<title>"` if no project-local command exists and the user accepts a non-project-local tool invocation.

If none works, ask the user how to invoke `idr`. Do not install dependencies globally or add `idr-tools` as a project dependency without explicit approval.

### Context handoff

After the IDR is approved, the skill should ask whether the user wants to continue in the same agent session or start a new agent session manually. The skill should not force either choice.

If the user wants a new session, the agent should first capture important implementation-relevant research in `## Implementation (ephemeral)`. If the user continues in the same session, proceed from the approved IDR.

## Cross cutting concerns (as needed)

### Skill activation

The trigger should be narrow. This skill is intended for explicit IDR workflows, not for automatically creating process overhead around every implementation task.

### Workflow reliability

Skills are instructions, not deterministic state machines. The skill should include explicit stop points, but the user should still be able to steer the workflow with plain language.

### Installation

The repository can document symlink-based installation for Codex and Claude Code, but the skill itself should remain agent-neutral.

## Alternatives considered (as needed)

### Make the agent write the IDR

We want to avoid this, as IDRs are meant to track human intent. Allowed only when explicitly requested. The default workflow should be human-authored IDR sections plus agent development support and critique.

## Other reading (as needed)

- [Superpowers]

[Superpowers]: https://github.com/obra/superpowers
