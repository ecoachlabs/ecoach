# Mathematics CCP Education Semantic Map

Companion for `mathematics_ccp_education_semantic_map.json`.

## What It Adds

- Education-centered semantic tags for each topic and indicator.
- Deterministic `learning_intent`, `balance`, `assessment_style_hints`, and `likely_misconceptions`.
- Grade-stage progression intent plus vertical chain positions.
- Explicit `orphan_indicators` section so referential defects are visible (not hidden).

## Reliability Notes

- Heuristics use strict word boundaries to avoid substring false positives (e.g., `rate` inside `demonstrate`, `mode` inside `model`).
- Semantic tags are guidance metadata, not replacements for official curriculum wording.
- Source provenance remains anchored through topic/indicator codes and `P###` anchor arrays.

## Current Snapshot

- Topics: 57
- Indicator pool from source: 179
- Attached indicators: 178
- Orphan indicators: 1
- Progression chains: 18

## Primary Fields

- `metadata`
- `summary`
- `progression_chains`
- `orphan_indicators`
- `topics[]` with embedded indicator-level semantics
