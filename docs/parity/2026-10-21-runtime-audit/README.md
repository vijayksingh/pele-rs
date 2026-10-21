# Functional runtime audit — 2026-10-21

**Full public API parity is incomplete.** All 33 diagram families have native
parsers/renderers and mutable adapters; successful source corpora do not establish
all mutable runtime behavior.

Subsequent work implements all 22 exposed root constructors plus ClassMember;
fresh Rust, Swift, C and three-browser evidence is recorded in the
[ordered constructor checkpoint](../2026-10-21-ordered-constructors/README.md).
The audit observations below remain the original `d7caf8c` results. They are not
a current claim that constructor stubs remain.

At runtime `d7caf8c684ac6b6fda3918a2bcbd91c5953b92c9`, actual public per-type
prepared-entry comparisons against pinned Pele find:

| Public behavior | Observed result |
| --- | --- |
| First width callback edits title then throws its original object | 31/33 native entries lose the edit; all oracle entries retain it |
| Original thrown object and callback count | Both lanes preserve identity and call width once in all 33 cases |
| Same callback edit in Sankey and ER | Retained in this specific case; not a universal callback certificate |
| Exposed nonplain model constructors | 22/22 oracle constructors succeed; native stubs throw |
| Info title containing a lone UTF-16 surrogate | Oracle renders the original code unit; native guard rejects it |
| Radar sparse axes or empty circle geometry | Oracle throws TypeError; current native feature tests assert a different Semantic error |

[CHECKLIST.md](CHECKLIST.md) covers all 33 families and separates dynamic/string
limitations, declared helpers, constructors, callback phases and distribution.
Several existing integration tests explicitly expect an unsupported error and
then verify repair. They are useful recovery tests, not completed parity proof.
C/Swift Sankey same-document callback reentry is being implemented; this audit
makes no claim that its pending native tests pass.

## Evidence and reproduction

- `web-entry-report.json.gz`: exact original 33-family package/oracle report,
  including all 117 package-file hashes, 124 oracle-entry hashes, fixture hash and
  executed script hash. No binaries are duplicated here.
- `audit-original.mjs`: exact executed script, preserving its original local
  package paths and manifest dependency.
- `audit-portable.mjs`: same public operation loop, with explicit input paths and
  before/after hash checks against the recorded report. This adaptation has only
  been syntax checked; it is not presented as the executed original.
- `oracle-probe.mjs` / `oracle-report.json.gz`: pinned full-bundle reproduction of
  Info/Radar behavior and constructors for all family representatives. Complete
  successful SVGs and their UTF-16 code units are retained.
- `source-guards.json.gz`: exact source paths, line numbers and explicit
  unsupported guards from the audited source. Line numbers are historical.
- `manifest.json`: compressed-file and decompressed-byte hashes. Compression is
  lossless; original raw evidence remains unchanged.

Verify the portable data and recomputed counts, without any runtime dependency:

```sh
python3 docs/parity/2026-10-21-runtime-audit/verify.py
```

To reproduce against an **exact d7caf package** and the pinned oracle installation,
run from the repository root:

```sh
node docs/parity/2026-10-21-runtime-audit/audit-portable.mjs \
  /path/to/exact-d7caf-package \
  node_modules/pele/dist/esm \
  fixtures/pele/diagram-entries.json > /tmp/pele-entry-audit.json
node docs/parity/2026-10-21-runtime-audit/oracle-probe.mjs \
  > /tmp/pele-oracle-runtime-gaps.json
```

The portable entry probe rejects different package/oracle/fixture bytes before
execution. The standalone oracle probe rejects a different full-bundle hash.
These are Node public-API reproductions, not three-browser or Swift coverage.
A single title mutation/throw per family does not exhaust phase-sensitive
callbacks, coercions, responsive redraws, nested helpers or constructors with
arguments. Packaging, fresh SDK/platform verification and release publication
remain separate gates. No performance measurements belong to this audit.
