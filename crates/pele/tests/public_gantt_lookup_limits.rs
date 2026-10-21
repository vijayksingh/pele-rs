//! Full public string-table admission must distinguish reuse from new text.
use pele::compat::{public::*, text::MetricsResources, ErrorCode, RenderOptions};
use sha2::{Digest, Sha256};

fn reference(value: &Atom) -> u32 {
    match value {
        Atom::Ref(id) => *id,
        other => panic!("Expected object reference, got {other:?}"),
    }
}

fn field<'a>(graph: &'a ModelGraph, object: u32, name: &str) -> &'a Atom {
    &graph.nodes[object as usize]
        .properties()
        .iter()
        .find(|property| graph.strings[property.key as usize].is(name))
        .unwrap_or_else(|| panic!("Missing property {name}"))
        .value
}

fn invoke(
    document: &mut PublicDocument,
    receiver: u32,
    name: &str,
    arguments: Vec<Atom>,
) -> InvocationOutcome {
    let method = document
        .model_schema()
        .methods
        .iter()
        .find(|method| method.name == name)
        .unwrap()
        .id;
    document
        .invoke(
            document.revision(),
            MethodCall {
                method,
                receiver: Atom::Ref(receiver),
                arguments,
            },
            &mut MetricsResources,
        )
        .expect("The valid public call must dispatch")
}

fn graph_digest(graph: &ModelGraph) -> [u8; 32] {
    Sha256::digest(encode_binary_exchange(0, graph, b"{}", DEFAULT_BINARY_BUDGET).unwrap()).into()
}

#[test]
fn full_string_table_reuses_existing_class_but_rejects_new_sanitized_link() {
    let source =
        "gantt\ndateFormat YYYY-MM-DD\nA:a,2024-01-01,1d\nB:b,after a,1d\nclick a call callback()";
    let engine = PublicEngine::with_builtins();
    let href = "https://example.test/limit";
    let raw_href = " \thttps://example.test/limit\n ";
    // Establish the computed public result without capacity pressure first.
    {
        let mut control = engine
            .parse(source, &RenderOptions::default().parse)
            .unwrap();
        let mut graph = control.snapshot().unwrap();
        let root = reference(&graph.root);
        let tasks = reference(field(&graph, root, "tasks"));
        let task = reference(field(&graph, tasks, "0"));
        let task_id = field(&graph, task, "id").clone();
        let raw_url = graph.strings.len() as u32;
        graph.strings.push(JsString::from(raw_href));
        control.apply(0, graph).unwrap();
        let outcome = invoke(
            &mut control,
            root,
            "setLink",
            vec![task_id, Atom::String(raw_url)],
        );
        assert!(matches!(outcome.result, Ok(Atom::Undefined)));
        let links = reference(field(&outcome.graph, root, "links"));
        let Node::Map { entries, .. } = &outcome.graph.nodes[links as usize] else {
            panic!("Links must remain a Map")
        };
        assert_eq!(entries.len(), 1);
        let Atom::String(url) = &entries[0].1 else {
            panic!("Link result must be a string")
        };
        assert!(outcome.graph.strings[*url as usize].is(href));
    }
    let mut document = engine
        .parse(source, &RenderOptions::default().parse)
        .unwrap();
    let root = reference(&document.snapshot_view().unwrap().root);
    // Scheduling establishes real Date aliases and the existing numeric array
    // keys before filling the table. Later click calls only need existing text.
    let scheduled = invoke(
        &mut document,
        root,
        "getTasks",
        vec![Atom::Number(1_704_067_200_000.)],
    );
    assert_eq!(scheduled.revision, 1);
    assert!(scheduled.result.is_ok());
    let mut graph = scheduled.graph;
    let tasks = reference(field(&graph, root, "tasks"));
    let task = reference(field(&graph, tasks, "0"));
    let next_task = reference(field(&graph, tasks, "1"));
    let task_id = field(&graph, task, "id").clone();
    let classes = reference(field(&graph, task, "classes"));
    let end_date = reference(field(&graph, task, "endTime"));
    assert_eq!(reference(field(&graph, next_task, "startTime")), end_date);
    let clickable = graph
        .strings
        .iter()
        .position(|string| string.is("clickable"))
        .unwrap() as u32;
    assert!(!graph.strings.iter().any(|string| string.is(href)));
    // The argument itself already exists. Only its sanitized result is new.
    let raw_url = graph.strings.len() as u32;
    graph.strings.push(JsString::from(raw_href));
    // Empty duplicates occupy the public table without allocating one million
    // text buffers; only one large graph snapshot is retained at a time.
    graph.strings.resize_with(1_000_000, || JsString(vec![]));
    let nodes = graph.nodes.len();
    document.apply(document.revision(), graph).unwrap();
    assert_eq!(document.revision(), 2);

    let before_failure = {
        let outcome = invoke(&mut document, root, "setClickEvent", vec![task_id.clone()]);
        assert_eq!(outcome.revision, 3);
        assert!(matches!(outcome.result, Ok(Atom::Undefined)));
        assert_eq!(outcome.graph.strings.len(), 1_000_000);
        assert_eq!(outcome.graph.nodes.len(), nodes);
        assert_eq!(reference(field(&outcome.graph, task, "classes")), classes);
        assert!(
            matches!(field(&outcome.graph, classes, "1"), Atom::String(id) if *id == clickable)
        );
        assert_eq!(reference(field(&outcome.graph, task, "endTime")), end_date);
        assert_eq!(
            reference(field(&outcome.graph, next_task, "startTime")),
            end_date
        );
        graph_digest(&outcome.graph)
    };

    {
        let outcome = invoke(
            &mut document,
            root,
            "setLink",
            vec![task_id.clone(), Atom::String(raw_url)],
        );
        assert_eq!(outcome.revision, 4);
        let Err(PublicError::Pele(error)) = outcome.result else {
            panic!("New sanitized text must fail admission at full capacity")
        };
        assert_eq!(error.code, ErrorCode::Limit);
        assert_eq!(
            error.message,
            "Gantt native graph operation exceeds work budget"
        );
        assert_eq!(graph_digest(&outcome.graph), before_failure);
        assert!(!outcome.graph.strings.iter().any(|string| string.is(href)));
    }

    // A rejected new string must not poison subsequent reuse or dispatch.
    let recovered = invoke(&mut document, root, "setClickEvent", vec![task_id]);
    assert_eq!(recovered.revision, 5);
    assert_eq!(document.revision(), 5);
    assert!(matches!(recovered.result, Ok(Atom::Undefined)));
    assert_eq!(recovered.graph.strings.len(), 1_000_000);
    assert_eq!(recovered.graph.nodes.len(), nodes);
    assert!(matches!(field(&recovered.graph, classes, "2"), Atom::String(id) if *id == clickable));
    assert_eq!(
        reference(field(&recovered.graph, task, "endTime")),
        end_date
    );
    assert_eq!(
        reference(field(&recovered.graph, next_task, "startTime")),
        end_date
    );
}
