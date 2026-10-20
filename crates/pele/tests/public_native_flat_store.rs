#![cfg(feature = "web-host-sankey")]
//! Public owner/admission/operation integration. ModelGraph exists only as oracle.
use pele::compat::{
    diagrams::sankey::Sankey, host_sankey::*, public::*, text::MetricsResources, *,
};

fn capture(graph: &ModelGraph) -> FlatEpoch {
    let empty = FlatEpoch::empty();
    let mut observe = NativeObservation::new(&empty, &empty.string_history());
    for (id, string) in graph.strings.iter().enumerate() {
        observe.begin_string(id as u32, string.0.len()).unwrap();
        for chunk in string.0.chunks(1024) {
            observe.string_chunk(chunk).unwrap();
        }
        observe.end_string().unwrap();
    }
    for (id, node) in graph.nodes.iter().enumerate() {
        observe.begin_node(id as u32).unwrap();
        for p in node.properties() {
            observe.property(p.key, p.flags, p.value.clone()).unwrap();
        }
        let kind = match node {
            Node::Object { .. } => AdmissionKind::Object,
            Node::Instance { prototype, .. } => AdmissionKind::Instance(*prototype),
            Node::Array { length, .. } => AdmissionKind::Array(*length),
            Node::Map { entries, .. } => {
                for (k, v) in entries {
                    observe.map_pair(k.clone(), v.clone()).unwrap();
                }
                AdmissionKind::Map(entries.len())
            }
            Node::Set { values, .. } => {
                for v in values {
                    observe.set_value(v.clone()).unwrap();
                }
                AdmissionKind::Set(values.len())
            }
            Node::Date { milliseconds, .. } => AdmissionKind::Date(milliseconds.to_bits()),
            Node::Error { error_type, .. } => AdmissionKind::Error(*error_type),
        };
        observe.end_node(kind).unwrap();
    }
    observe.finish(graph.root.clone()).unwrap()
}
fn graph<S: CapturedRead<Value = Atom> + AdmissionRead>(store: &mut S) -> ModelGraph {
    let extent = store.extent().unwrap();
    let root = CapturedRead::root(store);
    let mut strings = Vec::new();
    let mut nodes = Vec::new();
    for id in 0..extent.strings {
        let mut units = Vec::new();
        store
            .with_string_chunks(id as u32, |chunk| {
                units.extend_from_slice(chunk);
                Ok(())
            })
            .unwrap();
        strings.push(JsString(units));
    }
    for id in 0..extent.nodes {
        let header = AdmissionRead::header(store, id).unwrap();
        let mut properties = Vec::new();
        while properties.len() < header.properties {
            let start = properties.len();
            store
                .with_properties(id, start, |values| {
                    properties.extend_from_slice(values);
                    Ok(())
                })
                .unwrap();
        }
        nodes.push(match header.kind {
            AdmissionKind::Object => Node::Object { properties },
            AdmissionKind::Instance(prototype) => Node::Instance {
                prototype,
                properties,
            },
            AdmissionKind::Array(length) => Node::Array { length, properties },
            AdmissionKind::Date(bits) => Node::Date {
                milliseconds: f64::from_bits(bits),
                properties,
            },
            AdmissionKind::Error(error_type) => Node::Error {
                error_type,
                properties,
            },
            AdmissionKind::Map(n) => {
                let mut entries = Vec::new();
                while entries.len() < n {
                    let start = entries.len();
                    store
                        .with_map_entries(id, start, |values| {
                            entries.extend_from_slice(values);
                            Ok(())
                        })
                        .unwrap();
                }
                Node::Map {
                    entries,
                    properties,
                }
            }
            AdmissionKind::Set(n) => {
                let mut values = Vec::new();
                while values.len() < n {
                    let start = values.len();
                    store
                        .with_set_values(id, start, |items| {
                            values.extend_from_slice(items);
                            Ok(())
                        })
                        .unwrap();
                }
                Node::Set { values, properties }
            }
            _ => panic!("invalid observed marker cannot serialize"),
        });
    }
    ModelGraph {
        version: extent.version,
        root,
        strings,
        nodes,
    }
}
fn bytes(graph: &ModelGraph) -> Vec<u8> {
    encode_binary_exchange(0, graph, b"", DEFAULT_BINARY_BUDGET).unwrap()
}
fn same<S: CapturedRead<Value = Atom> + AdmissionRead>(store: &mut S, arena: &PublicDocument) {
    let expected = arena.snapshot().unwrap();
    assert!(bytes(&graph(store)) == bytes(&expected));
}
fn result<T: serde::Serialize>(value: &T) -> serde_json::Value {
    serde_json::to_value(value).unwrap()
}
fn source(engine: &PublicEngine, text: &str) -> (Document, FlatEpoch) {
    match engine
        .parse_host_sankey_with_limits(
            text,
            &ParseOptions::default(),
            ParseLimits::default(),
            |_| Ok(NativeSourceWriter::new()),
        )
        .unwrap()
    {
        Parsed::Captured { document, storage } => (document, storage),
        _ => panic!("captured builtin"),
    }
}
fn method(doc: &Document, root: Atom, name: &str, arguments: Vec<Atom>) -> MethodCall {
    MethodCall {
        method: doc
            .schema()
            .model
            .methods
            .iter()
            .find(|m| m.name == name)
            .unwrap()
            .id,
        receiver: root,
        arguments,
    }
}
fn id(value: Atom) -> u32 {
    match value {
        Atom::Ref(id) => id,
        _ => panic!("reference"),
    }
}
fn field(store: &mut impl CapturedRead<Value = Atom>, node: u32, name: &str) -> Atom {
    let mut out = [None];
    store.own_fields(node, &[name], &mut out).unwrap();
    out[0].take().unwrap()
}

