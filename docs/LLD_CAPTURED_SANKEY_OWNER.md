# Captured Sankey document ownership

The opt-in `web-host-sankey` feature adds `compat::host_sankey::Document`.
It retains the exact captured registration, configuration, revision and eager
`Result<SankeyModel, PeleError>`. It retains no generic graph, node/property table,
encoded frame or JavaScript value. Each operation receives stable storage through
the shared `CapturedRead`, `CapturedWrite` and `AdmissionRead` interfaces.

Only the concrete built-in Sankey registration selects this owner. Matching model
or schema names and delegating providers are insufficient. Unsupported modules
consume the already captured parse plan through the existing arena path. Registry
replacement cannot change the policy retained by an existing document.

Ordinary source parses before creating the sink, emits through `InitialWriter`,
independently admits the completed storage and consumes authentic owned parser
state for its render projection. Prepared and caller-authored storage eagerly run
the existing importer. Parsing, import, helper policy and rendering each retain a
single implementation shared with the arena route. The default adapters do not
enable this feature or select this document yet.

`parse_host_storage_with_limits` prepares source once and chooses the captured ER
owner first, captured Sankey second, then the arena. It reuses each family's
existing plan consumer. In particular, ER retains its direct-parser abort and
fresh-factory fallback sequence; opting into Sankey cannot route ER through the
arena or repeat source preparation in JavaScript. Prepared/raw selection remains
explicitly typed. Three added public routing cases and the affected ER/method
consumers pass together: 22 integration checks across four targets.

## Failure and publication order

Apply rejects stale/exhausted revisions before admission, validates captured and
concrete schema policy, replaces the projection (including latent import errors)
and advances revision. Invoke validates the graph and method call, runs the shared
helper against private working storage, refreshes projection even after partial
failure, advances revision and validates post-state/return values. Render uses the
eager projection and existing rendering orchestration; revision and post-state
admission precede normal presentation publication.

The adapter must distinguish native state from the facade's last acknowledgment.
A dispatched operation can advance native revision before a later transport error.
It must not replay, silently roll back or repair that operation. Working epochs
remain private until admitted publication. Original callback exceptions, complete
restoration after callback edits and JavaScript identity ownership remain adapter
responsibilities.

Canonical frame counting shares the existing binary limits and exact atom widths.
It counts historical slots and duplicate entries. Observed ingress checks encoded
size before table/unit/work limits and structural admission; outbound admission
checks structure before encoded size. Invalid observed Array/Error markers retain
their counted width and then fail before projection. Explicit binary decoding
retains physical-buffer length checks. Metadata counts must come from actual native
serialization at the original adapter stage.

## Verification and remaining integration

The feature-enabled public consumer run passed 61 integration checks across 15
targets. Eight new checks exercise separate column storage, exact graph/error
results for all 11,616 frozen Sankey source cases, prepared/raw adoption, first
helpers and normal/raw renders, captured registration replacement, partial failure,
stale revisions, malformed sink rejection and canonical frame counting. Its
64 MiB/+1 case uses virtual metadata length; it does not prove browser combined
frame/config/factory exception ordering.

An additional default-feature run passed 29 integration checks across six targets
with zero warnings. Independent review checked operation ordering and exact
concrete-provider selection. Fresh WASM and Swift SDK validation remain separate gates: these native
checks do not prove JavaScript callback restoration, retained object ownership,
cross-boundary error identity or performance. The complete captured WASM backend
must pass those paths before public selection. No v0.3 release is authorized while
the Pele JavaScript performance gate remains unmet.
