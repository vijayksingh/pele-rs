//! Public parse/publication integration: all builtin ER sources keep a host
//! owner, with exact legacy tables and errors. The test sink is only an oracle;
//! production host publication never constructs this comparison graph.
#![cfg(feature = "web-host-er")]
use pele::compat::{
    diagrams::er::{Er, ErModel},
    host_er::{InitialWriter, Parsed},
    public::*,
    *,
};
use serde::Deserialize;
use std::{cell::Cell, rc::Rc};

struct Sink {
    graph: GraphWriter,
    aborted: Rc<Cell<usize>>,
}
impl InitialWriter for Sink {
    type Output = Vec<u8>;
    fn string(&mut self, value: &str) -> Result<u32, PeleError> {
        Ok(self.graph.string(value))
    }
    fn node(&mut self, node: Node) -> Result<u32, PeleError> {
        Ok(self.graph.node(node))
    }
    fn finish(self, root: Atom) -> Result<Vec<u8>, PeleError> {
        encoded(&self.graph.finish(root)?)
    }
    fn abort(self) -> Result<(), PeleError> {
        self.aborted.set(self.aborted.get() + 1);
        Ok(())
    }
}
fn encoded(graph: &ModelGraph) -> Result<Vec<u8>, PeleError> {
    encode_binary_exchange(0, graph, br#"{"type":"er"}"#, DEFAULT_BINARY_BUDGET)
}
fn hosted(
    engine: &PublicEngine,
    source: &str,
    options: &ParseOptions,
    limits: ParseLimits,
) -> (Result<Parsed<Vec<u8>>, PeleError>, usize, usize) {
    let created = Cell::new(0);
    let aborted = Rc::new(Cell::new(0));
    let result = engine.parse_host_er_with_limits(source, options, limits, |_| {
        created.set(created.get() + 1);
        Ok(Sink {
            graph: GraphWriter::default(),
            aborted: aborted.clone(),
        })
    });
    (result, created.get(), aborted.get())
}
fn compare(
    engine: &PublicEngine,
    source: &str,
    options: &ParseOptions,
    limits: ParseLimits,
) -> (usize, usize) {
    let expected = engine
        .parse_with_limits(source, options, limits)
        .and_then(|document| encoded(document.snapshot_view()?.as_ref()));
    let (actual, factories, aborts) = hosted(engine, source, options, limits);
    match (actual, expected) {
        (Ok(Parsed::Host { output, .. }), Ok(expected)) => assert_eq!(output, expected),
        (Ok(Parsed::Arena(_)), _) => panic!("builtin ER must retain the host backend"),
        (Err(actual), Err(expected)) => assert_eq!(actual, expected),
        _ => panic!("parse/publication acceptance differs"),
    }
    (factories, aborts)
}
#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    source: String,
    options: ParseOptions,
}
#[test]
fn every_frozen_er_source_preserves_exact_publication_and_errors() {
    let corpus: Corpus =
        serde_json::from_str(include_str!("../../../fixtures/pele/er.json")).unwrap();
    assert_eq!(corpus.cases.len(), 17_437);
    let engine = PublicEngine::with_builtins();
    for case in corpus.cases {
        compare(&engine, &case.source, &case.options, ParseLimits::default());
    }
}
#[test]
fn fallback_uses_fresh_owner_and_preserves_late_syntax_and_limits() {
    let engine = PublicEngine::with_builtins();
    let mut options = ParseOptions::default();
    options.preparation.limit = 1_000_000;
    let large = format!("erDiagram\n{}", "N".repeat(70_000));
    for source in [
        large.as_str(),
        "erDiagram\nA { int x }\nA { string y }",
        "erDiagram\nA\nA[\"alias\"]",
        "erDiagram\nA\nB\nA ||--o{ B : first\nC",
        "erDiagram\nA\nclassDef x fill:red\nclass A x",
        "erDiagram\nsubgraph s\nA\nend\nsubgraph s\nB\nend",
    ] {
        assert_eq!(
            compare(&engine, source, &options, ParseLimits::default()),
            (2, 1),
            "fallback must discard its first owner and publish into a fresh one",
        );
    }
    let malformed = "erDiagram\nA\nstyle A fill:red\nB { int }";
    let (result, factories, aborts) = hosted(&engine, malformed, &options, ParseLimits::default());
    assert_eq!(result.err().unwrap().code, ErrorCode::Syntax);
    assert_eq!((factories, aborts), (1, 1));
    compare(&engine, malformed, &options, ParseLimits::default());
    for source in [large.as_str(), malformed] {
        assert_eq!(
            compare(
                &engine,
                source,
                &options,
                ParseLimits {
                    source: Some(SourceLimit::Null),
                    max_edges: None,
                },
            ),
            (0, 0),
        );
    }
    // Actual legacy GraphWriter string admission, then a later grammar error:
    // neither path may create the second host owner before deciding the error.
    options.preparation.limit = 10_000_000;
    let mut over = format!("erDiagram\n{}", "N".repeat(8_000_001));
    for syntax in [false, true] {
        if syntax {
            over.push_str("\nA { int }");
        }
        let (result, factories, aborts) = hosted(&engine, &over, &options, ParseLimits::default());
        let error = result.err().unwrap();
        assert_eq!(
            error.code,
            if syntax {
                ErrorCode::Syntax
            } else {
                ErrorCode::Limit
            },
        );
        if !syntax {
            assert_eq!(error.message, "Public model graph exceeds work budget");
        }
        assert_eq!((factories, aborts), (1, 1));
        let expected = engine
            .parse_with_limits(&over, &options, ParseLimits::default())
            .err()
            .unwrap();
        assert_eq!(error, expected);
    }
}

struct Provider;
impl DiagramModule for Provider {
    type Model = ErModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Er.descriptor()
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<ErModel, PeleError> {
        Er.parse(context)
    }
    fn render(
        &self,
        model: &ErModel,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Er.render(model, config, options, resources)
    }
}
impl PublicDiagramModule for Provider {
    type PublicState = <Er as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Er.model_schema()
    }
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
#[test]
fn provider_replacement_does_not_select_builtin_host_publication() {
    let source = "---\nconfig:\n  er:\n    nodeSpacing: 71\n---\nerDiagram\nA\nstyle A fill:red";
    let mut engine = PublicEngine::with_builtins();
    let (first, factories, aborts) = hosted(
        &engine,
        source,
        &ParseOptions::default(),
        ParseLimits::default(),
    );
    let Parsed::Host { policy, output } = first.unwrap() else {
        panic!("builtin host")
    };
    assert_eq!((factories, aborts), (2, 1));
    let config = policy.config().clone();
    assert_eq!(
        config
            .get("er")
            .and_then(Value::as_object)
            .and_then(|section| section.get("nodeSpacing")),
        Some(&Value::Number(71.)),
    );
    engine.register(Provider);
    let (replaced, factories, aborts) = hosted(
        &engine,
        source,
        &ParseOptions::default(),
        ParseLimits::default(),
    );
    let Parsed::Arena(document) = replaced.unwrap() else {
        panic!("external provider arena")
    };
    assert_eq!((factories, aborts), (0, 0));
    assert_eq!(output, encoded(&document.snapshot_view().unwrap()).unwrap());
    drop(engine);
    assert_eq!(policy.config(), &config);
}
