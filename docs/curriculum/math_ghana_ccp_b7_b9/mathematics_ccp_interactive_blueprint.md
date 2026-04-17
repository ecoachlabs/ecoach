# Mathematics CCP Interactive Blueprint

Companion for `mathematics_ccp_interactive_blueprint.json`.

## Purpose

Define a production-ready data contract for rendering the curriculum page as an interactive, animated knowledge map rather than a static PDF dump.

## Scene Model

1. `subject_overview`
2. `grade_focus`
3. `strand_focus`
4. `topic_focus`
5. `indicator_source_drawer`

Each scene includes entry/exit transitions, visible objects, and interaction contracts for deterministic UI state changes.

## Overlay Model

- `seed_readiness`
- `grade_progress`
- `dependency_arcs`
- `source_provenance`
- `progression_compare`

## Data Contracts

- `data_index.topic_nodes` carries canonical curriculum objects and anchors.
- `data_index.topic_dependency_edges` supports arc drawing and prerequisite highlighting.
- `data_index.progression_chains` enables B7->B8->B9 vertical comparison.
- `qa_anomalies` keeps integrity defects visible in the UI for admin review.

## QA Signals Captured

- Count mismatches between official table (`P030`) and extracted totals.
- Orphan indicators (missing parent topic).
- Truncated/noisy topic statement candidates.
- Encoding defects (replacement-character contamination).
