# Explore Next Question Buffer Design

**Date:** 2026-04-20

**Scope:** Explore / Journey test flow only. Other question flows remain separate.

## Goal

Make `Next question` in Explore feel instant by ensuring the next question is already prepared before the learner clicks.

## Problem

Explore currently treats the active question, answer review state, and forward navigation as one tightly-coupled state bundle inside `JourneyHome.vue`. Even with cached KaTeX and shallow question storage, the click still has to drive a visible state transition on the critical path.

The result is a button that can feel late because the UI is still doing work at the moment of navigation instead of before it.

## Chosen Architecture

Use a local double-buffered Explore runner.

- Keep Explore separate from Session, CoachHub, and game runners.
- Maintain two question slots inside Explore:
  - `active`: the visible question the learner is answering or reviewing
  - `standby`: the next question, already staged in the background
- On test start:
  - load all raw session questions as today
  - mount question `0` as `active`
  - mount question `1` as `standby` if it exists
- While the learner is reviewing the explanation for the current question, the standby question stays mounted but hidden.
- On `Next question`:
  - clear review state
  - swap `standby` into `active`
  - immediately refill the old slot with the following question

This makes the click path a slot swap instead of a fresh question render.

## Non-Goals

- No consolidation with other test/session surfaces
- No change to explanation style or wording
- No waiting for save completion before normal question-to-question movement

## Data Model

Introduce a small Explore-only buffer model:

- `activeSlot`: which of the two slots is visible
- `slotA`: prepared question stage or `null`
- `slotB`: prepared question stage or `null`
- each stage stores:
  - question index
  - raw question object
  - stable slot id

The buffer is pure state logic and should live outside `JourneyHome.vue` so it can be tested in isolation.

## UI Behavior

- The visible question pane uses the active slot.
- The standby question pane is mounted but hidden with non-interactive styling.
- The review panel remains tied to the active question only.
- Clicking `Next question` must not wait for:
  - answer submission completion
  - explanation parsing
  - KaTeX cache warming
  - next-question DOM creation

Only the final `Finish test` action is allowed to wait for session flush and completion.

## Error Handling

- If answer submission fails, keep the unsaved payload for end-of-test flush as today.
- Do not block normal forward movement on non-final questions.
- If the standby slot is unexpectedly empty on a non-final question, rebuild it synchronously from raw questions as a safety fallback.

## Testing

Add pure tests for the Explore buffer:

- initializes with question 0 active and question 1 staged
- advances by swapping active and standby
- refills the inactive slot with the following question
- leaves the final state stable when there is no further question to stage

Then verify the frontend with:

- `pnpm exec vue-tsc --noEmit --pretty false`
- `pnpm build`
