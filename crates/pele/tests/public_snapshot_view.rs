//! Public borrowed snapshots and explicit apply retain all-family operation behavior.
use pele::compat::{public::*, text::MetricsResources, *};
use serde_json::Value;
use std::borrow::Cow;

fn bytes(graph: &ModelGraph) -> Vec<u8> {
    encode_binary_exchange(0, graph, &[], DEFAULT_BINARY_BUDGET).unwrap()
}

#[test]
fn borrowed_builtin_snapshots_preserve_binary_state_and_repeated_operation_results() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/pele/diagram-entries.json")).unwrap();
    let registry = PublicEngine::with_builtins();
    let mut families = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if !case["id"].as_str().unwrap().ends_with("/baseline") {
            continue;
        }
        families += 1;
        let id = case["family"].as_str().unwrap();
        let diagram: DiagramId = serde_json::from_value(case["family"].clone()).unwrap();
        let source = case["source"].as_str().unwrap();
        let mut retained = registry
            .parse_as(&diagram, source, &ParseOptions::default())
            .unwrap();
        let mut reapplied = registry
            .parse_as(&diagram, source, &ParseOptions::default())
            .unwrap();
        assert!(retained.unchanged_public_graph_is_noop(), "{id}");
        assert!(
            matches!(retained.snapshot_view().unwrap(), Cow::Borrowed(_)),
            "{id}"
        );
        assert_eq!(
            bytes(&retained.snapshot_view().unwrap()),
            bytes(&retained.snapshot().unwrap()),
            "{id}: owned and borrowed binary export"
        );
        let mut options = RenderOptions::default();
        options.render.now = Some(1704067200000.);
        // Full apply is still a real public operation: it advances revision.
        // A transport that opts out of redundant apply must get identical state
        // and rendering, including a dispatched output error and later recovery.
        for output_limit in [None, Some(1), None] {
            options.render.output_limit = output_limit;
            let before_revision = reapplied.revision();
            let graph = reapplied.snapshot().unwrap();
            reapplied.apply(before_revision, graph).unwrap();
            assert_eq!(reapplied.revision(), before_revision + 1, "{id}");
            let a = retained
                .render_operation(retained.revision(), &options, &mut MetricsResources)
                .unwrap();
            let b = reapplied
                .render_operation(reapplied.revision(), &options, &mut MetricsResources)
                .unwrap();
            assert_eq!(
                a.result.is_err(),
                output_limit.is_some(),
                "{id}: operation outcome"
            );
            assert_eq!(
                serde_json::to_value(&a.result).unwrap(),
                serde_json::to_value(&b.result).unwrap(),
                "{id}: unchanged apply changed rendering"
            );
            assert_eq!(
                bytes(&a.graph),
                bytes(&b.graph),
                "{id}: post-operation state"
            );
            assert_eq!(
                bytes(&retained.snapshot_view().unwrap()),
                bytes(&a.graph),
                "{id}: borrowed post-operation state"
            );
        }
    }
    assert_eq!(families, 33);
}
