# Shared public construction and captured projection

This is correctness and diagnostic evidence for source commits `48a29aa` and
`1906d683476b7cff1d76221a02a2591ce6d424b1`, compared together with the preceding
arena packet runtime `9ae8142`. The changes share the construction storage
interface and make Sankey's importer consume a captured read interface. They do
not establish a preparation speedup. All six measured Sankey parse rows and all
six retained-render rows were slower in the VM comparison.

## Results and scope

The main VM diagnostic has 30 conditions, each with 21 balanced three-way trials
and 1000 ms warmup per implementation. The separate first-use diagnostic has 24
conditions, each with 15 trials and 1000 ms warmup. Fixed and Canvas resource
lanes, all samples, matched batch counts, exact output checks, and ER/Sequence
controls are retained. [RESULTS.md](RESULTS.md) includes every row; percentages
are medians of per-trial candidate/control ratios, not ratios of medians.

- Sankey parse: all six rows adverse, +0.514% to +6.224%.
- Sankey retained render: all six rows adverse, +0.038% to +5.446%.
- Main source-render controls: mixed. Sankey small Canvas was +7.163%, with
  candidate lower in only 3/21 pairs. Sequence retained controls were +2.169%
  and +5.330%; ER Canvas retained was +2.638%.
- First unchanged Sankey large fixed render was +19.037%, with only 2/15 lower
  pairs. Large Canvas was -0.309% (8/15). Large edited renders were +1.380% and
  +2.548% (4/15 each). Small fixed edited render was -4.218% (12/15).
- First-use Sequence edited controls were +1.386% and +1.572%; its source
  controls were +0.497% and +1.186%. The adverse results are not dismissed as
  noise, and the two VM reports are not pooled.

One authorized local Swift attempt had a total 180-second cap and no retries.
Two complete Sankey/medium conditions passed all guards: retained render
-0.4776% (9/15 lower), and lifetime +0.0043% (7/15 lower). Lifetime's individual
paired ratios ranged from 0.564 to 1.221; its ratio of separate medians differs
from its paired median. The next parseMutable condition was rejected by the
unchanged global compiler guard when external `swift-build` PID 10769 appeared,
even at 0% CPU. Its persisted `completed:false` report is excluded in full.
Immutable parse was not attempted. The attempt ended after 26.5 seconds; no
retry, partial-condition pooling, or construction/parse latency claim follows.
Accepted local pairs have 15/15 aggregate control/candidate first order. Host
process snapshots record substantial external app load: passing the compiler
guard does not mean an idle machine.

WASM size changed from 8,157,169 to 8,170,207 bytes (+13,038); gzip from 2,490,637
to 2,494,023 (+3,386); Brotli from 1,714,300 to 1,715,868 (+1,568).

## Correctness and identity

- Construction integration slice: 18 passed. Projection slice: 20 passed.
  These are separate targeted runs with overlapping coverage, not 38 unique
  tests. The repaired baseline importer slice passed 10 before adoption.
- Focused browser checks: 24 passed across the three configured engines. The
  previous full 417-check suite belongs to `9ae8142`, not this source batch.
- Fresh host SDK and one Swift consumer build: 16 selected integration tests
  passed, including 12,481 drawings, 16,572 expected errors, and zero explicit
  budget differences. All 168 consumer API conditions over 21 cases passed;
  all three C consumers compiled with strict warnings and passed.
- All 662 frozen source hashes bind to immutable Git `1906d68`. The Swift freeze
  contains 353 inputs, including 340 Git-backed inputs; all source and SDK
  hashes remained unchanged through correctness. Against arena control, the
  changes are five core source paths (including two new files) plus the SDK
  archive. No Swift consumer source, web facade, or C ABI change is included.
- Frozen local package files were physically rehashed: 106 per side, matching
  both VM reports. This is a local-copy check, not a separate remote physical
  rehash. Hash mapping does not establish reproducible builds.

The baseline Unicode fixture initially failed because an unquoted source label
was not valid parser input. The repaired fixture uses a direct typed oracle;
its 10-test baseline passed. The first projection compile/run was interrupted
for the reviewed own-fields scan correction; the final 20-test log is the
accepted run. Initial logs remain alongside repaired logs. These failures are
not represented as passing executions.

Candidate Swift binary: `16900d32f60685e93d92a1e697650d3fb34e5e0f63be6ad7c5be9c8305fcfdbf`.
Control binary: `2e1491d5b80da5d64b7f042abe934f47d662901d9d8037eb1646a0d7a03cff9a`.
Fresh source/SDK aggregate: `d898df768e0bab3d3f2f21848a79d1ac9b32ade7cf6756b4643c14c4824ce7fc`.
SDK archive: `6cef08bdbbd45cd21db6d302b606968f2bd70c72a3fc5cf687a1c8bdb15b2fe1`.

## Recheck saved evidence

Run from this folder:

```sh
python3 summarize.py
```

This portable standard-library script executes no benchmark, build, browser,
repository import, or network operation. It checks raw paired arithmetic,
output checks, warmups and orders, frozen hashes, logs, and accepted/rejected
Swift reports; then regenerates `summary.json`, `RESULTS.md`, and
`evidence-index.json`. Gzip members use deterministic timestamps. Harness copies
are archival provenance, not portable executable benchmark entry points.

No new full performance gate or release parity result is claimed. VM and local
Swift ratios cannot be combined into cross-host speed arithmetic. The combined
source batch and recorded host conditions limit causal attribution to either
individual primitive. This remains pre-0.3 architecture work.
