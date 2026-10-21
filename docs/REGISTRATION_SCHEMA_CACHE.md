# Captured registration schema validation

Each private Registration lazily retains the validation result and membership
indexes of its immutable captured base ModelSchema. The cache belongs to that
registration, not a name, version, diagram type or global schema address.
Replacement creates a fresh registration; retained documents keep the old one.

Registration remains infallible. Schema validation starts only at its existing
operation stage, and failures retain the same PeleError. Snapshot export still
runs before schema checks. Graph structure, reference/work limits, instance IDs,
function IDs, method existence, argument bounds and receiver checks still run
on every operation, in their previous order. Full ModelSchemaView validation
and the public arbitrary-schema validators retain their existing behavior.

The change adds no public API, wire representation or provider capability.
Public integration covers same-ID/version table replacement, retained methods
after registry drop, repeated snapshots/apply/render, invalid schema versus
invalid graph/grammar precedence, and changing stack schemas through public
validators. Cross-platform verification uses the real WASM and source-built
Swift SDK. Performance scope is repeated small-model metadata validation;
source rendering and host graph transport remain separate costs.
