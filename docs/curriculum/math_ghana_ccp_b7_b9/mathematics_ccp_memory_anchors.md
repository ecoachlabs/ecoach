# Mathematics CCP Memory Anchors (B7-B9)

Source PDF: `C:/Users/victo/Downloads/MATHEMATICS.pdf`

## Memory Backbone

- `A00-GLOBAL-METADATA`: `P001-P004` (cover, foreword, acknowledgements)
- `A01-CONTENTS-ROADMAP`: `P005-P007` (full strand/sub-strand page map)
- `A02-PHILOSOPHY-AND-AIMS`: `P008-P017` (intro, rationale, philosophy, aims, learner profile)
- `A03-ASSESSMENT-AND-PEDAGOGY`: `P018-P029` (assessment model, pedagogy, competencies, instructional expectations)
- `A04-STRUCTURE-COUNTS`: `P030` (official coverage table by strand/sub-strand across B7-B9)
- `A10-B7-CURRICULUM`: `P031-P118`
- `A20-B8-CURRICULUM`: `P119-P193`
- `A30-B9-CURRICULUM`: `P194-P250`
- `A40-APPENDIX-CORE-COMPETENCIES`: `P251-P255`
- `A41-BIBLIOGRAPHY-AND-CREDITS`: `P256-P259`

## Coverage Snapshot (from P030)

- Total content standards by grade (table value): B7 = 19, B8 = 18, B9 = 17
- Strand set: Number, Algebra, Geometry and Measurement, Handling Data
- Sub-strand set (12 total):
  - S1.SS1: Number and Numeration Systems
  - S1.SS2: Number Operations
  - S1.SS3: Fractions, Decimals and Percentages
  - S1.SS4: Number: Ratios and Proportion
  - S2.SS1: Patterns and Relations
  - S2.SS2: Algebraic Expressions
  - S2.SS3: Variables and Equations
  - S3.SS1: Shapes and Space
  - S3.SS2: Measurement
  - S3.SS3: Position and Transformation
  - S4.SS1: Data
  - S4.SS2: Chance or Probability

## Extracted Code Inventory

- B7: 21 content standards, 69 indicators
- B8: 18 content standards, 58 indicators
- B9: 18 content standards, 52 indicators

## Plug/Socket Relationship Model

- `Plug: content_standard_to_indicator` -> each content standard fans out into measurable indicators.
- `Plug: next_standard_same_grade` -> sequential build-up within the same grade and sub-strand.
- `Plug: next_grade_progression` -> vertical progression from B7 to B8 to B9 for same strand/sub-strand/standard index.
- `Socket: indicator_requires_content_standard` -> each indicator depends on parent standard meaning.
- `Socket: previous_standard_same_grade` -> a standard consumes prior standard in sequence.
- `Socket: prior_grade_foundation` -> a grade-level standard consumes lower-grade foundation.

## Seed Data Artifacts

- `mathematics_ccp_pages.jsonl`: lossless page-level source extraction with anchors.
- `mathematics_ccp_fulltext.md`: human-readable anchored full text.
- `mathematics_ccp_structured_graph.json`: extracted standards/indicators graph with plugs/sockets.
- `mathematics_ccp_seed_package.json`: seed-ready topic + indicator package for ingestion/testing.

## Retrieval Strategy

- For exact provenance, retrieve via page anchor (`P###`) from `mathematics_ccp_pages.jsonl`.
- For topic graph and dependencies, use `mathematics_ccp_structured_graph.json`.
- For direct DB/UI seed workflows, use `mathematics_ccp_seed_package.json`.
