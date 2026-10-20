//! Public prepared-entry feature slice: exact eager publication and captured
//! provider policy. GraphWriter is an integration oracle, never production state.
#![cfg(feature = "web-host-er")]
use pele::compat::{
    diagrams::er::{Er, ErModel},
    host_er::{InitialWriter, Parsed},
    public::*,
    *,
};
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

struct Sink(GraphWriter);
impl InitialWriter for Sink {
    type Output = Vec<u8>;
    fn string(&mut self, value: &str) -> Result<u32, PeleError> {
        Ok(self.0.string(value))
    }
    fn node(&mut self, node: Node) -> Result<u32, PeleError> {
        Ok(self.0.node(node))
    }
    fn finish(self, root: Atom) -> Result<Vec<u8>, PeleError> {
        encoded(&self.0.finish(root)?)
    }
}
fn encoded(graph: &ModelGraph) -> Result<Vec<u8>, PeleError> {
    encode_binary_exchange(0, graph, br#"{"type":"er"}"#, DEFAULT_BINARY_BUDGET)
}
fn context<'a>(
    source: &'a str,
    config: &'a Object,
    title: Option<&'a str>,
    offset: usize,
) -> ParseContext<'a> {
    ParseContext {
        source,
        config,
        title,
        line_offset: offset,
        max_edges: DEFAULT_MAX_EDGES,
    }
}
fn compare(
    engine: &PublicEngine,
    source: &str,
    config: &Object,
    title: Option<&str>,
    offset: usize,
    limit: EdgeLimit,
) -> (bool, usize) {
    let id = DiagramType::Er.into();
    let expected = engine
        .parse_prepared_as(&id, context(source, config, title, offset), limit)
        .and_then(|document| encoded(document.snapshot_view()?.as_ref()));
    let factories = Cell::new(0);
    let actual = engine.parse_prepared_host_er_as(
        &id,
        context(source, config, title, offset),
        limit,
        |_| {
            factories.set(factories.get() + 1);
            Ok(Sink(GraphWriter::default()))
        },
    );
    match (actual, expected) {
        (Ok(Parsed::Host { policy, output }), Ok(expected)) => {
            assert_eq!(output, expected);
            assert_eq!(policy.config(), config);
            (true, factories.get())
        }
        (Err(actual), Err(expected)) => {
            assert_eq!(actual, expected);
            (false, factories.get())
        }
        _ => panic!("prepared acceptance or backend differs"),
    }
}
#[test]
fn prepared_sources_keep_verbatim_text_title_positions_and_exact_publication() {
    let engine = PublicEngine::with_builtins();
    let config = serde_json::from_str(r#"{"er":{"nodeSpacing":71},"preparedStamp":19}"#).unwrap();
    for source in [
        "erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns\n",
        "erDiagram\nA\nclassDef marked fill:red\nclass A marked",
        "erDiagram\nsubgraph box\nA\nB\nend\nA ||--o{ B : owns",
        "erDiagram\nA[\"first\"]\nA[\"replacement\"]",
        "erDiagram\n%% prepared comment\nA",
        "erDiagram\nA[\"&amp; literal\"]",
        "erDiagram\nA { int }",
        "---\ntitle: Not preprocessed\n---\nerDiagram\nA",
    ] {
        for limit in [
            EdgeLimit::Undefined,
            EdgeLimit::Null,
            EdgeLimit::Number(f64::NAN),
            EdgeLimit::Number(f64::INFINITY),
        ] {
            let (ok, factories) = compare(
                &engine,
                source,
                &config,
                Some("Explicit prepared title"),
                17,
                limit,
            );
            assert_eq!(factories, usize::from(ok));
        }
    }
    assert_eq!(
        compare(
            &engine,
            "erDiagram\nA",
            &config,
            None,
            0,
            EdgeLimit::Undefined
        ),
        (true, 1)
    );
    let title = "T".repeat(65_537);
    assert_eq!(
        compare(
            &engine,
            "erDiagram\nA",
            &config,
            Some(&title),
            3,
            EdgeLimit::Undefined
        ),
        (true, 1)
    );
}
#[test]
fn prepared_admission_and_syntax_fail_before_host_factory() {
    let engine = PublicEngine::with_builtins();
    let config = Object::default();
    assert_eq!(
        compare(
            &engine,
            "erDiagram\nA",
            &config,
            None,
            usize::MAX,
            EdgeLimit::Undefined
        ),
        (false, 0)
    );
    let mut source = format!("erDiagram\n{}", "N".repeat(8_000_001));
    for syntax in [false, true] {
        if syntax {
            source.push_str("\nB { int }");
        }
        let id = DiagramType::Er.into();
        let expected = engine
            .parse_prepared_as(
                &id,
                context(&source, &config, None, 0),
                EdgeLimit::Undefined,
            )
            .err()
            .unwrap();
        assert_eq!(
            expected.code,
            if syntax {
                ErrorCode::Syntax
            } else {
                ErrorCode::Limit
            }
        );
        assert_eq!(
            compare(&engine, &source, &config, None, 0, EdgeLimit::Undefined),
            (false, 0)
        );
    }
}
static INVALID_SCHEMA: ModelSchema = ModelSchema {
    id: "",
    version: 1,
    root: None,
    records: &[],
    prototypes: &[],
    methods: &[],
};
#[derive(Clone)]
struct Provider {
    events: Arc<Mutex<Vec<String>>>,
    invalid: bool,
}
impl DiagramModule for Provider {
    type Model = ErModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Er.descriptor()
    }
    fn parse(&self, _: ParseContext<'_>) -> Result<ErModel, PeleError> {
        panic!("prepared entry substituted ordinary parse")
    }
    fn parse_prepared(&self, c: ParseContext<'_>, limit: EdgeLimit) -> Result<ErModel, PeleError> {
        self.events.lock().unwrap().push(format!(
            "parse:{:?}:{}:{}:{limit:?}",
            c.title, c.line_offset, c.source
        ));
        Er.parse(c)
    }
    fn render(
        &self,
        m: &ErModel,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Er.render(m, c, o, h)
    }
}
impl PublicDiagramModule for Provider {
    type PublicState = <Er as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        if self.invalid {
            &INVALID_SCHEMA
        } else {
            Er.model_schema()
        }
    }
    fn try_parse_public(
        &self,
        _: ParseContext<'_>,
        _: Option<EdgeLimit>,
    ) -> Option<Result<Self::PublicState, PeleError>> {
        panic!("prepared entry called ordinary public direct hook")
    }
    fn create_public(&self, _: &ErModel) -> Result<Self::PublicState, PeleError> {
        panic!("prepared entry substituted ordinary creation")
    }
    fn create_public_prepared(
        &self,
        m: &ErModel,
        limit: EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        self.events
            .lock()
            .unwrap()
            .push(format!("create:{limit:?}"));
        Er.create_public(m)
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Er.export_public(s)
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        Er.apply_public(s, g)
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Er.render_public(s, c, o, h)
    }
}
#[test]
fn prepared_custom_provider_runs_once_and_retains_its_captured_state() {
    let mut engine = PublicEngine::with_builtins();
    let events = Arc::new(Mutex::new(vec![]));
    engine.register(Provider {
        events: events.clone(),
        invalid: false,
    });
    let config = serde_json::from_str(r#"{"stamp":71}"#).unwrap();
    let id = DiagramType::Er.into();
    let result = engine
        .parse_prepared_host_er_as(
            &id,
            context("erDiagram\nA", &config, Some("Caller title"), 13),
            EdgeLimit::Number(12.5),
            |_| -> Result<Sink, PeleError> { panic!("custom provider reached host constructor") },
        )
        .unwrap();
    let Parsed::Arena(document) = result else {
        panic!("custom provider must stay arena")
    };
    assert_eq!(
        *events.lock().unwrap(),
        [
            "parse:Some(\"Caller title\"):13:erDiagram\nA:Number(12.5)",
            "create:Number(12.5)"
        ]
    );
    assert_eq!(document.revision(), 0);
    assert_eq!(document.retained_config(), &config);
    let before = encoded(document.snapshot_view().unwrap().as_ref()).unwrap();
    engine.register(Provider {
        events: Arc::new(Mutex::new(vec![])),
        invalid: true,
    });
    drop(engine);
    assert_eq!(
        encoded(document.snapshot_view().unwrap().as_ref()).unwrap(),
        before
    );
}
#[test]
fn prepared_schema_errors_remain_after_parser_errors_and_before_creation() {
    let mut engine = PublicEngine::empty();
    let events = Arc::new(Mutex::new(vec![]));
    engine.register(Provider {
        events: events.clone(),
        invalid: true,
    });
    let config = Object::default();
    let id = DiagramType::Er.into();
    for (source, code) in [
        ("erDiagram\nA { int }", ErrorCode::Syntax),
        ("erDiagram\nA", ErrorCode::Semantic),
    ] {
        events.lock().unwrap().clear();
        let expected = engine
            .parse_prepared_as(&id, context(source, &config, None, 0), EdgeLimit::Null)
            .err()
            .unwrap();
        events.lock().unwrap().clear();
        let actual = engine
            .parse_prepared_host_er_as(
                &id,
                context(source, &config, None, 0),
                EdgeLimit::Null,
                |_| -> Result<Sink, PeleError> {
                    panic!("invalid provider reached host constructor")
                },
            )
            .err()
            .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(actual.code, code);
        let events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert!(events[0].starts_with("parse:"));
    }
}
#[test]
fn prepared_builtin_policy_keeps_config_after_registry_replacement_and_drop() {
    let mut engine = PublicEngine::with_builtins();
    let config = serde_json::from_str(r#"{"stamp":79}"#).unwrap();
    let result = engine
        .parse_prepared_host_er_as(
            &DiagramType::Er.into(),
            context("erDiagram\nA", &config, None, 0),
            EdgeLimit::Undefined,
            |_| Ok(Sink(GraphWriter::default())),
        )
        .unwrap();
    let Parsed::Host { policy, output } = result else {
        panic!("builtin host expected")
    };
    engine.register(Provider {
        events: Arc::new(Mutex::new(vec![])),
        invalid: true,
    });
    drop(engine);
    assert_eq!(policy.config(), &config);
    assert!(!output.is_empty());
}
