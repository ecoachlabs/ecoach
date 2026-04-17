# Mathematics CCP B7-B9 Curriculum Memory Pack

This folder stores a durable, source-anchored memory pack built from:

- `C:/Users/victo/Downloads/MATHEMATICS.pdf`

## Files

- `mathematics_ccp_metadata.json`
  - Source metadata (publisher, publication date, page count).
- `mathematics_ccp_pages.jsonl`
  - Lossless page-level extraction with anchors (`P001` to `P259`).
- `mathematics_ccp_fulltext.md`
  - Full anchored text for human review.
- `mathematics_ccp_fulltext.txt`
  - Plain text variant of the full extraction.
- `mathematics_ccp_page_anchors.md`
  - Fast page preview index for retrieval by anchor.
- `mathematics_ccp_structured_graph.json`
  - Parsed curriculum graph (content standards, indicators, relationships).
- `mathematics_ccp_seed_package.json`
  - Seed-oriented topic and indicator package for ingestion/testing.
- `mathematics_ccp_memory_anchors.md`
  - High-level anchor map + plug/socket relationship model.
- `mathematics_ccp_topic_atlas.md`
  - Grade-by-grade topic atlas for quick navigation.

## Anchor Strategy

- Every source fact should map back to a `P###` anchor.
- Use `mathematics_ccp_pages.jsonl` when exact provenance is required.
- Use `mathematics_ccp_structured_graph.json` for dependency-aware traversal.

## Plug/Socket Model

- Plug = what a node enables next (indicators, next standards, next-grade progression).
- Socket = what a node depends on (parent standards, prior standards, lower-grade foundations).

This model is intended to support interactive curriculum rendering and prerequisite-aware testing flows.
