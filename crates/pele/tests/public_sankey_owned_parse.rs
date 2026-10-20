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
    encode_binary_exchange(
        document.revision(),
        &document.snapshot_view().unwrap(),
        &[],
        DEFAULT_BINARY_BUDGET,
    )
    .unwrap()
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
