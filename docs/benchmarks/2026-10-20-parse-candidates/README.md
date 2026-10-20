# Rejected web parse candidates

These experiments remain outside production. No package below is a release
candidate or evidence of JS parity. Independent Linux VM, Chromium 156.0.8078.4;
latency diagnostics use 15 balanced trials, 1000 ms measured warmup, matched WASM
batch counts and exact output before/after. Reports retain package/oracle/harness
hashes and raw samples. Separate medians and paired medians are not interchangeable.

## Host text pool

The [host-text patch](host-text.patch), with [input hashes](host-text-inputs.json), replaced duplicate numeric UTF-16 arrays with strings
inside the private host bridge/codec, retaining the ordinary public graph wire
contract. [Parse comparison](host-text-comparison.json) gives mixed results.
[Whole parse plus first use](host-text-first-use.json) does not establish a useful
gain: Flowchart first unchanged render has an 18.7% higher separate median, while
Gantt method/callback directions remain small and mixed. Behavioral checks passed,
but they do not establish a performance win. The representation change was rejected.

## Single native ownership index

The [one-file patch](ownership-index.patch) combines native object ID and owner
registration into one WeakMap record. Caller-created and borrowed objects keep
a separate document-local index. Exact raw strings, descriptors, prototypes,
dispatch owners, publication and disposal remain on their existing paths. Two
new cross-document/callback/disposal feature slices pass for both packages.

[Parse/stage comparison](ownership-comparison.json) and
[whole first use](ownership-first-use.json) show mixed results.
The [independent paired audit](ownership-paired-audit.json) weakens the apparent
Gantt first-render gain: separate medians suggest -9.75%, but the paired median
is -2.26%, with 11/15 wins and a middle half spanning both directions. Gantt
callback is +0.10% paired; Sequence source control is +3.90% paired (13/15 slower).
Flowchart edited render is the strongest row (-5.35%, 13/15 wins). These do not
establish a broad parse or lifecycle improvement.

[Live-heap comparison](ownership-heap.json) uses one browser instance, separate
contexts/pages with distinct V8 isolate IDs, forced GC and 15 balanced pairs for
32/64 normal public documents. It samples initial, parsed-held, rendered-held,
disposed-but-caller-held and released phases, checking every retained output.

For 64 held documents, paired median additional parsed JS heap is:

| Workload | Candidate minus control |
| --- | ---: |
| Sequence small | -6,956 bytes |
| Flowchart medium | +115,300 bytes |
| ER medium | +40,268 bytes |
| Gantt medium | +85,244 bytes |

The additional record objects increase measured retained JS heap in three of
four workloads; a smaller WeakMap count did not imply lower memory use. Reserved
heap, embedder heap, backing storage and exported WASM capacity remain separate
raw counters. Backing storage can overlap WASM capacity; neither is Rust live
allocation bytes or RSS. Released residuals and baseline drift are disclosed.

The candidate was rejected: weak/mixed latency evidence and higher retained JS
heap do not justify the added ownership lookup complexity. Production retains
the established identity maps. Profiling a statement containing multiple map
operations does not identify either map as the cause of all sampled cost.
