# Math Ghana CCP B7-B9 Foundation Pack

This pack seeds the curriculum foundation for Ghana NaCCA Mathematics CCP (B7-B9) using the anchored source artifacts in:

- `docs/curriculum/math_ghana_ccp_b7_b9/`

## Scope

- Curriculum hierarchy:
  - 4 strand nodes
  - 12 sub-strand nodes
  - 57 content-standard topic nodes
- Intelligence foundation:
  - 179 academic nodes (indicator-backed)
  - 180 learning objectives
  - 171 misconception records, ensuring at least 3 diagnostic misconception patterns per content-standard topic
  - 56 prerequisite edges
- Seedable knowledge base:
  - 57 explanations
  - 12 glossary entries
  - 14 formula entries
  - 57 worked examples
- Analytics-ready question bank:
  - 342 question families
  - 2,850 authored MCQ questions
  - Exactly 50 questions per content-standard topic
  - 8,550 wrong-answer options mapped to misconception records with explicit `Misstep`, `Reveals`, and `Needs attention` diagnostics
  - Question stems, option text, answers, and explanation payloads prepared with inline LaTeX markers for mathematical notation
  - Detailed solution breakdowns for every question, including commentary, step-by-step reasoning, correct-answer rationale, and per-option diagnostics
  - One intelligence record per question across knowledge role, cognitive demand, solve pattern, pedagogic function, content grain, family, and misconception exposure

Question families/questions are generated deterministically from the anchored curriculum graph by:

```powershell
node scripts/generate_math_ccp_questions.mjs
```

The generator uses `docs/curriculum/math_ghana_ccp_b7_b9/mathematics_ccp_seed_package.json` plus the pack's objectives, academic nodes, and misconception records. Re-running it should reproduce `curriculum/misconceptions.json`, `questions/families.json`, `questions/questions.json`, `questions/intelligence.json`, and the manifest question count.

Question quality is checked by:

```powershell
node scripts/validate_math_ccp_questions.mjs
```

## Install

Use the existing content pack install command flow (`install_pack`) with this folder path.

Example path:

- `packs/math-ghana-ccp-b7b9-foundation`

CLI option (local runtime DB file):

```powershell
cargo run -p ecoach-commands --bin install_pack -- "<db_path>" "C:\Users\victo\OneDrive\ecoach\packs\math-ghana-ccp-b7b9-foundation"
```

## Source Fidelity

- Topic descriptions preserve source-anchor breadcrumbs (`P###`) for traceability.
- Known extraction anomalies are tracked in:
  - `docs/curriculum/math_ghana_ccp_b7_b9/mathematics_ccp_qa_findings.md`