#[test]
fn native_flat_frozen_source_corpus_preserves_complete_graphs_and_errors() {
    use std::io::Read;
    let file = std::fs::File::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/pele/sankey.json.gz"
    ))
    .unwrap();
    let mut json = Vec::new();
    flate2::read::GzDecoder::new(file)
        .read_to_end(&mut json)
        .unwrap();
    let fixture: serde_json::Value = serde_json::from_slice(&json).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 11_616);
    let mut engine = PublicEngine::empty();
    engine.register(Sankey);
    for case in cases {
        let options: ParseOptions = serde_json::from_value(case["options"].clone()).unwrap();
        let text = case["source"].as_str().unwrap();
        let arena = engine.parse(text, &options);
        let native =
            engine.parse_host_sankey_with_limits(text, &options, ParseLimits::default(), |_| {
                Ok(NativeSourceWriter::new())
            });
        match (arena, native) {
            (Err(a), Err(b)) => assert_eq!(a, b, "{}", case["id"]),
            (
                Ok(arena),
                Ok(Parsed::Captured {
                    document,
                    mut storage,
                }),
            ) => {
                document.snapshot_admission(&mut storage).unwrap();
                assert!(
                    bytes(&graph(&mut storage)) == bytes(&arena.snapshot().unwrap()),
                    "{}",
                    case["id"]
                );
            }
            _ => panic!("source outcome differs: {}", case["id"]),
        }
    }
}

