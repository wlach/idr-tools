---
draft: true
---

<!--
This is a working draft during development and review, and a record of intent
and decisions afterward. Start with what you know; questions, tentative
explanations, and incomplete sections are welcome. Update it as your
understanding changes.

Approval of the proposed solution allows implementation to begin; the IDR
remains a draft until human or LLM review has at least mostly finished.
Completing implementation or passing tests alone does not mean it is ready
to finalize. Keep the draft and working notes while substantive review can
still reshape the work.

By the time the work is ready to merge, a reader should be able to understand:

* What motivated the change
* What approach was taken
* What might be done in the future

Include technical detail where it helps explain a decision; the diff can
supply routine implementation details.
Aside from the overview, all sections are optional.

Once review is at least mostly finished, finalize through deliberate edits,
either yourself or by asking an LLM to help. If review readiness is unclear,
keep the draft and clarify before finalizing.
Preserve discoveries and changes in understanding that explain the decisions.
Keep the reasoning; omit the experiment log.
Then remove working notes, comments, and empty optional sections.
Remove the draft field rather than setting it to false. Preserve unrelated
frontmatter; remove the block if it becomes empty.

Keep guidance comments during drafting; remove them before merge.
-->

# 2026-10-15: Reduce template comment clutter

Owner: Will Lachance <wlach@protonmail.com>

## Overview

### Problem Statement

Long guidance comments make an IDR harder to read and edit as the author's
own thinking takes shape. The guidance is useful on first load, but repeatedly
wading through it detracts from the drafting experience. Authors should be able
to clear it away without losing access to the expectations for a finished IDR.

### Context (as needed)

The current template explicitly says to keep guidance comments during drafting
and remove them before merge. This reflects a concern that agents might lose
track of instructions across sessions, after context compaction, or during a
long conversation. Keeping guidance in the document makes it available on each
reread, but we have not established that retaining all of it improves behaviour.

The tool already supports generating an IDR without comments through
`--no-comments` or `IDR_NO_COMMENTS`. That skips the useful initial exposure to
guidance; the desired experience is to read it first and discard it as needed.

The IDR skill separately describes the collaboration workflow, including human
ownership of the design and when finalization is appropriate. Some rules matter
throughout the work; expectations for the finished record can be revisited at
finalization.

### Goals

- Let authors remove guidance when it starts getting in the way of writing.
- Preserve the useful guidance available when starting an IDR.
- Make the expectations for a finished record available again at finalization.
- Preserve human ownership and the distinction between drafting and finalization.

### Non-Goals

- Remove or substantially shorten the initial template's guidance.
- Treat removing comments as finalizing the IDR.
- Guarantee agent adherence through comments or repeated instructions.
- Introduce per-section guidance commands as part of this approach.

### Proposed Solution

Keep the full guidance in newly generated IDRs, but explicitly make comments
disposable during drafting. Authors can delete them whenever they get in the
way; doing so does not change the document's draft status or imply that review
is complete.

Update the IDR skill so an agent rereads the template's guidance when asked to
finalize a record. Use that guidance to assess the document's motivation,
approach, and retained reasoning, as well as to perform cleanup. Material gaps
should be surfaced for discussion, rather than filled with invented decisions
or resolved by silently changing the agreed design.

Keep workflow rules needed throughout a session in the skill. Finalization
remains a deliberate step after review has at least mostly finished.

Add `idr template` to print the current template, including guidance comments,
without creating an IDR. Finalization retrieves the guidance with this command.

## Detailed Design

### Template command

`idr template` prints the raw template to stdout. Preserve guidance comments,
frontmatter, and unrendered title, date, and owner placeholders. Do not create
files or require Git author information. Ignore `IDR_NO_COMMENTS`: the purpose
of this command is to retrieve the guidance.

Share the bundled template source with `idr new` so the two commands cannot
silently use different guidance. Custom templates are not currently supported.
The command retrieves the current template, which may differ from the template
used to start an older draft. Its guidance informs review without requiring
new sections or changes to the agreed design.

### Drafting and finalization

Keep the full initial guidance. Replace the instruction to retain comments
throughout drafting with permission to remove them when they get in the way.
Keep draft status and finalization readiness independent of comment removal.
The skill removes a section's guidance comments when filling it in and keeps
comments in blank sections. The template gives the same advice, with an
optional reminder that humans can retrieve guidance with `idr template`.

At finalization, the skill runs `idr template` and reads its full output before
assessing motivation, approach, and retained reasoning. Material gaps require
discussion, rather than invented decisions or silent changes to the design.
Keep the existing human ownership and review-readiness rules in the skill.
Document comment removal and template retrieval in the README.

## Cross cutting concerns

<!--
Describe the design from other angles, as required: security, reliability, etc.

You may have nothing to say here. If so, simply delete this section.
-->

## Alternatives considered

### Generate without comments

Already supported, but removes guidance before the author has a chance to use
it. It does not address the desire to read guidance initially and clear it away
as drafting progresses. This proposal does not require removing that option.

### Replace comments with per-section guidance commands

Comments could direct agents to commands such as `idr explain problem-statement`
before editing a section. This would reduce clutter and allow guidance to be
retrieved as needed, but introduces a command interface and depends on agents
following the references and having the tool available. Whether agents would
treat imperative comments cautiously as possible prompt injection is untested.

### Move all guidance into the skill

This would make the template smaller, but lose useful initial guidance for
humans and make more of the experience depend on an installed agent skill.

### Retain comments throughout drafting

This keeps instructions alongside the text whenever an agent rereads it, but
imposes the reading and editing friction that motivated this change. It does
not guarantee that an agent follows those instructions.

## Future plans

<!--
Describe possible extensions or follow-up work.

You may have nothing to say here. If so, simply delete this section.
-->

## Other reading

- [Draft IDRs](202610111659-draft-idrs.md): established the draft lifecycle and
  the current expectation that guidance comments remain during drafting.

## Working notes (ephemeral)

Open questions:

- What small practical comparison would give us confidence that deleting
  comments preserves the behaviours we care about? Candidate checks include
  human ownership, tolerance of incomplete drafts, and avoiding premature
  finalization, including when resuming in a fresh session.

Implementation notes:

- Added `idr template` using the same bundled source as `idr new`.
- Updated the template, skill, and README for disposable draft comments and
  guidance retrieval at finalization.
- The CLI test checks exact raw output, including comments and placeholders,
  with `IDR_NO_COMMENTS` set, and verifies that no files are created.
