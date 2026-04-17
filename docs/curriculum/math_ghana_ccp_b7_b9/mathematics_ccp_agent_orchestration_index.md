# Mathematics CCP Agent Orchestration Index

This file coordinates the multi-agent outputs used to move from source curriculum extraction to production-ready curriculum UX and test seeding.

## Core Inputs

- `mathematics_ccp_pages.jsonl`
- `mathematics_ccp_structured_graph.json`
- `mathematics_ccp_seed_package.json`
- `mathematics_ccp_memory_anchors.md`

## Agent Output Slots

- QA and reliability report:
  - `Parfit` (explorer, read-only report in chat output)
- Education semantic enrichment:
  - `mathematics_ccp_education_semantic_map.json`
  - `mathematics_ccp_education_semantic_map.md`
- Interactive curriculum blueprint:
  - `mathematics_ccp_interactive_blueprint.json`
  - `mathematics_ccp_interactive_blueprint.md`

## Integration Intent

- Keep curriculum truth anchored to original page anchors (`P###`) and official curriculum codes.
- Add education-centered semantics for instructional decisions, assessment design, and misconception-aware practice generation.
- Add interaction blueprint metadata for animated curriculum presentation, drill-down transitions, and dependency-aware navigation.

## Read Order

1. `mathematics_ccp_memory_anchors.md`
2. `mathematics_ccp_structured_graph.json`
3. `mathematics_ccp_seed_package.json`
4. `mathematics_ccp_education_semantic_map.json`
5. `mathematics_ccp_interactive_blueprint.json`


## Integrated Outputs

- `mathematics_ccp_qa_findings.md`
- `mathematics_ccp_education_semantic_map.json`
- `mathematics_ccp_education_semantic_map.md`
- `mathematics_ccp_interactive_blueprint.json`
- `mathematics_ccp_interactive_blueprint.md`
