# Seeding Status: Mathematics CCP B7-B9

## Foundation Check

The smart curriculum foundation is active in the codebase and ready to drive content systems:

- Canonical curriculum graph (`curriculum_nodes`, objectives, concepts, relationships)
- Source fidelity layer (citations, exemplars, comments, node intelligence)
- Ingestion workspace and review pipeline
- Publish/snapshot workflow for shared academic truth
- Pack install pipeline that materializes curriculum + seedable knowledge into runtime tables

## Math Ingestion Wiring

A full foundation pack has been generated and wired:

- Pack path: `packs/math-ghana-ccp-b7b9-foundation`
- Pack id: `math-ghana-ccp-b7b9-foundation-v1`
- Curriculum version label: `Ghana NaCCA Mathematics CCP B7-B9 2020`

Pack payload highlights:

- 73 curriculum topics (4 strands, 12 sub-strands, 57 content standards)
- 179 indicator-backed academic nodes
- 180 learning objectives
- 171 misconception records, guaranteeing at least 3 diagnostic misconception patterns per content-standard topic
- 56 prerequisite edges
- Seedable knowledge base entries across explanations, glossary, formulas, and worked examples
- 342 question families
- 2,850 authored MCQ questions
- Exactly 50 seeded questions per content-standard topic
- 8,550 wrong-answer options mapped to misconception records with explicit `Misstep`, `Reveals`, and `Needs attention` diagnostics
- Per-question solution breakdowns with commentary, step-by-step reasoning, correct-answer rationale, and per-option diagnostic meaning
- Inline LaTeX markers for mathematical notation in stems, options, answers, and explanation payloads
- Question intelligence coverage across family, difficulty, cognitive demand, solve pattern, pedagogic function, content grain, source anchors, and misconception exposure

## Verification

Runtime ingestion and idempotence are covered by tests in:

- `crates/ecoach-content/src/pack_service.rs`

Added tests:

- `installs_math_ghana_ccp_foundation_pack_into_runtime_tables`
- `reinstalling_math_ghana_foundation_pack_is_idempotent_for_subject_slice`

Both pass locally. The full foundation-pack install tests now classify all 2,850 questions and can take several minutes.

The question seed files are generated deterministically by:

```powershell
node scripts/generate_math_ccp_questions.mjs
```

The generated question quality contract is checked by:

```powershell
node scripts/validate_math_ccp_questions.mjs
```

## Runtime Install Utility

CLI installer added:

- `crates/ecoach-commands/src/bin/install_pack.rs`

Usage:

```powershell
cargo run -p ecoach-commands --bin install_pack -- "<db_path>" "C:\Users\victo\OneDrive\ecoach\packs\math-ghana-ccp-b7b9-foundation"
```
