//! Public parser ownership transfer: exact graph publication, legacy-provider
//! dispatch and provenance isolation. No private formatter/codec tests.
use pele::compat::{
    diagrams::sankey::{Sankey, SankeyModel, SankeyNode},
    public::*,
    text::MetricsResources,
    *,
};
use std::sync::{Arc, Mutex};

const SOURCE: &str = "sankey-beta\nA,B,2\nA,C,3";
type State = <Sankey as PublicDiagramModule>::PublicState;
#[derive(Default)]
struct Observations {
    calls: Vec<String>,
    configs: Vec<Object>,
}
struct Legacy {
    observed: Arc<Mutex<Observations>>,
    supplied: Option<SankeyModel>,
    reject_create: bool,
}
impl Legacy {
    fn new() -> Self {
        Self {
            observed: Arc::default(),
            supplied: None,
            reject_create: false,
        }
    }
    fn mark(&self, value: impl Into<String>) {
        let mut observed = self.observed.lock().unwrap();
        if observed.calls.len() < 32 {
            observed.calls.push(value.into());
        }
    }
    fn create(&self, model: &SankeyModel) -> Result<State, PeleError> {
        if self.reject_create {
            Err(PeleError::new(
                "extension construction failed",
                ErrorCode::Semantic,
            ))
        } else {
            Sankey.create_public(model)
        }
    }
}
fn policy(limit: EdgeLimit) -> String {
    match limit {
        EdgeLimit::Undefined => "undefined".into(),
        EdgeLimit::Null => "null".into(),
        EdgeLimit::Number(n) => format!("{:016x}", n.to_bits()),
    }
}
impl DiagramModule for Legacy {
    type Model = SankeyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Sankey.descriptor()
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<SankeyModel, PeleError> {
        self.mark("parse");
        {
            let mut observed = self.observed.lock().unwrap();
            if observed.configs.len() < 16 {
                observed.configs.push(context.config.clone());
            }
        }
        if let Some(model) = &self.supplied {
            Ok(model.clone())
        } else {
            Sankey.parse(context)
        }
    }
    fn parse_with_edge_limit(
        &self,
        context: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<SankeyModel, PeleError> {
        self.mark(format!("parse-edge:{}", policy(limit)));
        self.parse(context)
    }
    fn parse_prepared(
        &self,
        context: ParseContext<'_>,
        limit: EdgeLimit,
    ) -> Result<SankeyModel, PeleError> {
        self.mark(format!("parse-prepared:{}", policy(limit)));
        self.parse(context)
    }
    fn render(
        &self,
        model: &SankeyModel,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render(model, config, options, host)
    }
}
impl PublicDiagramModule for Legacy {
    type PublicState = State;
    fn model_schema(&self) -> &'static ModelSchema {
        Sankey.model_schema()
    }
    // Deliberately inherit the new hook's default dispatch.
    fn create_public(&self, model: &SankeyModel) -> Result<State, PeleError> {
        self.mark("create");
        self.create(model)
    }
    fn create_public_with_edge_limit(
        &self,
        model: &SankeyModel,
        limit: EdgeLimit,
    ) -> Result<State, PeleError> {
        self.mark(format!("create-edge:{}", policy(limit)));
        self.create(model)
    }
    fn create_public_prepared(
        &self,
        model: &SankeyModel,
        limit: EdgeLimit,
    ) -> Result<State, PeleError> {
        self.mark(format!("create-prepared:{}", policy(limit)));
        self.create(model)
    }
    fn export_public(&self, state: &State) -> Result<ModelGraph, PeleError> {
        Sankey.export_public(state)
    }
    fn apply_public(&self, state: &mut State, graph: ModelGraph) -> Result<(), PeleError> {
        Sankey.apply_public(state, graph)
    }
    fn render_public(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render_public(state, config, options, host)
    }
    fn invoke_public(
        &self,
        state: &mut State,
        call: MethodCall,
        config: &Object,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        Sankey.invoke_public(state, call, config, host)
    }
}
// Same descriptor/schema/Model type, but a different concrete parser. Forwarding
// the opaque carrier must not certify this provider's arbitrary typed output.
struct Delegated(Legacy);
impl DiagramModule for Delegated {
    type Model = SankeyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Sankey.descriptor()
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<SankeyModel, PeleError> {
        self.0.parse(context)
    }
    fn render(
        &self,
        model: &SankeyModel,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render(model, config, options, host)
    }
}
impl PublicDiagramModule for Delegated {
    type PublicState = State;
    fn model_schema(&self) -> &'static ModelSchema {
        Sankey.model_schema()
    }
    fn create_public(&self, model: &SankeyModel) -> Result<State, PeleError> {
        Sankey.create_public(model)
    }
    fn create_public_parsed(
        &self,
        parsed: ParsedPublicModel<SankeyModel>,
    ) -> Result<State, PeleError> {
        assert!(parsed.is_from_module::<Self>());
        assert!(!parsed.is_from_module::<Sankey>());
        assert_eq!(
            parsed.model().nodes.len(),
            self.0.supplied.as_ref().unwrap().nodes.len()
        );
        Sankey.create_public_parsed(parsed)
    }
    fn export_public(&self, state: &State) -> Result<ModelGraph, PeleError> {
        Sankey.export_public(state)
    }
    fn apply_public(&self, state: &mut State, graph: ModelGraph) -> Result<(), PeleError> {
        Sankey.apply_public(state, graph)
    }
    fn render_public(
        &self,
        state: &mut State,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render_public(state, config, options, host)
    }
}
fn bytes(document: &PublicDocument) -> Vec<u8> {
    let view = document.snapshot_read().unwrap();
    let actual = view
        .encode_binary(document.revision(), &[], DEFAULT_BINARY_BUDGET)
        .unwrap();
    let expected = encode_binary_exchange(
        document.revision(),
        &document.snapshot_view().unwrap(),
        &[],
        DEFAULT_BINARY_BUDGET,
    )
    .unwrap();
    assert!(
        actual == expected,
        "logical snapshot differs from public canonical snapshot"
    );
    assert_eq!(
        view.canonical_binary_size(0, DEFAULT_BINARY_BUDGET)
            .unwrap(),
        expected.len()
    );
    actual
}

