# Rejected computed-property record literals

**Rejected; the dense-array package was restored.** Generic computed-property object literals replaced fresh object allocation plus individual property assignment for eligible 3/4/6-field records. Eligibility allowed only strings and earlier node references; numeric values and other atoms retained the existing path. This passed public parity checks but slowed every ER parse workload.

The VM comparison preserves all 24 rows, three implementations and 21 samples per implementation per row: **1,512 batch-mean samples**. All six ER parse rows regress by **+8.84% to +11.74% paired median**, with only **2–6 candidate wins out of 21**. Ratio-of-independent-medians changes are +6.49% to +11.96%; these are different statistics. All sequence, retained-model and source-render controls remain in `comparison.json.gz` and `summary.json`. ER retained-render paired changes range from −3.57% to +1.59%, insufficient to offset the parse regression.

| ER parse | Paired median change | Wins / 21 |
| --- | ---: | ---: |
| small, fixed | +10.00% | 3 |
| small, canvas | +8.84% | 6 |
| medium, fixed | +10.57% | 2 |
| medium, canvas | +11.74% | 2 |
| large, fixed | +10.67% | 2 |
| large, canvas | +10.70% | 4 |

`browser.log.gz` records **84 public checks passing: 28 each in Chromium, Firefox and WebKit**, using one worker. This is the focused feature suite, not a new full-suite claim. No first-use experiment followed the clear ordinary-parse regression. No native/Swift build, timing or new native correctness run belongs to this experiment.

## Identity and rollback

- Dense runtime: `770113546dd302a60b52707194bbda40bb9e070c`. Candidate applied privately over documentation HEAD `7ceb15edab31503e41493b9d0b67d0bfe0db06b9`; the runtime experiment was not committed.
- Rejected patch: `c5a80eb5362bf708386908ad2811b06bb21e625ba76838bcf18da222278cd51c`. The preserved pre-execution manifest/review describe static acceptance, not final adoption.
- Control WASM: `6567d1a9ca51488928fd5940c7dc53afb85038c181d6278b84f55fcb0d4a1f8e`.
- Candidate WASM: `d9a7de8e53354922df7878822911809f3e70ce155b1dcdb86d8c110ed9589858`.
- All **106 files per lane** match the frozen packages and VM runtime maps. Only `host-er-store.js` and `pkg/pele_bg.wasm` differ between packages.
- At audit HEAD `6dfa9cac0ed7cf92ee4e5e7cacec0c1e1aa9268e`, the worktree was clean. Restored distribution matches all 106 dense control hashes. The two reverted runtime source files match dense commit blobs. Among 655 candidate source inputs, these are the only differences.
- All **351 native/consumer/SDK input hashes** still match the existing build manifest (`439fe2e6a023eebcafb980508b447162cdd5cd9c07519200df47f5ab3fcaea28`), as do the native archive and Swift executable. Literal changes touched only WASM construction and web publication.

`verification.json` records independent local rehashes and exact paths. The **raw Cargo WASM target was still reported to contain the rejected build** when the distribution was restored; its observed hash is recorded separately. This audit did not rebuild it or claim that raw target matched restored source.

## Reproduce arithmetic

Run `python3 summarize.py` in this folder. It reads saved data, checks row/sample counts and median arithmetic, and emits `summary.json` content. No browser or benchmark runs. Original source/package freezes, all runtime hashes, build/browser/VM logs and the rejected patch are retained. The sibling `profile/` folder contains six fresh **dense control** profiles, not literal-candidate profiles; these cannot explain the regression by direct before/after attribution.
