//! Direct public ER construction versus an explicit hook-None typed provider.
//! Exact binary comparison includes all IDs, unused strings, descriptors and aliases.
use pele::compat::{
    diagrams::er::{Er, ErModel},
    public::*,
    text::MetricsResources,
    *,
};
use serde::Deserialize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct Legacy(Arc<AtomicUsize>);
impl DiagramModule for Legacy {
    type Model = ErModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Er.descriptor()
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<ErModel, PeleError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Er.parse(context)
    }
    fn render(
        &self,
        model: &ErModel,
        config: &Object,
        settings: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Er.render(model, config, settings, resources)
    }
}
impl PublicDiagramModule for Legacy {
    type PublicState = <Er as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Er.model_schema()
    }
    // Deliberately inherit try_parse_public == None: this provider executes
    // actual typed parse/create even when registered under the builtin ER ID.
    fn create_public(&self, model: &ErModel) -> Result<Self::PublicState, PeleError> {
        Er.create_public(model)
    }
    fn export_public(&self, state: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Er.export_public(state)
    }
    fn apply_public(
        &self,
        state: &mut Self::PublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        Er.apply_public(state, graph)
    }
    fn invoke_public(
        &self,
        state: &mut Self::PublicState,
        call: MethodCall,
        config: &Object,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        Er.invoke_public(state, call, config, host)
    }
    fn render_public(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Er.render_public(state, config, options, host)
    }
}
fn legacy() -> (PublicEngine, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut engine = PublicEngine::empty();
    engine.register(Legacy(calls.clone()));
    (engine, calls)
}
fn encoded(graph: &ModelGraph, revision: u64) -> Vec<u8> {
    encode_binary_exchange(revision, graph, &[], DEFAULT_BINARY_BUDGET).unwrap()
}
fn bytes(document: &PublicDocument) -> Vec<u8> {
    encoded(&document.snapshot_view().unwrap(), document.revision())
}
fn compare(a: Result<PublicDocument, PeleError>, b: Result<PublicDocument, PeleError>, id: &str) {
    match (a, b) {
        (Ok(a), Ok(b)) => assert!(bytes(&a) == bytes(&b), "complete arena differs: {id}"),
        (Err(a), Err(b)) => assert!(a == b, "complete parse/create error differs: {id}"),
        _ => panic!("parse/create acceptance differs: {id}"),
    }
}
#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    options: ParseOptions,
}
#[test]
fn er_direct_and_legacy_match_every_frozen_source_and_full_graph_table() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("../../../fixtures/pele/er.json")).unwrap();
    assert_eq!(corpus.cases.len(), 17_437);
    let direct = PublicEngine::with_builtins();
    let (old, _) = legacy();
    for case in corpus.cases {
        compare(
            direct.parse(&case.source, &case.options),
            old.parse(&case.source, &case.options),
            &case.id,
        );
    }
}
fn benchmark_source(n: usize) -> String {
    let mut source = "erDiagram\n".to_owned();
    for i in 0..n {
        if i != 0 {
            source.push('\n');
        }
        source.push_str(&format!("E{i} {{\n int id PK\n string label\n}}"));
    }
    source.push('\n');
    for i in 0..n - 1 {
        if i != 0 {
            source.push('\n');
        }
        source.push_str(&format!("E{i} ||--o{{ E{} : owns", i + 1));
    }
    source
}
#[test]
fn er_measured_workloads_use_direct_public_construction_without_fallback() {
    let direct = PublicEngine::with_builtins();
    let (old, _) = legacy();
    let config = Object::default();
    for n in [8, 48, 200] {
        // Same ordinary source shapes as scripts/perf/cases.mjs. The existing
        // public optional hook proves eligibility; public-engine binary equality
        // independently proves normal routing, preparation and final publication.
        let source = benchmark_source(n);
        let state = Er
            .try_parse_public(
                ParseContext {
                    source: &source,
                    config: &config,
                    title: None,
                    line_offset: 0,
                    max_edges: 10_000,
                },
                Some(EdgeLimit::Number(10_000.0)),
            )
            .expect("measured source must be direct")
            .unwrap();
        let mut options = ParseOptions::default();
        options.preparation.limit = 1_000_000;
        options.max_edges = 10_000;
        let a = direct.parse(&source, &options).unwrap();
        let b = old.parse(&source, &options).unwrap();
        assert!(encoded(&Er.export_public(&state).unwrap(), 0) == bytes(&b));
        assert!(bytes(&a) == bytes(&b));
    }
}
fn reference(atom: &Atom) -> u32 {
    if let Atom::Ref(id) = atom {
        *id
    } else {
        panic!("expected object")
    }
}
fn field(graph: &ModelGraph, id: u32, name: &str) -> Atom {
    graph.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| graph.strings[p.key as usize].is(name))
        .unwrap()
        .value
        .clone()
}
fn entity(graph: &ModelGraph, name: &str) -> u32 {
    let map = reference(&field(graph, reference(&graph.root), "entities"));
    let Node::Map { entries, .. } = &graph.nodes[map as usize] else {
        panic!("expected Map")
    };
    reference(
        &entries
            .iter()
            .find(|(key, _)| matches!(key,Atom::String(id) if graph.strings[*id as usize].is(name)))
            .unwrap()
            .1,
    )
}
fn text(graph: &mut ModelGraph, text: &str) -> Atom {
    let id = graph.strings.len() as u32;
    graph.strings.push(JsString::from(text));
    Atom::String(id)
}
fn property(graph: &mut ModelGraph, id: u32, key: &str, value: Atom, flags: u8) {
    if let Some(at) = graph.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| graph.strings[p.key as usize].is(key))
    {
        let property = &mut graph.nodes[id as usize].properties_mut()[at];
        property.value = value;
        property.flags = flags;
    } else {
        let key_id = graph.strings.len() as u32;
        graph.strings.push(JsString::from(key));
        graph.nodes[id as usize].properties_mut().push(Property {
            key: key_id,
            value,
            flags,
        });
    }
}
#[test]
fn er_direct_first_render_edits_helpers_aliases_and_captured_provider_match() {
    let source = "erDiagram\nA { int id PK }\nB\nA ||--o{ B : owns";
    let mut engine = PublicEngine::with_builtins();
    let (old, calls) = legacy();
    let mut a = engine.parse(source, &ParseOptions::default()).unwrap();
    let mut b = old.parse(source, &ParseOptions::default()).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let replacement = Arc::new(AtomicUsize::new(0));
    engine.register(Legacy(replacement.clone()));
    let _ = engine.parse(source, &ParseOptions::default()).unwrap();
    assert_eq!(replacement.load(Ordering::SeqCst), 1);
    drop(engine);
    let first_a = a
        .render_operation(
            a.revision(),
            &RenderOptions::default(),
            &mut MetricsResources,
        )
        .unwrap();
    let first_b = b
        .render_operation(
            b.revision(),
            &RenderOptions::default(),
            &mut MetricsResources,
        )
        .unwrap();
    assert!(first_a.result.as_ref().unwrap() == first_b.result.as_ref().unwrap());
    assert!(bytes(&a) == bytes(&b));
    // Keep the original entity and attributes array reachable as aliases while
    // editing the same record and adding detached descriptor/IEEE-bit values.
    for document in [&mut a, &mut b] {
        let mut graph = document.snapshot().unwrap();
        let root = reference(&graph.root);
        let id = entity(&graph, "A");
        let attrs = field(&graph, id, "attributes");
        let alias = text(&mut graph, "Changed");
        property(&mut graph, id, "alias", alias, 7);
        let detached = graph.nodes.len() as u32;
        graph.nodes.push(Node::Object { properties: vec![] });
        property(
            &mut graph,
            detached,
            "nan",
            Atom::Number(f64::from_bits(0x7ff8_0000_0000_0042)),
            0,
        );
        property(&mut graph, root, "savedEntity", Atom::Ref(id), 7);
        property(&mut graph, root, "savedAttributes", attrs, 7);
        property(&mut graph, root, "self", Atom::Ref(root), 7);
        document.apply(document.revision(), graph).unwrap();
    }
    let ar = a
        .render_operation(
            a.revision(),
            &RenderOptions::default(),
            &mut MetricsResources,
        )
        .unwrap();
    let br = b
        .render_operation(
            b.revision(),
            &RenderOptions::default(),
            &mut MetricsResources,
        )
        .unwrap();
    assert!(ar.result.as_ref().unwrap() == br.result.as_ref().unwrap());
    assert!(encoded(&ar.graph, ar.revision) == encoded(&br.graph, br.revision));
    for document in [&mut a, &mut b] {
        let mut graph = document.snapshot().unwrap();
        let root = graph.root.clone();
        let id = entity(&graph, "A");
        let name = text(&mut graph, "A");
        document.apply(document.revision(), graph).unwrap();
        let method = document
            .model_schema()
            .methods
            .iter()
            .find(|m| m.name == "addEntity")
            .unwrap()
            .id;
        let result = document
            .invoke(
                document.revision(),
                MethodCall {
                    method,
                    receiver: root,
                    arguments: vec![name],
                },
                &mut MetricsResources,
            )
            .unwrap();
        assert!(matches!(result.result,Ok(Atom::Ref(found)) if found==id));
    }
    assert!(bytes(&a) == bytes(&b));
}
#[test]
fn er_direct_fallback_preserves_limits_syntax_order_and_legacy_graph_bytes() {
    let direct = PublicEngine::with_builtins();
    let (old, _) = legacy();
    let mut options = ParseOptions::default();
    options.preparation.limit = 1_000_000;
    // The hook sees prepared bytes, including Engine's final newline. Exercise
    // both accepted graphs and original grammar errors at every cap boundary.
    let config = Object::default();
    for size in [65_535, 65_536, 65_537] {
        for malformed in [false, true] {
            let suffix = if malformed { "\nA { int }\n" } else { "\n" };
            let source = format!(
                "erDiagram\n{}{}",
                "N".repeat(size - 10 - suffix.len()),
                suffix
            );
            assert_eq!(source.len(), size);
            let attempt = Er.try_parse_public(
                ParseContext {
                    source: &source,
                    config: &config,
                    title: None,
                    line_offset: 0,
                    max_edges: 5_000,
                },
                None,
            );
            assert_eq!(attempt.is_some(), size <= 65_536);
            if let Some(result) = attempt {
                assert_eq!(result.is_err(), malformed);
            }
            compare(
                direct.parse(source.trim_end_matches('\n'), &options),
                old.parse(source.trim_end_matches('\n'), &options),
                "prepared source boundary",
            );

            let title = "T".repeat(size);
            let body = if malformed {
                "erDiagram\nA { int }\n"
            } else {
                "erDiagram\nA\n"
            };
            let attempt = Er.try_parse_public(
                ParseContext {
                    source: body,
                    config: &config,
                    title: Some(&title),
                    line_offset: 0,
                    max_edges: 5_000,
                },
                None,
            );
            assert_eq!(attempt.is_some(), size <= 65_536);
            if let Some(result) = attempt {
                assert_eq!(result.is_err(), malformed);
            }
            let frontmatter = format!("---\ntitle: {title}\n---\n{body}");
            let actual = direct.parse(&frontmatter, &options);
            let expected = old.parse(&frontmatter, &options);
            assert_eq!(actual.is_err(), malformed);
            compare(actual, expected, "prepared title boundary");
        }
    }
    let mut large = "erDiagram\n".to_owned();
    large.push_str(&"N".repeat(70_000));
    for source in [
        large.as_str(),
        "erDiagram\nA { int x }\nA { string y }",
        "erDiagram\nA\nB\nA ||--o{ B : first\nC",
        "erDiagram\nA\nclassDef x fill:red\nclass A x",
        "erDiagram\nsubgraph s\nA\nend\nsubgraph s\nB\nend",
        "erDiagram\nA\nstyle A fill:red\nB { int }",
        "erDiagram\nA\nA[\"alias\"]\nB { int }",
    ] {
        let config = Object::default();
        assert!(
            Er.try_parse_public(
                ParseContext {
                    source,
                    config: &config,
                    title: None,
                    line_offset: 0,
                    max_edges: 5_000,
                },
                None
            )
            .is_none(),
            "fixture must use the compatibility fallback"
        );
        compare(
            direct.parse(source, &options),
            old.parse(source, &options),
            "fallback",
        );
    }
    large.push_str("\nA { int }");
    let a = direct.parse(&large, &options).err().expect("late syntax");
    let b = old.parse(&large, &options).err().expect("late syntax");
    assert_eq!(a.code, ErrorCode::Syntax);
    assert!(a == b);
    let source = benchmark_source(8);
    for source_limit in [
        SourceLimit::Undefined,
        SourceLimit::Null,
        SourceLimit::Number(-0.0),
        SourceLimit::Number(10.5),
        SourceLimit::Number(f64::NAN),
    ] {
        for edge in [
            EdgeLimit::Undefined,
            EdgeLimit::Null,
            EdgeLimit::Number(-0.0),
            EdgeLimit::Number(f64::NAN),
        ] {
            let limits = ParseLimits {
                source: Some(source_limit),
                max_edges: Some(edge),
            };
            compare(
                direct.parse_with_limits(&source, &options, limits),
                old.parse_with_limits(&source, &options, limits),
                "numeric limits",
            );
        }
    }
    // Two distinct interned strings (name and generated ID) exceed 16M units.
    // Parse the same valid prefix first to prove real graph admission, then add
    // a short syntax error. Large documents are dropped between calls; no dump.
    options.preparation.limit = 10_000_000;
    let mut source = String::with_capacity(8_000_040);
    source.push_str("erDiagram\n");
    source.extend(std::iter::repeat_n('N', 8_000_001));
    assert!(source.len() < MAX_PUBLIC_SOURCE_BYTES);
    let expected = old
        .parse(&source, &options)
        .err()
        .expect("graph string budget");
    let actual = direct
        .parse(&source, &options)
        .err()
        .expect("graph string budget");
    assert_eq!(expected.code, ErrorCode::Limit);
    assert_eq!(expected.message, "Public model graph exceeds work budget");
    assert!(actual == expected);
    source.push_str("\nA { int }");
    let expected = old
        .parse(&source, &options)
        .err()
        .expect("syntax before graph budget");
    let actual = direct
        .parse(&source, &options)
        .err()
        .expect("syntax before graph budget");
    assert_eq!(expected.code, ErrorCode::Syntax);
    assert!(actual == expected);
}