fn same(a: Result<PublicDocument, PeleError>, b: Result<PublicDocument, PeleError>, label: &str) {
    match (a, b) {
        (Ok(a), Ok(b)) => assert!(bytes(&a) == bytes(&b), "graph table differs: {label}"),
        (Err(a), Err(b)) => assert!(a == b, "parse/create error differs: {label}"),
        _ => panic!("parse/create acceptance differs: {label}"),
    }
}
fn legacy_engine() -> PublicEngine {
    let mut engine = PublicEngine::empty();
    engine.register(Legacy::new());
    engine
}
#[test]
fn sankey_owned_and_legacy_match_complete_frozen_source_corpus() {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let file = std::fs::File::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/pele/sankey.json.gz"
    ))
    .unwrap();
    let mut data = Vec::new();
    flate2::read::GzDecoder::new(file)
        .read_to_end(&mut data)
        .unwrap();
    let corpus: serde_json::Value = serde_json::from_slice(&data).unwrap();
    let cases = corpus["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 11_616);
    let native = PublicEngine::with_builtins();
    let legacy = legacy_engine();
    let mut digest = Sha256::new();
    let (mut accepted, mut rejected) = (0usize, 0usize);
    for case in cases {
        let source = case["source"].as_str().unwrap();
        let options: ParseOptions = serde_json::from_value(case["options"].clone()).unwrap();
        let actual = native.parse(source, &options);
        let id = case["id"].as_str().unwrap();
        digest.update((id.len() as u64).to_le_bytes());
        digest.update(id.as_bytes());
        let value = match &actual {
            Ok(document) => {
                accepted += 1;
                digest.update([1]);
                bytes(document)
            }
            Err(error) => {
                rejected += 1;
                digest.update([0]);
                serde_json::to_vec(error).unwrap()
            }
        };
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value);
        same(actual, legacy.parse(source, &options), id);
    }
    // Frozen public graph frames and errors captured at 9ae8142, before the
    // shared construction sink refactor. This independently fixes string/node
    // order even when both current codec routes share the same builder.
    assert_eq!((accepted, rejected), (5_047, 6_569));
    assert_eq!(
        format!("{:x}", digest.finalize()),
        "de51448521928799f26cf2e4fe02c4804a323e7ec5bd7a623ca5932f79e8fe4c"
    );
}
#[test]
fn sankey_owned_retained_methods_edits_and_replacement_match_legacy() {
    let mut native = PublicEngine::with_builtins();
    let legacy = legacy_engine();
    for source in [
        SOURCE,
        "sankey-beta\nA,B,-0\nA,C,Infinity\nB,C,NaN",
        "---\ntitle: λ 😀\n---\nsankey-beta\n\"00\",\"B label\",2\n\"B label\",\"00\",0",
    ] {
        let mut a = native.parse(source, &Default::default()).unwrap();
        let mut b = legacy.parse(source, &Default::default()).unwrap();
        assert!(bytes(&a) == bytes(&b));
        assert_eq!(
            a.render(&Default::default(), &mut MetricsResources),
            b.render(&Default::default(), &mut MetricsResources)
        );
        for document in [&mut a, &mut b] {
            let mut graph = document.snapshot().unwrap();
            let id = graph.strings.len() as u32;
            graph.strings.push(JsString::from("fresh 😀"));
            document.apply(document.revision(), graph).unwrap();
            let method = document
                .model_schema()
                .methods
                .iter()
                .find(|method| method.name == "findOrCreateNode")
                .unwrap()
                .id;
            let receiver = document.snapshot().unwrap().root;
            let outcome = document
                .invoke(
                    document.revision(),
                    MethodCall {
                        method,
                        receiver,
                        arguments: vec![Atom::String(id)],
                    },
                    &mut MetricsResources,
                )
                .unwrap();
            assert!(outcome.result.is_ok());
        }
        assert!(bytes(&a) == bytes(&b));
        assert_eq!(
            a.render(&Default::default(), &mut MetricsResources),
            b.render(&Default::default(), &mut MetricsResources)
        );
    }
    let mut a = native.parse(SOURCE, &Default::default()).unwrap();
    let mut b = legacy.parse(SOURCE, &Default::default()).unwrap();
    native.register(Legacy {
        reject_create: true,
        ..Legacy::new()
    });
    assert!(native.parse(SOURCE, &Default::default()).is_err());
    drop(native);
    assert_eq!(
        a.render(&Default::default(), &mut MetricsResources),
        b.render(&Default::default(), &mut MetricsResources)
    );
}
#[test]
fn sankey_custom_delegation_cannot_forge_canonical_parser_provenance() {
    let parsed = Sankey
        .parse(ParseContext {
            source: SOURCE,
            config: &Object::default(),
            title: None,
            line_offset: 0,
            max_edges: 5000,
        })
        .unwrap();
    let mut models = vec![parsed.clone()];
    let mut duplicate = parsed.clone();
    duplicate.nodes.push(duplicate.nodes[0].clone());
    duplicate.links[0].source.id = "detached source".into();
    duplicate.links[0].value = f64::from_bits(0x7ff8_0000_0000_0042);
    duplicate.links[1].value = -0.0;
    models.push(duplicate);
    let mut invalid = parsed;
    invalid.nodes[0].index = usize::MAX;
    invalid.links[0].source = SankeyNode {
        id: "stale endpoint".into(),
        index: usize::MAX,
    };
    models.push(invalid);
    for model in models {
        let mut engine = PublicEngine::empty();
        engine.register(Delegated(Legacy {
            supplied: Some(model.clone()),
            ..Legacy::new()
        }));
        let mut actual = engine.parse(SOURCE, &Default::default()).unwrap();
        let mut expected =
            PublicDocument::from_module(Sankey, &model, actual.retained_config().clone()).unwrap();
        assert!(bytes(&actual) == bytes(&expected));
        assert_eq!(
            actual.render(&Default::default(), &mut MetricsResources),
            expected.render(&Default::default(), &mut MetricsResources)
        );
    }
}
#[test]
fn public_owned_default_hook_preserves_legacy_policy_dispatch_and_errors() {
    let module = Legacy::new();
    let observed = module.observed.clone();
    let mut engine = PublicEngine::empty();
    engine.register(module);
    let ordinary = engine.parse(SOURCE, &Default::default()).unwrap();
    assert_eq!(observed.lock().unwrap().calls, ["parse", "create"]);
    assert_eq!(
        &observed.lock().unwrap().configs[0],
        ordinary.retained_config()
    );
    observed.lock().unwrap().calls.clear();
    let bits = 0x7ff8_0000_0000_0042;
    let edge = EdgeLimit::Number(f64::from_bits(bits));
    engine
        .parse_with_limits(
            SOURCE,
            &Default::default(),
            ParseLimits {
                source: None,
                max_edges: Some(edge),
            },
        )
        .unwrap();
    assert_eq!(
        observed.lock().unwrap().calls,
        [
            format!("parse-edge:{bits:016x}"),
            "parse".into(),
            format!("create-edge:{bits:016x}")
        ]
    );
    observed.lock().unwrap().calls.clear();
    let config = Object::default();
    engine
        .parse_prepared_as(
            &Sankey.descriptor().id,
            ParseContext {
                source: SOURCE,
                config: &config,
                title: Some("prepared"),
                line_offset: 0,
                max_edges: 7,
            },
            EdgeLimit::Null,
        )
        .unwrap();
    assert_eq!(
        observed.lock().unwrap().calls,
        ["parse-prepared:null", "parse", "create-prepared:null"]
    );
    let module = Legacy {
        reject_create: true,
        ..Legacy::new()
    };
    let observed = module.observed.clone();
    engine.register(module);
    let error = engine.parse(SOURCE, &Default::default()).err().unwrap();
    assert_eq!(
        error,
        PeleError::new("extension construction failed", ErrorCode::Semantic)
    );
    assert_eq!(observed.lock().unwrap().calls, ["parse", "create"]);
}