#[test]
fn native_flat_source_prepared_first_helper_render_and_epoch_isolation() {
    let engine = PublicEngine::with_builtins();
    let text = "sankey\nA,B,5\nB,C,2";
    let (mut doc, mut accepted) = source(&engine, text);
    let mut arena = engine.parse(text, &ParseOptions::default()).unwrap();
    same(&mut accepted, &arena);
    let original = accepted.clone();
    let original_bytes = bytes(&graph(&mut accepted));
    let mut work = accepted.fork();
    let name = work.intern("D").unwrap();
    doc.apply(0, &mut work).unwrap();
    arena.apply(0, graph(&mut work)).unwrap();
    let call = method(
        &doc,
        CapturedRead::root(&work),
        "findOrCreateNode",
        vec![Atom::String(name)],
    );
    let a = doc.invoke(1, &mut work, call.clone()).unwrap();
    let b = arena.invoke(1, call, &mut MetricsResources).unwrap();
    assert_eq!(result(&a.result), result(&b.result));
    same(&mut work, &arena);
    accepted = work.freeze().unwrap();
    assert!(!accepted.exact_eq(&original));
    assert!(bytes(&graph(&mut original.clone())) == original_bytes);
    let a = doc
        .render(
            2,
            &mut accepted,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = arena
        .render_operation(2, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(result(&a.result), result(&b.result));
    same(&mut accepted, &arena);
    let config = Object::default();
    let context = || ParseContext {
        source: text,
        config: &config,
        title: Some("prepared\u{feff}title"),
        line_offset: 0,
        max_edges: 500,
    };
    let (mut prepared, mut store) = match engine
        .parse_prepared_host_sankey_as(
            &DiagramType::Sankey.into(),
            context(),
            EdgeLimit::Undefined,
            |_| Ok(NativeSourceWriter::new()),
        )
        .unwrap()
    {
        Parsed::Captured { document, storage } => (document, storage),
        _ => panic!("prepared"),
    };
    let mut old = engine
        .parse_prepared_as(&DiagramType::Sankey.into(), context(), EdgeLimit::Undefined)
        .unwrap();
    same(&mut store, &old);
    let a = prepared
        .render(
            0,
            &mut store,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = old
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(result(&a.result), result(&b.result));
}

#[test]
fn native_flat_raw_history_bits_duplicate_descriptors_partial_helper_and_repair() {
    let engine = PublicEngine::with_builtins();
    let (_, mut fresh) = source(&engine, "sankey\nA,B,5");
    let mut g = graph(&mut fresh);
    let special = g.strings.len() as u32;
    g.strings.push(JsString(vec![0xd800, 0, 0xdc00]));
    g.strings.push(JsString(vec![0xd800, 0, 0xdc00]));
    let x = g.nodes.len() as u32;
    let nan = f64::from_bits(0x7ff8000000001234);
    g.nodes.push(Node::Date {
        milliseconds: nan,
        properties: vec![],
    });
    g.nodes.push(Node::Map {
        entries: vec![
            (Atom::Number(-0.0), Atom::Undefined),
            (Atom::Number(0.0), Atom::Ref(x)),
            (Atom::Number(nan), Atom::String(special)),
            (Atom::Number(f64::NAN), Atom::String(special + 1)),
        ],
        properties: vec![],
    });
    g.nodes.push(Node::Set {
        values: vec![
            Atom::Ref(x),
            Atom::Number(-0.0),
            Atom::Number(nan),
            Atom::HostReference(7),
        ],
        properties: vec![],
    });
    for error_type in [
        ErrorObjectKind::Error,
        ErrorObjectKind::Pele,
        ErrorObjectKind::Type,
        ErrorObjectKind::Range,
        ErrorObjectKind::Syntax,
        ErrorObjectKind::Reference,
        ErrorObjectKind::Eval,
        ErrorObjectKind::Uri,
    ] {
        g.nodes.push(Node::Error {
            error_type,
            properties: vec![Property {
                key: special,
                value: Atom::Ref(x),
                flags: 1,
            }],
        });
    }
    let first_a = g.strings.iter().position(|s| s.is("A")).unwrap() as u32;
    g.strings.push(JsString::from("A"));
    let mut accepted = capture(&g);
    assert!(bytes(&graph(&mut accepted)) == bytes(&g));
    assert_eq!(
        canonical_frame_bytes(&mut accepted, 17, DEFAULT_BINARY_BUDGET).unwrap(),
        encode_binary_exchange(0, &g, &[0; 17], DEFAULT_BINARY_BUDGET)
            .unwrap()
            .len()
    );
    let selection = engine
        .select_host_sankey_entry(&DiagramType::Sankey.into())
        .unwrap()
        .unwrap();
    let mut doc = selection.bind(Object::default(), &mut accepted).unwrap();
    let mut arena = PublicDocument::from_graph(Sankey, g, Object::default()).unwrap();
    let pinned = accepted.clone();
    let mut work = accepted.fork();
    let before_strings = work.counts().unwrap().strings;
    assert_eq!(work.intern("A").unwrap(), first_a);
    assert_eq!(work.counts().unwrap().strings, before_strings);
    let root = id(CapturedRead::root(&work));
    let nodes = id(field(&mut work, root, "nodes"));
    let n = CapturedRead::header(&mut work, nodes)
        .unwrap()
        .unwrap()
        .array_length
        .unwrap();
    let length = work.intern("length").unwrap();
    work.append_property(
        nodes,
        Property {
            key: length,
            flags: 7,
            value: Atom::Number(n as f64),
        },
    )
    .unwrap();
    work.append_property(
        nodes,
        Property {
            key: length,
            flags: 0,
            value: Atom::Number(n as f64),
        },
    )
    .unwrap();
    let name = work.intern("missing").unwrap();
    doc.apply(0, &mut work).unwrap();
    arena.apply(0, graph(&mut work)).unwrap();
    let call = method(
        &doc,
        CapturedRead::root(&work),
        "findOrCreateNode",
        vec![Atom::String(name)],
    );
    let a = doc.invoke(1, &mut work, call.clone()).unwrap();
    let b = arena.invoke(1, call, &mut MetricsResources).unwrap();
    assert!(a.result.is_err());
    assert_eq!(result(&a.result), result(&b.result));
    assert_eq!(doc.revision(), 2);
    same(&mut work, &arena);
    // Latent helper failure retains its cache insertion, while prior epoch remains exact.
    assert!(pinned.exact_eq(&accepted));
    accepted = work.freeze().unwrap();
    let mut repair = graph(&mut accepted);
    repair.nodes[nodes as usize]
        .properties_mut()
        .retain(|p| p.key != length);
    let mut repaired = capture(&repair);
    doc.apply(2, &mut repaired).unwrap();
    arena.apply(2, repair).unwrap();
    let a = doc
        .render(
            3,
            &mut repaired,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = arena
        .render_operation(3, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(result(&a.result), result(&b.result));
    same(&mut repaired, &arena);
}

#[test]
fn native_flat_observation_abort_history_page_boundaries_and_ordered_admission() {
    let engine = PublicEngine::with_builtins();
    let (mut doc, accepted) = source(&engine, "sankey\nA,B,5");
    let mut baseline = accepted.clone();
    let baseline_bytes = bytes(&graph(&mut baseline));
    let mut observation = NativeObservation::new(&accepted, &accepted.string_history());
    let string_id = accepted.logical_counts().1 as u32;
    let mut units = vec![0x61; 2050];
    units[1023] = 0xd83d;
    units[1024] = 0xde00;
    units[2049] = 0xd800;
    observation.begin_string(string_id, units.len()).unwrap();
    for chunk in units.chunks(1024) {
        observation.string_chunk(chunk).unwrap();
    }
    observation.end_string().unwrap();
    observation
        .begin_node(id(CapturedRead::root(&accepted)))
        .unwrap();
    observation
        .property(string_id, 7, Atom::String(string_id))
        .unwrap();
    let history = observation.abort();
    assert_eq!(history.len(), string_id as usize + 1);
    assert!(bytes(&graph(&mut accepted.clone())) == baseline_bytes);
    let resumed = NativeObservation::new(&accepted, &history)
        .finish(CapturedRead::root(&accepted))
        .unwrap();
    let mut work = resumed.fork();
    let mut out = Vec::new();
    work.with_string_chunks(string_id, |chunk| {
        out.extend_from_slice(chunk);
        Ok(())
    })
    .unwrap();
    assert_eq!(out, units);
    let node = work
        .append_node(Node::Object { properties: vec![] })
        .unwrap();
    let key = work.intern("x").unwrap();
    for i in 0..2200 {
        work.append_property(
            node,
            Property {
                key,
                value: Atom::Number(i as f64),
                flags: 7,
            },
        )
        .unwrap();
    }
    let epoch = work.freeze().unwrap();
    let mut fork = epoch.fork();
    fork.replace_property_value(node, 1024, Atom::Number(f64::from_bits(0x7ff8000000004321)))
        .unwrap();
    let changed = fork.freeze().unwrap();
    assert!(!changed.exact_eq(&epoch));
    assert!(
        matches!(graph(&mut epoch.clone()).nodes[node as usize].properties()[1024].value,Atom::Number(n) if n==1024.0)
    );
    // Same out-of-range reference reaches the same public structural error before projection.
    let mut invalid = changed.fork();
    invalid.set_root(Atom::Ref(u32::MAX));
    let expected = graph(&mut invalid).validate().unwrap_err();
    let actual = doc.apply(0, &mut invalid).unwrap_err();
    assert_eq!(result(&actual), result(&expected));
    assert_eq!(doc.revision(), 0);
    let mut marker = NativeObservation::new(&accepted, &accepted.string_history());
    marker
        .begin_node(accepted.logical_counts().0 as u32)
        .unwrap();
    marker.end_node(AdmissionKind::InvalidArray).unwrap();
    let mut marked = marker.finish(CapturedRead::root(&accepted)).unwrap();
    let count = canonical_frame_bytes(&mut marked, 0, DEFAULT_BINARY_BUDGET).unwrap();
    assert!(admit_observed(&mut marked, 0, count - 1)
        .unwrap_err()
        .message
        .contains("budget"));
    assert!(admit_observed(&mut marked, 0, count).is_err());
}

#[test]
fn native_flat_malformed_batch_cannot_replace_accepted_owner_or_allocate_high_id() {
    let engine = PublicEngine::with_builtins();
    let (mut doc, mut accepted) = source(&engine, "sankey\nA,B,5");
    let before = accepted.clone();
    let mut observation = NativeObservation::new(&accepted, &accepted.string_history());
    // ABI1 one BeginNode with an inadmissible highwater ID. No giant header allocation.
    let mut frame = vec![1, 0, 1, 0, 17, 0, 0, 0, 4, 4, 0, 0, 0];
    frame.extend_from_slice(&u32::MAX.to_le_bytes());
    assert!(observation.append_batch(&frame).is_err());
    let history = observation.abort();
    assert_eq!(history.len(), accepted.logical_counts().1);
    assert!(accepted.exact_eq(&before));
    assert_eq!(doc.revision(), 0);
    let rendered = doc
        .render(
            0,
            &mut accepted,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    assert!(rendered.result.is_ok());
    assert_eq!(rendered.revision, 1);
}

#[test]
fn native_flat_observation_frames_preserve_sparse_arguments_and_renderable_source() {
    let engine = PublicEngine::with_builtins();
    let (mut doc, accepted) = source(&engine, "sankey\nA,B,5");
    let root = accepted.root_words();
    let id = accepted.logical_counts().1 as u32;
    let mut records = Vec::new();
    let mut event = |op: u8, payload: &[u8]| {
        records.push(op);
        records.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        records.extend_from_slice(payload);
    };
    let mut begin = Vec::new();
    begin.extend_from_slice(&id.to_le_bytes());
    begin.extend_from_slice(&3u32.to_le_bytes());
    event(1, &begin);
    event(2, &[0x00, 0xd8, 0x41, 0, 0x00, 0xdc]);
    event(3, &[]);
    let mut root_bytes = vec![root[0] as u8];
    root_bytes.extend_from_slice(&root[1].to_le_bytes());
    root_bytes.extend_from_slice(&root[2].to_le_bytes());
    event(9, &root_bytes);
    event(10, &3u32.to_le_bytes());
    let mut arg = Vec::new();
    arg.extend_from_slice(&2u32.to_le_bytes());
    arg.push(5);
    arg.extend_from_slice(&id.to_le_bytes());
    arg.extend_from_slice(&0u32.to_le_bytes());
    event(11, &arg);
    let mut frame = vec![1, 0, 6, 0];
    frame.extend_from_slice(&((8 + records.len()) as u32).to_le_bytes());
    frame.extend_from_slice(&records);
    let mut observation = NativeObservation::new(&accepted, &accepted.string_history());
    observation.append_batch(&frame).unwrap();
    let observed = observation.finish_stream().unwrap();
    assert_eq!(observed.argument_length, 3);
    assert_eq!(observed.arguments, vec![(2, [5, id, 0])]);
    let mut store = observed.epoch;
    let mut units = Vec::new();
    store
        .with_string_chunks(id, |chunk| {
            units.extend_from_slice(chunk);
            Ok(())
        })
        .unwrap();
    assert_eq!(units, vec![0xd800, 0x41, 0xdc00]);
    doc.apply(0, &mut store).unwrap();
    let rendered = doc
        .render(
            1,
            &mut store,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    assert!(rendered.result.is_ok());
}
