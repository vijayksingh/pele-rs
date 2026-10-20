//! Public admission preserves fault precedence and detached historical values.
use pele::compat::{diagrams::sankey::Sankey, public::*, text::MetricsResources, *};

fn semantic(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn bare() -> ModelGraph {
    ModelGraph {
        version: 1,
        root: Atom::Undefined,
        strings: vec!["key".into()],
        nodes: vec![Node::Object { properties: vec![] }],
    }
}
fn property(key: u32, flags: u8, value: Atom) -> Property {
    Property { key, flags, value }
}
fn document() -> PublicDocument {
    PublicEngine::with_builtins()
        .parse("sankey\nA,B,5\nA,C,2", &ParseOptions::default())
        .unwrap()
}
fn binary(graph: &ModelGraph) -> Vec<u8> {
    encode_binary_exchange(0, graph, &[], DEFAULT_BINARY_BUDGET).unwrap()
}

#[test]
fn public_graph_admission_orders_structural_faults_before_schema_identity_faults() {
    let mut graph = bare();
    graph.version = 2;
    graph.root = Atom::Ref(99);
    assert_eq!(
        graph.validate().unwrap_err(),
        semantic("Unsupported public graph version")
    );
    graph.version = 1;
    graph.nodes[0]
        .properties_mut()
        .push(property(99, 255, Atom::String(99)));
    assert_eq!(
        graph.validate().unwrap_err(),
        semantic("Invalid public object reference")
    );
    graph.root = Atom::BuiltinFunction(u32::MAX);
    assert_eq!(
        graph.validate_schema(&OPAQUE_MODEL_SCHEMA).unwrap_err(),
        semantic("Invalid public property")
    );
    graph.nodes[0] = Node::Map {
        entries: vec![(Atom::Ref(99), Atom::String(99))],
        properties: vec![property(0, 7, Atom::String(99))],
    };
    assert_eq!(
        graph.validate().unwrap_err(),
        semantic("Invalid public string reference")
    );
    graph.nodes[0].properties_mut().clear();
    assert_eq!(
        graph.validate().unwrap_err(),
        semantic("Invalid public object reference")
    );
    if let Node::Map { entries, .. } = &mut graph.nodes[0] {
        entries[0].0 = Atom::Null;
    }
    assert_eq!(
        graph.validate().unwrap_err(),
        semantic("Invalid public string reference")
    );
    graph.nodes[0] = Node::Object { properties: vec![] };
    graph.nodes.push(Node::Set {
        values: vec![Atom::Ref(99)],
        properties: vec![],
    });
    assert_eq!(
        graph.validate_schema(&OPAQUE_MODEL_SCHEMA).unwrap_err(),
        semantic("Invalid public object reference")
    );
    graph.nodes.pop();
    assert_eq!(
        graph.validate_schema(&OPAQUE_MODEL_SCHEMA).unwrap_err(),
        semantic("Unknown public builtin function")
    );
    graph.root = Atom::Undefined;
    graph.nodes[0] = Node::Instance {
        prototype: PrototypeId(u32::MAX),
        properties: vec![property(0, 7, Atom::BuiltinFunction(u32::MAX))],
    };
    assert_eq!(
        graph.validate_schema(&OPAQUE_MODEL_SCHEMA).unwrap_err(),
        semantic("Unknown public instance prototype")
    );
    let mut invalid_schema = OPAQUE_MODEL_SCHEMA;
    invalid_schema.root = Some(RecordId(u32::MAX));
    graph.version = 2;
    assert_eq!(
        graph.validate_schema(&invalid_schema).unwrap_err(),
        semantic("Unknown public root record")
    );
}

#[test]
fn rejected_adoption_apply_and_call_keep_revision_and_recover_with_same_graph() {
    let mut doc = document();
    let baseline = doc.snapshot().unwrap();
    let mut invalid = baseline.clone();
    invalid.nodes.push(Node::Set {
        values: vec![Atom::BuiltinFunction(u32::MAX), Atom::Ref(u32::MAX)],
        properties: vec![],
    });
    assert_eq!(
        PublicDocument::from_graph(Sankey, invalid.clone(), Object::default())
            .err()
            .unwrap(),
        semantic("Invalid public object reference")
    );
    assert_eq!(
        doc.apply(1, invalid.clone()).unwrap_err(),
        semantic("Stale public model revision")
    );
    assert_eq!(
        doc.apply(0, invalid).unwrap_err(),
        semantic("Invalid public object reference")
    );
    assert_eq!(doc.revision(), 0);
    assert_eq!(binary(&doc.snapshot().unwrap()), binary(&baseline));
    let method = doc
        .model_schema()
        .methods
        .iter()
        .find(|m| m.name == "findOrCreateNode")
        .unwrap()
        .id;
    let mut call = MethodCall {
        method,
        receiver: Atom::BuiltinFunction(u32::MAX),
        arguments: vec![Atom::Ref(u32::MAX)],
    };
    assert_eq!(
        doc.invoke(0, call.clone(), &mut MetricsResources)
            .err()
            .unwrap(),
        semantic("Invalid public object reference")
    );
    call.arguments.clear();
    assert_eq!(
        doc.invoke(0, call.clone(), &mut MetricsResources)
            .err()
            .unwrap(),
        semantic("Unknown public builtin function")
    );
    call.method = MethodId(u32::MAX);
    call.arguments.push(Atom::Ref(u32::MAX));
    assert_eq!(
        doc.invoke(0, call, &mut MetricsResources).err().unwrap(),
        semantic("Public model method is unavailable")
    );
    assert_eq!(doc.revision(), 0);
    doc.apply(0, baseline.clone()).unwrap();
    assert_eq!(doc.revision(), 1);
    assert_eq!(binary(&doc.snapshot().unwrap()), binary(&baseline));
}

#[test]
fn historical_graph_values_remain_admitted_without_normalization() {
    let mut doc = document();
    let mut graph = doc.snapshot().unwrap();
    let key = graph.strings.len() as u32;
    graph.strings.extend([
        JsString(vec![0xd800, 65, 0xdc00]),
        JsString(vec![0xd800, 65, 0xdc00]),
        "length".into(),
    ]);
    let first = graph.nodes.len() as u32;
    let nan = f64::from_bits(0x7ff8_0000_0000_0042);
    graph.nodes.push(Node::Array {
        length: u32::MAX,
        properties: vec![
            property(key + 2, 0, Atom::Number(-0.0)),
            property(key, 0, Atom::Number(-0.0)),
            property(key, 7, Atom::Ref(first)),
        ],
    });
    graph.nodes.push(Node::Date {
        milliseconds: nan,
        properties: vec![],
    });
    graph.nodes.push(Node::Error {
        error_type: ErrorObjectKind::Type,
        properties: vec![],
    });
    graph.nodes.push(Node::Map {
        entries: vec![
            (Atom::Number(-0.0), Atom::String(key)),
            (Atom::Number(0.0), Atom::String(key + 1)),
            (Atom::Number(nan), Atom::Ref(first + 1)),
            (Atom::Number(nan), Atom::HostReference(u32::MAX)),
        ],
        properties: vec![],
    });
    graph.nodes.push(Node::Set {
        values: vec![
            Atom::Undefined,
            Atom::Null,
            Atom::Bool(true),
            Atom::Bool(false),
            Atom::Number(f64::INFINITY),
            Atom::Number(f64::NEG_INFINITY),
            Atom::Ref(first + 2),
        ],
        properties: vec![],
    });
    graph.validate_schema(doc.model_schema()).unwrap();
    let expected = binary(&graph);
    let adopted = PublicDocument::from_graph(Sankey, graph.clone(), Object::default()).unwrap();
    assert_eq!(binary(&adopted.snapshot().unwrap()), expected);
    doc.apply(0, graph).unwrap();
    assert_eq!(binary(&doc.snapshot().unwrap()), expected);
    let outcome = doc
        .render_operation(1, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert!(outcome.result.is_ok());
    assert_eq!(outcome.revision, 2);
    assert_eq!(binary(&outcome.graph), expected);
}