#[test]
fn sankey_owned_parse_keeps_source_limits_and_prepared_publication() {
    let builtin = PublicEngine::with_builtins();
    let legacy = legacy_engine();
    for source in [SOURCE, "sankey-beta\nA,B", "sankey-beta\nA,B,2\ninvalid"] {
        for limit in [
            SourceLimit::Null,
            SourceLimit::Number(5.),
            SourceLimit::Number(f64::INFINITY),
        ] {
            let limits = ParseLimits {
                source: Some(limit),
                max_edges: Some(EdgeLimit::Null),
            };
            same(
                builtin.parse_with_limits(source, &Default::default(), limits),
                legacy.parse_with_limits(source, &Default::default(), limits),
                "source/edge admission",
            );
        }
    }
    let config = Object::default();
    let context = || ParseContext {
        source: SOURCE,
        config: &config,
        title: Some("raw prepared 😀"),
        line_offset: 0,
        max_edges: 1,
    };
    same(
        builtin.parse_prepared_as(&Sankey.descriptor().id, context(), EdgeLimit::Undefined),
        legacy.parse_prepared_as(&Sankey.descriptor().id, context(), EdgeLimit::Undefined),
        "prepared legacy path",
    );
}

#[test]
fn sankey_indexed_source_keeps_first_helper_latent_errors_and_raw_recovery() {
    let mut actual = PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap();
    let mut expected = legacy_engine().parse(SOURCE, &Default::default()).unwrap();
    let initial = actual.snapshot().unwrap();
    let existing = initial.strings.iter().position(|s| s.is("A")).unwrap() as u32;
    // No intervening apply: the first helper starts from the direct source state.
    for argument in [Atom::String(existing), Atom::Number(123.)] {
        let invoke = |document: &mut PublicDocument| {
            let method = document
                .model_schema()
                .methods
                .iter()
                .find(|m| m.name == "findOrCreateNode")
                .unwrap()
                .id;
            document
                .invoke(
                    document.revision(),
                    MethodCall {
                        method,
                        receiver: document.snapshot().unwrap().root,
                        arguments: vec![argument.clone()],
                    },
                    &mut MetricsResources,
                )
                .unwrap()
                .result
        };
        assert_eq!(
            serde_json::to_vec(&invoke(&mut actual)).unwrap(),
            serde_json::to_vec(&invoke(&mut expected)).unwrap()
        );
        assert!(bytes(&actual) == bytes(&expected));
    }
    // A numeric ID is accepted by the helper but invalid for the render
    // projection. The node/Map mutation must survive that latent error.
    let before = bytes(&actual);
    let error = actual
        .render(&Default::default(), &mut MetricsResources)
        .unwrap_err();
    assert_eq!(
        error,
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err()
    );
    assert_eq!(error.message, "Expected a public string");
    assert!(bytes(&actual) == before);
    assert!(bytes(&actual) == bytes(&expected));
    for document in [&mut actual, &mut expected] {
        document
            .apply(document.revision(), initial.clone())
            .unwrap();
    }
    let mut raw = PublicDocument::from_graph(
        Sankey,
        actual.snapshot().unwrap(),
        actual.retained_config().clone(),
    )
    .unwrap();
    let rendered = actual
        .render(&Default::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(
        rendered,
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap()
    );
    assert_eq!(
        rendered,
        raw.render(&Default::default(), &mut MetricsResources)
            .unwrap()
    );
}

#[test]
fn sankey_indexed_source_preserves_resource_order_and_public_typed_projection() {
    struct Trace {
        calls: Vec<(String, u64, u8)>,
        reject_at: Option<usize>,
    }
    impl RenderResources for Trace {
        fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
            self.calls
                .push((request.text.into(), request.size.to_bits(), request.style));
            if self.reject_at == Some(self.calls.len()) {
                return Err(PeleError::new(
                    "indexed source host rejection",
                    ErrorCode::Semantic,
                ));
            }
            MetricsResources.text_width(request)
        }
    }
    let source = "---\ntitle: λ 😀\n---\nsankey\n\"A label\",B,-0\nB,C,2.5";
    let typed = Engine::with_builtins()
        .parse(source, &Default::default())
        .unwrap();
    let model = typed.model::<SankeyModel>().unwrap();
    assert_eq!(model.links[0].source.id, model.nodes[0].id);
    assert_eq!(model.links[0].target.id, model.nodes[1].id);
    assert_eq!(model.links[0].value.to_bits(), (-0.0f64).to_bits());
    let json = serde_json::to_value(model).unwrap();
    assert_eq!(json["links"][0]["source"], json["nodes"][0]);
    assert_eq!(json["links"][0]["target"], json["nodes"][1]);

    let mut actual = PublicEngine::with_builtins()
        .parse(source, &Default::default())
        .unwrap();
    let mut expected = legacy_engine().parse(source, &Default::default()).unwrap();
    for reject_at in [None, Some(1), Some(2), None] {
        let mut a = Trace {
            calls: Vec::new(),
            reject_at,
        };
        let mut b = Trace {
            calls: Vec::new(),
            reject_at,
        };
        let result = actual.render(&Default::default(), &mut a);
        assert_eq!(result, expected.render(&Default::default(), &mut b));
        assert_eq!(a.calls, b.calls);
        assert!(a.calls.len() >= reject_at.unwrap_or(1));
        if reject_at.is_none() {
            assert!(result.is_ok());
        } else {
            assert_eq!(result.unwrap_err().message, "indexed source host rejection");
        }
        assert!(bytes(&actual) == bytes(&expected));
    }
}

