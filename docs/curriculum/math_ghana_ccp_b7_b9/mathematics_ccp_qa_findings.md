# Mathematics CCP Extraction QA Findings

This file captures high-priority integrity findings from delegated QA review.

## Findings

1. Critical: orphan indicator-parent link
- Indicator `B9.1.2.3.4` points to missing parent topic code `B9.1.2.3`.
- Source context indicates it appears under `B9.1.2.4` pages.

2. High: truncated topic statements
- Multiple extracted topic statements end mid-sentence, reducing quality for UI/search/seed generation.

3. High: official totals vs extracted totals mismatch
- Official table (P030): `B7=19, B8=18, B9=17`.
- Extracted graph: `B7=21, B8=18, B9=18`.

4. Medium: encoding contamination
- Some entries contain replacement-character artifacts and symbol distortion.

5. Medium: text hygiene defects
- Spacing/punctuation artifacts remain in canonical text fields.

## Recommended Gate Before Production Seeding

- Reject when orphan indicators exist.
- Reject when topic or indicator code graph is not referentially closed.
- Warn/fail on count mismatches unless explicit adjudication note is attached.
- Warn/fail on replacement-character detection.
- Warn on truncated statement heuristics requiring manual confirmation.
