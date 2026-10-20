//! Public construction and lifecycle equivalence independent of initial ID numbering.
use pele::compat::{
    diagrams::gantt::{Gantt, GanttModel},
    public::*,
    text::MetricsResources,
    *,
};
use serde_json::{json, Value as Json};
use std::collections::HashMap;

fn normalized(graph: &ModelGraph) -> Json {
    fn atoms(
        value: &mut Json,
        graph: &ModelGraph,
        ids: &mut HashMap<u32, usize>,
        queue: &mut Vec<u32>,
    ) {
        if let Some(kind) = value.get("kind").and_then(Json::as_str).map(str::to_owned) {
            if kind == "ref" {
                let id = value["value"].as_u64().unwrap() as u32;
                let next = ids.len();
                let canonical = *ids.entry(id).or_insert_with(|| {
                    queue.push(id);
                    next
                });
                value["value"] = json!(canonical);
                return;
            }
            if kind == "string" {
                let id = value["value"].as_u64().unwrap() as usize;
                value["value"] = json!(graph.strings[id].0);
                return;
            }
        }
        match value {
            Json::Array(values) => {
                for value in values {
                    atoms(value, graph, ids, queue)
                }
            }
            Json::Object(values) => {
                for value in values.values_mut() {
                    atoms(value, graph, ids, queue)
                }
            }
            _ => {}
        }
    }
    let mut ids = HashMap::new();
    let mut queue = vec![];
    let mut root = serde_json::to_value(&graph.root).unwrap();
    atoms(&mut root, graph, &mut ids, &mut queue);
    let mut nodes = vec![];
    let mut i = 0;
    while i < queue.len() {
        let mut node = serde_json::to_value(&graph.nodes[queue[i] as usize]).unwrap();
        for property in node["properties"].as_array_mut().unwrap() {
            let key = property["key"].as_u64().unwrap() as usize;
            property["key"] = json!(graph.strings[key].0);
        }
        atoms(&mut node, graph, &mut ids, &mut queue);
        nodes.push(node);
        i += 1;
    }
    json!({"root":root,"nodes":nodes,"retainedNodeCount":graph.nodes.len()})
}
fn adapted(source: &str, options: &RenderOptions) -> PublicDocument {
    let typed = Engine::with_builtins()
        .parse(source, &options.parse)
        .unwrap();
    PublicDocument::from_module(
        Gantt,
        typed.model::<GanttModel>().unwrap(),
        typed.config().clone(),
    )
    .unwrap()
}
#[test]
fn gantt_source_public_parse_matches_typed_adaptation_through_scheduling_and_apply() {
    let fixtures: Json = serde_json::from_str(include_str!(
        "../../../fixtures/pele/gantt-construction.json"
    ))
    .unwrap();
    let engine = PublicEngine::with_builtins();
    for case in fixtures["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let mut options = render_options(&case["options"].to_string()).unwrap();
        options.render.now = Some(1704067200000.);
        let mut direct = engine.parse(source, &options.parse).unwrap();
        let mut legacy = adapted(source, &options);
        assert_eq!(direct.revision(), 0);
        assert_eq!(
            normalized(&direct.snapshot().unwrap()),
            normalized(&legacy.snapshot().unwrap()),
            "{} initial",
            case["id"]
        );
        for _ in 0..2 {
            let a = direct.snapshot().unwrap();
            let b = legacy.snapshot().unwrap();
            direct.apply(direct.revision(), a).unwrap();
            legacy.apply(legacy.revision(), b).unwrap();
            let a = direct
                .render_operation(direct.revision(), &options, &mut MetricsResources)
                .unwrap();
            let b = legacy
                .render_operation(legacy.revision(), &options, &mut MetricsResources)
                .unwrap();
            assert_eq!(
                serde_json::to_value(&a.result).unwrap(),
                serde_json::to_value(&b.result).unwrap(),
                "{} outcome",
                case["id"]
            );
            assert_eq!(
                normalized(&a.graph),
                normalized(&b.graph),
                "{} scheduled state",
                case["id"]
            );
            assert_eq!(a.revision, b.revision);
        }
    }
}
#[test]
fn gantt_source_errors_remain_the_typed_parser_errors() {
    for source in [
        "gantt\nA:active",
        "gantt\nA:a,b,c,d",
        "gantt\nsection S\nnot valid !",
    ] {
        let options = ParseOptions::default();
        let typed = Engine::with_builtins()
            .parse(source, &options)
            .err()
            .expect("invalid source");
        let public = PublicEngine::with_builtins()
            .parse(source, &options)
            .err()
            .expect("invalid source");
        assert_eq!(
            serde_json::to_value(typed).unwrap(),
            serde_json::to_value(public).unwrap()
        );
    }
}