#[test]
fn sankey_logical_outcomes_preserve_first_use_partial_errors_and_recovery() {
    let mut actual = PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap();
    let mut expected = legacy_engine().parse(SOURCE, &Default::default()).unwrap();
    let original = expected.snapshot().unwrap();
    // Actual document has not exported/materialized any ModelGraph before its
    // first unchanged render or its first helper call.
    {
        let a = actual
            .render_operation_read(0, &Default::default(), &mut MetricsResources)
            .unwrap();
        let b = expected
            .render_operation(0, &Default::default(), &mut MetricsResources)
            .unwrap();
        match (&a.result, &b.result) {
            (Ok(a), Ok(b)) => {
                assert_eq!(a, b);
                assert_eq!(a.rendered.width.to_bits(), b.rendered.width.to_bits());
                assert_eq!(a.rendered.height.to_bits(), b.rendered.height.to_bits());
            }
            _ => assert_eq!(
                serde_json::to_vec(&a.result).unwrap(),
                serde_json::to_vec(&b.result).unwrap()
            ),
        }
        assert_eq!(a.revision, b.revision);
        let canonical =
            encode_binary_exchange(b.revision, &b.graph, &[], DEFAULT_BINARY_BUDGET).unwrap();
        for _ in 0..2 {
            assert_eq!(
                a.graph.canonical_binary_size(0, canonical.len()).unwrap(),
                canonical.len()
            );
        }
        assert!(
            a.graph
                .encode_binary(a.revision, &[], DEFAULT_BINARY_BUDGET)
                .unwrap()
                == encode_binary_exchange(b.revision, &b.graph, &[], DEFAULT_BINARY_BUDGET)
                    .unwrap()
        );
    }
    let call = MethodCall {
        method: MethodId(1),
        receiver: original.root.clone(),
        arguments: vec![Atom::Number(123.)],
    };
    {
        let a = actual
            .invoke_read(1, call.clone(), &mut MetricsResources)
            .unwrap();
        let b = expected.invoke(1, call, &mut MetricsResources).unwrap();
        assert_eq!(
            serde_json::to_vec(&a.result).unwrap(),
            serde_json::to_vec(&b.result).unwrap()
        );
        assert!(
            a.graph
                .encode_binary(a.revision, &[], DEFAULT_BINARY_BUDGET)
                .unwrap()
                == encode_binary_exchange(b.revision, &b.graph, &[], DEFAULT_BINARY_BUDGET)
                    .unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&a.graph).unwrap(),
            serde_json::to_vec(&b.graph).unwrap()
        );
    }
    assert_eq!(
        actual
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err(),
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err()
    );
    for document in [&mut actual, &mut expected] {
        document.apply(2, original.clone()).unwrap();
    }
    let root = match original.root {
        Atom::Ref(id) => id,
        _ => panic!(),
    };
    let nodes = original.nodes[root as usize]
        .properties()
        .iter()
        .find(|p| original.strings[p.key as usize].is("nodes"))
        .unwrap()
        .value
        .clone();
    let nodes = match nodes {
        Atom::Ref(id) => id,
        _ => panic!(),
    };
    let mut locked = original.clone();
    let length_key = locked.strings.len() as u32;
    locked.strings.push(JsString::from("length"));
    let Node::Array { length, properties } = &mut locked.nodes[nodes as usize] else {
        panic!()
    };
    properties.push(Property {
        key: length_key,
        value: Atom::Number(*length as f64),
        flags: 0,
    });
    for document in [&mut actual, &mut expected] {
        document.apply(3, locked.clone()).unwrap();
    }
    // Repeated logical exports precede a helper that makes partial writes and
    // then fails on the readonly array length. Its resulting state must export.
    assert!(bytes(&actual) == bytes(&expected));
    assert!(bytes(&actual) == bytes(&expected));
    let call = MethodCall {
        method: MethodId(1),
        receiver: Atom::Ref(root),
        arguments: vec![Atom::Number(456.)],
    };
    {
        let a = actual
            .invoke_read(4, call.clone(), &mut MetricsResources)
            .unwrap();
        let b = expected.invoke(4, call, &mut MetricsResources).unwrap();
        assert!(a.result.is_err());
        assert_eq!(
            serde_json::to_vec(&a.result).unwrap(),
            serde_json::to_vec(&b.result).unwrap()
        );
        assert_eq!(
            serde_json::to_vec(&a.graph).unwrap(),
            serde_json::to_vec(&b.graph).unwrap()
        );
        assert!(
            a.graph
                .encode_binary(a.revision, &[], DEFAULT_BINARY_BUDGET)
                .unwrap()
                == encode_binary_exchange(b.revision, &b.graph, &[], DEFAULT_BINARY_BUDGET)
                    .unwrap()
        );
    }
    for document in [&mut actual, &mut expected] {
        document.apply(5, original.clone()).unwrap();
    }
    assert_eq!(
        actual
            .render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap()
    );
}

