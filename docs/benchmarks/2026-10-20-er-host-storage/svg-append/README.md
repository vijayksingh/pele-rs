# Shared SVG append sink, Sankey pilot

Source **e1486055d596d4e0ac25e4c78f1d5d95a73f781a**. A borrowed sink appends coordinates, numbers and escaped text into existing paint layers. Existing owned-string helpers share the same formatting core. Sankey no longer allocates a string for each coordinate or whole SVG element. Measurement order, separate paint layers, title/root rendering and final budget checks stay in place. No public API or browser selector changes.

## Matched VM results

Default OptS candidate versus indexed-source default `31dab0b`; pinned Pele 1.6.0. Chromium156 on vm-agent. Same113file sets; only WASM differs. Warm and first-use each retain all24rows,21balanced pairs per row, fixed/canvas resources, Sankey3sizes plus Flowchart medium. Complete samples, source controls and both paired/median definitions are preserved. No cross-window compounding.

| Sankey path | Paired candidate/control change | Candidate / Pele JS |
| --- | --- | --- |
| Warm source render | −26.04% to −18.74% | 0.993–1.578× |
| Warm retained render | −18.14% to −14.43% | 1.78–2.79× |
| First unchanged render | −12.79% to −5.51% | 2.20–5.11× |
| First edited render | −8.98% to −5.69% | 3.77–7.49× |

Edited first-use adds a cyclic extension property and checks retained identity; it is not a diagram-content edit. The separate first-use source controls are1.03–1.54×JS. Large/canvas warm source is near JS in this window; this is not general parity.

**Adverse controls:** five of six warm Sankey parse rows are1.81–4.69% higher; parse remains6.44–15.06×JS. Flowchart warm retained rows rise2.06–2.63% paired, with canvas ratio-of-lane-medians+9.32%. Five of six first-use Flowchart controls are higher, including canvas source+3.13%. No default-path neutrality claim. See [warm table](WARM.md), [first-use table](FIRST_USE.md) and complete JSON/provenance.

## Correctness

- **24 native feature integrations** pass: public Sankey/owned parsing, complete11,616-scenario frozen source corpus,31 numeric-boundary vectors and115 complete SVG-root vectors through public render lanes.
- **12 browser checks** pass across Chromium, Firefox and WebKit: Sankey corpus, first-render thrown-value/alias recovery, numeric boundaries and SVG roots. All591 source inputs and113package files stay unchanged and map to the source commit. Focused checks, not a new full-suite claim.
- Fresh arm64 SDK passes **Swift16, strict C3 and API168**, including12,481drawings and16,572expected errors with zero explicit budget differences. All364source/SDK inputs stay unchanged through correctness and map to source. No new Swift timing or allocation run. [Artifact/source bindings](swift-correctness.json).

Raw text logs have trailing blank lines removed; JSON samples are unchanged. Manifest hashes bind copied evidence. Historical full126-condition release gate remains failing; Intel SDK/AppKit and broader native edge conditions remain release gates.

**No0.3 release.** Next parse architecture replaces duplicated generic graph construction/publication with indexed source-backed retained state and sparse edits; it must preserve complete helpers, arbitrary edits, callback restoration and canonical admission before default use.
