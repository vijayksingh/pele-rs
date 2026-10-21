# ER resource callback evidence

One correctness diagnostic, no timings. Frozen package `parse-architecture/normalized-er/package` on existing VM workspace; parent identifies this as4e9. Exact package content is recorded in the raw report, including WASM SHA256 `bc7da469d35ec7c46fb718c7d3c66c10bbd9c14e8bad9d82eec4e522ff6a6c2b`. Browser `156.0.8078.4`. Pinned bundled oracle SHA256 `d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d`. Raw report SHA256 `4d844b4f3c9298c249ef5326a320c6af7b73478938068f751db701bb0612c672`. Local/remote reproducer SHA256 `97bb2719fda8305b61ec6f6aba22dba0ddddab2204c081e0a8bc4081308178ce`. ESM ER oracle chunk SHA256 `1ab6f66e449e3c68a5c8ac476b8a921e1634316122b512c5b963344c0bce066d`; it was read to explain phases, while the diagnostic imported the pinned full bundled oracle.

Executed commands from `/Users/vijaysingh/unlocalhosted/pele-rs`:

```sh
scp -o RemoteCommand=none dist/perf-patches/parse-architecture/direct-er/resource-callback-diagnostic.mjs vm-agent:/home/admin/pele-perf.jL9wzP/resource-callback-diagnostic.mjs
ssh -o RemoteCommand=none -T vm-agent 'cd /home/admin/pele-perf.jL9wzP && PLAYWRIGHT_BROWSERS_PATH=/home/admin/pele-perf.jL9wzP/browsers timeout 75s node resource-callback-diagnostic.mjs'
scp -o RemoteCommand=none vm-agent:/home/admin/pele-perf.jL9wzP/parse-architecture/normalized-er/resource-callback-diagnostic.json dist/perf-patches/parse-architecture/direct-er/resource-callback-diagnostic.json
```

The script started one Chromium browser, closed it in finally, imposed its own60-second deadline, and completed exit0. It verified all regular package file hashes unchanged before/after. Twelve variants, two runtimes, first and next render; no reruns. Initial full graph signatures matched in every variant. The no-mutation control additionally matched both whole outputs, both post-state signatures, identity checks and all20 width calls. This validates fixture comparability before interpreting mutations.

Read `resource-callback-summary.json` for each output hash, dimensions, width-call count and identity result; raw JSON preserves whole SVGs, signatures, arguments and errors. `RESOURCE_CALLBACK_PUBLICATION_LLD.md` records the concrete renderer phase audit and why publication-only repair is insufficient.