#[test]
fn sankey_logical_history_and_combined_transport_limits_match_public_graphs() {
    let mut actual = PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap();
    let mut expected = legacy_engine().parse(SOURCE, &Default::default()).unwrap();
    assert!(bytes(&actual) == bytes(&expected));
    assert!(bytes(&actual) == bytes(&expected));
    let mut graph = expected.snapshot().unwrap();
    let key = graph.strings.len() as u32;
    graph.strings.extend([
        JsString(vec![0xd800, 0, 0xdc00]),
        JsString::from("A"),
        JsString::from("A"),
    ]);
    let nan = f64::from_bits(0x7ff8000000000042);
    graph.nodes.push(Node::Date {
        milliseconds: -0.,
        properties: vec![Property {
            key,
            value: Atom::Number(nan),
            flags: 1,
        }],
    });
    graph.nodes.push(Node::Set {
        values: vec![Atom::Number(-0.), Atom::Number(nan)],
        properties: vec![],
    });
    graph.nodes.push(Node::Error {
        error_type: ErrorObjectKind::Type,
        properties: vec![],
    });
    for document in [&mut actual, &mut expected] {
        document.apply(0, graph.clone()).unwrap();
    }
    let view = actual.snapshot_read().unwrap();
    let old = expected.snapshot().unwrap();
    assert_eq!(
        serde_json::to_vec(&view).unwrap(),
        serde_json::to_vec(&old).unwrap()
    );
    let metadata = b"\"quoted UTF8 \xf0\x9f\x98\x80\"";
    let frame = encode_binary_exchange(1, &old, metadata, DEFAULT_BINARY_BUDGET).unwrap();
    assert_eq!(
        view.canonical_binary_size(metadata.len(), frame.len())
            .unwrap(),
        frame.len() - metadata.len()
    );
    assert!(view.encode_binary(1, metadata, frame.len()).unwrap() == frame);
    assert_eq!(
        view.canonical_binary_size(metadata.len(), frame.len() - 1)
            .unwrap_err(),
        encode_binary_exchange(1, &old, metadata, frame.len() - 1).unwrap_err()
    );
    assert_eq!(
        view.encode_binary(1, metadata, frame.len() - 1)
            .unwrap_err(),
        encode_binary_exchange(1, &old, metadata, frame.len() - 1).unwrap_err()
    );

    // Reuse a successful count with different metadata/current byte limits.
    let empty = view
        .canonical_binary_size(0, DEFAULT_BINARY_BUDGET)
        .unwrap();
    assert_eq!(view.canonical_binary_size(31, empty + 31).unwrap(), empty);
    assert_eq!(
        view.canonical_binary_size(32, empty + 31).unwrap_err(),
        encode_binary_exchange(1, &old, &[0; 32], empty + 31).unwrap_err()
    );
    drop(view);

    // Accepted root and string-table replacement, including detached history,
    // must replace every prior export's validation and size facts.
    let original = graph.clone();
    graph.root = Atom::Undefined;
    graph.strings[key as usize] = JsString(vec![0xdc00, 0xd800]);
    graph.strings.push(JsString::from("new retained history"));
    for document in [&mut actual, &mut expected] {
        document.apply(1, graph.clone()).unwrap();
    }
    assert!(bytes(&actual) == bytes(&expected));
    assert_eq!(
        actual
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err(),
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err()
    );

    // Invalid incoming references and schema IDs still fail before adoption;
    // a previously accepted graph cannot authorize replacement data.
    for root in [
        Atom::Ref(u32::MAX),
        Atom::String(u32::MAX),
        Atom::BuiltinFunction(u32::MAX),
    ] {
        let mut invalid = graph.clone();
        invalid.root = root;
        let a = actual.apply(2, invalid.clone()).unwrap_err();
        let b = expected.apply(2, invalid).unwrap_err();
        assert_eq!(a, b);
        assert_eq!(actual.revision(), 2);
        assert!(bytes(&actual) == bytes(&expected));
    }
    for document in [&mut actual, &mut expected] {
        document.apply(2, original.clone()).unwrap();
    }
    assert!(bytes(&actual) == bytes(&expected));
    assert_eq!(
        actual
            .render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap()
    );
}

