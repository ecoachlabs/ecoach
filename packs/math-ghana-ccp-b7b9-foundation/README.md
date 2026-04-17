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
  - 97 misconception records
  - 56 prerequisite edges
- Seedable knowledge base:
  - 57 explanations
  - 12 glossary entries
  - 14 formula entries
  - 57 worked examples

Question families/questions are intentionally empty in this foundation pack (`question_count = 0`) so content-generation pipelines can seed high-quality items on top of a stable curriculum graph.

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