#[test]
fn public_logical_reader_cannot_borrow_another_documents_admission() {
    // A public extension may delegate graph reads and even forward an opaque
    // capability, but different root data still requires independent admission.
    struct Forwarded {
        document: PublicDocument,
        graph: ModelGraph,
    }
    impl PublicGraphRead for Forwarded {
        fn trusted_read(&self) -> Option<TrustedGraphRead<'_>> {
            match self.document.snapshot_read().unwrap() {
                GraphReadView::Logical(graph) => graph.trusted_read(),
                GraphReadView::Arena(_) => None,
            }
        }
        fn extent(&self) -> GraphExtent {
            PublicGraphRead::extent(&self.graph)
        }
        fn root(&self) -> Atom {
            Atom::Ref(u32::MAX)
        }
        fn header(&self, node: usize) -> Result<GraphHeader, PeleError> {
            PublicGraphRead::header(&self.graph, node)
        }
        fn string_length(&self, id: usize) -> Result<usize, PeleError> {
            PublicGraphRead::string_length(&self.graph, id)
        }
        fn string_lengths(
            &self,
            visit: &mut dyn FnMut(usize) -> Result<(), PeleError>,
        ) -> Result<(), PeleError> {
            PublicGraphRead::string_lengths(&self.graph, visit)
        }
        fn string_chunks(
            &self,
            id: usize,
            visit: &mut dyn FnMut(&[u16]) -> Result<(), PeleError>,
        ) -> Result<(), PeleError> {
            PublicGraphRead::string_chunks(&self.graph, id, visit)
        }
        fn properties(
            &self,
            node: usize,
            start: usize,
            visit: &mut dyn FnMut(&[Property]) -> Result<(), PeleError>,
        ) -> Result<(), PeleError> {
            PublicGraphRead::properties(&self.graph, node, start, visit)
        }
        fn map_entries(
            &self,
            node: usize,
            start: usize,
            visit: &mut dyn FnMut(&[(Atom, Atom)]) -> Result<(), PeleError>,
        ) -> Result<(), PeleError> {
            PublicGraphRead::map_entries(&self.graph, node, start, visit)
        }
        fn set_values(
            &self,
            node: usize,
            start: usize,
            visit: &mut dyn FnMut(&[Atom]) -> Result<(), PeleError>,
        ) -> Result<(), PeleError> {
            PublicGraphRead::set_values(&self.graph, node, start, visit)
        }
    }
    let document = PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap();
    for _ in 0..2 {
        document
            .snapshot_read()
            .unwrap()
            .canonical_binary_size(0, DEFAULT_BINARY_BUDGET)
            .unwrap();
    }
    let graph = document.snapshot().unwrap();
    let mut invalid = graph.clone();
    invalid.root = Atom::Ref(u32::MAX);
    let expected = invalid.validate().unwrap_err();
    assert_eq!(expected.code, ErrorCode::Semantic);
    let forwarded = Forwarded { document, graph };
    assert!(forwarded.trusted_read().is_some());
    for _ in 0..2 {
        assert_eq!(
            GraphReadView::Logical(&forwarded)
                .canonical_binary_size(0, DEFAULT_BINARY_BUDGET)
                .unwrap_err(),
            expected
        );
    }
}

#[test]
fn source_layout_updates_preserve_original_base_and_complete_current_state() {
    // This public transport consumer reads the declarative frame independently:
    // original catalog/layout bytes belong to the initial frame, while updates
    // carry complete replacements relative to that base, never the prior update.
    struct Packet<'a> {
        header: &'a [u8],
        strings: usize,
        original: Vec<JsString>,
        immutable: &'a [u8],
        columns: Vec<Vec<(u8, u64)>>,
        patches: Vec<(usize, JsString)>,
        overrides: &'a [u8],
    }
    fn u32_at(bytes: &[u8], pos: &mut usize) -> usize {
        let value = u32::from_le_bytes(bytes[*pos..*pos + 4].try_into().unwrap());
        *pos += 4;
        value as usize
    }
    fn text(bytes: &[u8], pos: &mut usize) -> JsString {
        let len = u32_at(bytes, pos);
        let end = *pos + len * 2;
        let units = bytes[*pos..end]
            .chunks_exact(2)
            .map(|unit| u16::from_le_bytes(unit.try_into().unwrap()))
            .collect();
        *pos = end;
        JsString(units)
    }
    fn packet(bytes: &[u8], full: bool) -> Packet<'_> {
        assert_eq!(&bytes[..8], if full { b"PLCSRC03" } else { b"PLCSRU01" });
        assert_ne!(u64::from_le_bytes(bytes[8..16].try_into().unwrap()), 0);
        let mut pos = 37;
        let strings = u32_at(bytes, &mut pos);
        let _nodes = u32_at(bytes, &mut pos);
        let base_start = pos;
        let mut original = Vec::new();
        let mut column_values = Vec::new();
        if full {
            let count = u32_at(bytes, &mut pos);
            for _ in 0..count {
                original.push(text(bytes, &mut pos));
            }
            let columns = u32_at(bytes, &mut pos);
            for _ in 0..columns {
                let tag = bytes[pos];
                pos += 1;
                assert!([4, 5, 6].contains(&tag));
                let count = u32_at(bytes, &mut pos);
                while pos % 4 != 0 {
                    assert_eq!(bytes[pos], 0);
                    pos += 1;
                }
                let mut values = Vec::new();
                for _ in 0..count {
                    let lo = u32_at(bytes, &mut pos) as u64;
                    let hi = if tag == 4 {
                        u32_at(bytes, &mut pos) as u64
                    } else {
                        0
                    };
                    values.push((tag, lo | (hi << 32)));
                }
                column_values.push(values);
            }
            let groups = u32_at(bytes, &mut pos);
            for _ in 0..groups {
                // first/count/kind/aux0/aux1, then fixed properties,
                // indexed properties, indexed Map pairs. Expr is ten bytes.
                pos += 17;
                let properties = u32_at(bytes, &mut pos);
                pos += properties * 21;
                let dynamic = u32_at(bytes, &mut pos);
                pos += dynamic * 25;
                let maps = u32_at(bytes, &mut pos);
                pos += maps * 24;
            }
        }
        let base_end = pos;
        let count = u32_at(bytes, &mut pos);
        let mut patches = Vec::new();
        for _ in 0..count {
            let id = u32_at(bytes, &mut pos);
            assert!(id < strings);
            assert!(patches.last().is_none_or(|(previous, _)| *previous < id));
            patches.push((id, text(bytes, &mut pos)));
        }
        Packet {
            header: &bytes[8..45],
            strings,
            original,
            immutable: &bytes[base_start..base_end],
            columns: column_values,
            patches,
            overrides: &bytes[pos..],
        }
    }
    fn verify(document: &PublicDocument, expected: &PublicDocument, initial: &[u8]) {
        let graph = expected.snapshot().unwrap();
        let view = document.snapshot_read().unwrap();
        let full_bytes = view.source_layout(document.revision()).unwrap().unwrap();
        let update_bytes = view
            .source_layout_update(document.revision())
            .unwrap()
            .unwrap();
        let base = packet(initial, true);
        let full = packet(&full_bytes, true);
        let update = packet(&update_bytes, false);
        assert_eq!(full.header, update.header);
        assert_eq!(&full.header[..8], &base.header[..8]);
        assert_eq!(full.immutable, base.immutable);
        assert_eq!(full.original, base.original);
        assert_eq!(full.columns, base.columns);
        assert_eq!(full.patches, update.patches);
        assert_eq!(full.overrides, update.overrides);
        let mut restored: Vec<_> = (0..update.strings)
            .map(|id| base.original.get(id).cloned())
            .collect();
        for (id, value) in update.patches {
            restored[id] = Some(value);
        }
        let restored: Vec<_> = restored.into_iter().map(Option::unwrap).collect();
        assert_eq!(restored, graph.strings);
        let metadata = b"{\"result\":null}";
        let encoded =
            encode_binary_exchange(document.revision(), &graph, metadata, DEFAULT_BINARY_BUDGET)
                .unwrap();
        assert_eq!(
            view.canonical_binary_size(metadata.len(), encoded.len())
                .unwrap(),
            encoded.len() - metadata.len()
        );
        assert_eq!(
            view.canonical_binary_size(metadata.len(), encoded.len() - 1)
                .unwrap_err(),
            encode_binary_exchange(document.revision(), &graph, metadata, encoded.len() - 1)
                .unwrap_err()
        );
        assert!(
            view.encode_binary(document.revision(), metadata, encoded.len())
                .unwrap()
                == encoded
        );
    }
    // Compare source column contents with the independent legacy provider's
    // canonical public records, not private writer helpers or column counters.
    // Unicode/alignment shifts, numeric spelling, and catalog
    // collisions keep their public meaning and exact IEEE bits.
    for source in [
        "sankey\nA,B,1",
        "---\ntitle: λ 😀\n---\nsankey-beta\n\"0\",\"B label\",-0\n\"B label\",C,Infinity\nC,\"0\",NaN",
        "sankey\nA,B,5e-324\nA,C,-2.5",
    ] {
        let actual = PublicEngine::with_builtins().parse(source, &Default::default()).unwrap();
        let expected = legacy_engine().parse(source, &Default::default()).unwrap();
        let graph = expected.snapshot().unwrap();
        let view = actual.snapshot_read().unwrap();
        let raw = view.source_layout(0).unwrap().unwrap();
        let full = packet(&raw, true);
        let field = |id: usize, name: &str| -> &Atom {
            &graph.nodes[id].properties().iter().find(|p| graph.strings[p.key as usize].is(name)).unwrap().value
        };
        let Atom::Ref(root) = graph.root else { panic!("root") };
        let Atom::Ref(nodes) = field(root as usize, "nodes") else { panic!("nodes") };
        let Atom::Ref(links) = field(root as usize, "links") else { panic!("links") };
        let node_ids: Vec<_> = graph.nodes[*nodes as usize].properties().iter().map(|p| match p.value { Atom::Ref(id) => id as usize, _ => panic!("node ref") }).collect();
        let link_ids: Vec<_> = graph.nodes[*links as usize].properties().iter().map(|p| match p.value { Atom::Ref(id) => id as usize, _ => panic!("link ref") }).collect();
        let mut columns = vec![Vec::new(); 5];
        for id in &node_ids {
            let Atom::String(value) = field(*id, "id") else { panic!("label") };
            columns[0].push((5, u64::from(*value)));
        }
        for id in &link_ids {
            let Atom::Ref(source) = field(*id, "source") else { panic!("source") };
            let Atom::Ref(target) = field(*id, "target") else { panic!("target") };
            let Atom::Number(value) = field(*id, "value") else { panic!("value") };
            columns[1].push((6, u64::from(*source)));
            columns[2].push((6, u64::from(*target)));
            columns[3].push((4, value.to_bits()));
        }
        for index in 0..node_ids.len().max(link_ids.len()) {
            let value = graph.strings.iter().position(|text| text.is(&index.to_string())).unwrap();
            columns[4].push((5, value as u64));
        }
        assert_eq!(full.columns, columns, "{source}");
        verify(&actual, &expected, &raw);
    }
    let source = "---\ntitle: original 😀\n---\nsankey\nA,B,2\nA,C,3";
    let mut actual = PublicEngine::with_builtins()
        .parse(source, &Default::default())
        .unwrap();
    let mut expected = legacy_engine().parse(source, &Default::default()).unwrap();
    let original = expected.snapshot().unwrap();
    let initial = actual
        .snapshot_read()
        .unwrap()
        .source_layout(0)
        .unwrap()
        .unwrap();
    verify(&actual, &expected, &initial);
    verify(&actual, &expected, &initial);
    // Different owner, identical diagram: bases are never interchangeable.
    let other = PublicEngine::with_builtins()
        .parse(source, &Default::default())
        .unwrap();
    let other_bytes = other
        .snapshot_read()
        .unwrap()
        .source_layout(0)
        .unwrap()
        .unwrap();
    assert_ne!(&initial[8..16], &other_bytes[8..16]);

    let mut edited = original.clone();
    let label = edited.strings.iter().position(|s| s.is("A")).unwrap();
    edited.strings[label] = JsString::from("replaced λ 😀");
    edited.strings.push(JsString(vec![0xd800, 0, 0xdc00]));
    edited.root = Atom::Undefined;
    for document in [&mut actual, &mut expected] {
        document.apply(0, edited.clone()).unwrap();
    }
    verify(&actual, &expected, &initial);
    assert_eq!(
        actual
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err(),
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err()
    );

    let empty = ModelGraph {
        version: original.version,
        strings: vec![],
        nodes: vec![],
        root: Atom::Undefined,
    };
    for document in [&mut actual, &mut expected] {
        document.apply(1, empty.clone()).unwrap();
    }
    verify(&actual, &expected, &initial);
    for document in [&mut actual, &mut expected] {
        document.apply(2, original.clone()).unwrap();
    }
    verify(&actual, &expected, &initial);

    let call = MethodCall {
        method: MethodId(1),
        receiver: original.root.clone(),
        arguments: vec![Atom::Number(123.)],
    };
    {
        let a = actual
            .invoke_read(3, call.clone(), &mut MetricsResources)
            .unwrap();
        let b = expected.invoke(3, call, &mut MetricsResources).unwrap();
        assert_eq!(
            serde_json::to_vec(&a.result).unwrap(),
            serde_json::to_vec(&b.result).unwrap()
        );
    }
    verify(&actual, &expected, &initial);
    assert_eq!(
        actual
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err(),
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err()
    );
    for document in [&mut actual, &mut expected] {
        document.apply(4, original.clone()).unwrap();
    }
    verify(&actual, &expected, &initial);
    assert_eq!(
        actual
            .render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        expected
            .render(&Default::default(), &mut MetricsResources)
            .unwrap()
    );
}

#[test]
fn sankey_first_export_limit_failure_retries_without_prior_count_or_materialization() {
    let expected = legacy_engine().parse(SOURCE, &Default::default()).unwrap();
    let graph = expected.snapshot().unwrap();
    let oracle = GraphReadView::Arena(std::borrow::Cow::Borrowed(&graph));
    for metadata in [0usize, 31, 32] {
        let actual = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let payload = vec![0; metadata];
        let frame = encode_binary_exchange(0, &graph, &payload, DEFAULT_BINARY_BUDGET).unwrap();
        // This is the source document's first snapshot and first size request.
        // Its failing export must not prevent an exact-budget retry.
        let view = actual.snapshot_read().unwrap();
        assert_eq!(
            view.canonical_binary_size(metadata, frame.len() - 1)
                .unwrap_err(),
            oracle
                .canonical_binary_size(metadata, frame.len() - 1)
                .unwrap_err()
        );
        assert_eq!(
            view.canonical_binary_size(metadata, frame.len()).unwrap(),
            frame.len() - metadata
        );
        for (size, budget) in [
            (usize::MAX, usize::MAX),
            (0, 0),
            (0, frame.len() - metadata),
            (31, frame.len() - metadata + 31),
            (32, frame.len() - metadata + 31),
            (32, frame.len() - metadata + 32),
        ] {
            assert_eq!(
                view.canonical_binary_size(size, budget),
                oracle.canonical_binary_size(size, budget)
            );
        }
        assert_eq!(view.encode_binary(0, &payload, frame.len()).unwrap(), frame);
        assert_eq!(
            serde_json::to_vec(&view).unwrap(),
            serde_json::to_vec(&graph).unwrap()
        );
    }
}

#[test]
fn sankey_first_helper_without_export_preserves_partial_state_and_repair() {
    for locked in [false, true] {
        let mut actual = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut expected = legacy_engine().parse(SOURCE, &Default::default()).unwrap();
        // Only the separate legacy oracle supplies receiver IDs and edits.
        // The source owner has had no snapshot, size request, or render.
        let original = expected.snapshot().unwrap();
        let mut before = original.clone();
        if locked {
            let Atom::Ref(root) = original.root else {
                panic!()
            };
            let Atom::Ref(nodes) = original.nodes[root as usize]
                .properties()
                .iter()
                .find(|p| original.strings[p.key as usize].is("nodes"))
                .unwrap()
                .value
            else {
                panic!()
            };
            let key = before.strings.len() as u32;
            before.strings.push(JsString::from("length"));
            let Node::Array { length, properties } = &mut before.nodes[nodes as usize] else {
                panic!()
            };
            properties.push(Property {
                key,
                value: Atom::Number(*length as f64),
                flags: 0,
            });
            actual.apply(0, before.clone()).unwrap();
            expected.apply(0, before.clone()).unwrap();
        }
        let revision = u64::from(locked);
        let call = MethodCall {
            method: MethodId(1),
            receiver: original.root.clone(),
            arguments: vec![Atom::Number(456.)],
        };
        {
            let a = actual
                .invoke_read(revision, call.clone(), &mut MetricsResources)
                .unwrap();
            let b = expected
                .invoke(revision, call, &mut MetricsResources)
                .unwrap();
            assert_eq!(a.result.is_err(), locked);
            assert_eq!(
                serde_json::to_vec(&a.result).unwrap(),
                serde_json::to_vec(&b.result).unwrap()
            );
            assert_eq!(a.revision, revision + 1);
            assert_eq!(a.revision, b.revision);
            assert_eq!(
                serde_json::to_vec(&a.graph).unwrap(),
                serde_json::to_vec(&b.graph).unwrap()
            );
            assert_ne!(
                serde_json::to_vec(&a.graph).unwrap(),
                serde_json::to_vec(&before).unwrap()
            );
            assert_eq!(
                a.graph
                    .encode_binary(a.revision, &[], DEFAULT_BINARY_BUDGET)
                    .unwrap(),
                encode_binary_exchange(b.revision, &b.graph, &[], DEFAULT_BINARY_BUDGET).unwrap()
            );
        }
        assert_eq!(
            actual.render(&Default::default(), &mut MetricsResources),
            expected.render(&Default::default(), &mut MetricsResources)
        );
        actual.apply(revision + 1, original.clone()).unwrap();
        expected.apply(revision + 1, original).unwrap();
        assert!(bytes(&actual) == bytes(&expected));
        assert_eq!(
            actual
                .render(&Default::default(), &mut MetricsResources)
                .unwrap(),
            expected
                .render(&Default::default(), &mut MetricsResources)
                .unwrap()
        );
    }
}
