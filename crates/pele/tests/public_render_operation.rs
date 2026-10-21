//! Stateful rendering is an extension contract, including graph publication on errors.
use pele::compat::{public::*, text::MetricsResources, *};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Serialize)]
struct Model;
struct Extension {
    mode: &'static str,
    calls: Arc<AtomicUsize>,
}
impl DiagramModule for Extension {
    type Model = Model;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: DiagramId::extension("stateful-render"),
            section: None,
            keep_comments: false,
        }
    }
    fn parse(&self, _: ParseContext<'_>) -> Result<Model, PeleError> {
        Ok(Model)
    }
    fn render(
        &self,
        _: &Model,
        _: &Object,
        _: &RenderSettings,
        _: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Ok(drawing(4., 5.))
    }
}
fn drawing(width: f64, height: f64) -> Rendered {
    Rendered {
        svg: "<svg xmlns=\"http://www.w3.org/2000/svg\"/>".into(),
        width,
        height,
        links: vec![],
    }
}
impl PublicDiagramModule for Extension {
    type PublicState = ModelGraph;
    fn create_public(&self, _: &Model) -> Result<ModelGraph, PeleError> {
        let mut w = GraphWriter::default();
        let p = w.property("renders", Atom::Number(0.));
        let root = w.node(Node::Object {
            properties: vec![p],
        });
        w.finish(Atom::Ref(root))
    }
    fn export_public(&self, state: &ModelGraph) -> Result<ModelGraph, PeleError> {
        if self.mode == "export-error" {
            return Err(PeleError::new(
                "original export failure",
                ErrorCode::Semantic,
            ));
        }
        Ok(state.clone())
    }
    fn borrow_public_graph<'a>(&self, state: &'a ModelGraph) -> Option<&'a ModelGraph> {
        self.mode.starts_with("borrowed").then_some(state)
    }
    fn apply_public(&self, state: &mut ModelGraph, graph: ModelGraph) -> Result<(), PeleError> {
        *state = graph;
        Ok(())
    }
    fn render_public(
        &self,
        _: &mut ModelGraph,
        _: &Object,
        _: &RenderSettings,
        _: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Ok(drawing(4., 5.))
    }
    fn render_public_operation(
        &self,
        state: &mut ModelGraph,
        _: &Object,
        _: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let Atom::Ref(root) = state.root else {
            unreachable!()
        };
        if let Atom::Number(value) = &mut state.nodes[root as usize].properties_mut()[0].value {
            *value += 1.;
        }
        match self.mode {
            "js" => Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString(vec![0xd800, 65]),
            }),
            "host" => {
                host.text_width(TextRequest {
                    text: "scheduled",
                    size: 16.,
                    style: 0,
                    font_family: None,
                    font_family_mono: None,
                })?;
                Ok(drawing(4., 5.))
            }
            "invalid" | "borrowed-invalid" => {
                state.nodes[root as usize].properties_mut()[0].value = Atom::Ref(u32::MAX);
                Ok(drawing(4., 5.))
            }
            "numbers" => Ok(drawing(f64::from_bits(0x7ff8000000000042), -0.)),
            _ => Ok(drawing(4., 5.)),
        }
    }
}
fn document(mode: &'static str, calls: Arc<AtomicUsize>) -> PublicDocument {
    let mut engine = PublicEngine::empty();
    engine.register(Extension { mode, calls });
    engine
        .parse_as(
            &DiagramId::extension("stateful-render"),
            "source",
            &ParseOptions::default(),
        )
        .unwrap()
}
fn count(graph: &ModelGraph) -> f64 {
    match graph.field("renders").unwrap().unwrap() {
        Atom::Number(n) => *n,
        _ => panic!("counter"),
    }
}
#[test]
fn render_operations_publish_native_mutations_and_retained_handlers() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut doc = document("ok", calls.clone());
    assert_eq!(
        doc.render(&RenderOptions::default(), &mut MetricsResources)
            .unwrap()
            .rendered
            .width,
        4.
    );
    assert_eq!(doc.revision(), 0);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let first = doc
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(first.revision, 1);
    assert_eq!(count(&first.graph), 1.);
    assert!(first.result.is_ok());
    assert_eq!(
        doc.render_operation(0, &RenderOptions::default(), &mut MetricsResources)
            .unwrap_err()
            .message,
        "Stale public model revision"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let mut options = RenderOptions::default();
    options.render.output_limit = Some(1);
    let failed = doc
        .render_operation(1, &options, &mut MetricsResources)
        .unwrap();
    assert_eq!(failed.revision, 2);
    assert_eq!(count(&failed.graph), 2.);
    assert!(matches!(failed.result,Err(PublicError::Pele(ref e)) if e.code==ErrorCode::Limit));
    let recovered = doc
        .render_operation(2, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(count(&recovered.graph), 3.);
    assert!(recovered.result.is_ok());
}
struct Failure;
impl RenderResources for Failure {
    fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
        Err(PeleError::new("original host failure", ErrorCode::Semantic))
    }
    fn icon(&mut self, _: &str) -> Result<Option<TrustedSvg>, PeleError> {
        Ok(None)
    }
}
#[test]
fn render_operation_errors_keep_lossless_categories_graphs_and_callback_edits() {
    let mut js = document("js", Arc::new(AtomicUsize::new(0)));
    let outcome = js
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(count(&outcome.graph), 1.);
    assert!(
        matches!(outcome.result,Err(PublicError::JavaScript{name:JavaScriptErrorKind::TypeError,message:JsString(ref m)}) if m==&[0xd800,65])
    );
    let mut host = document("host", Arc::new(AtomicUsize::new(0)));
    let outcome = host
        .render_operation(0, &RenderOptions::default(), &mut Failure)
        .unwrap();
    assert_eq!(count(&outcome.graph), 1.);
    assert!(
        matches!(outcome.result,Err(PublicError::Pele(ref e)) if e.message=="original host failure")
    );
    assert!(host
        .render_operation(1, &RenderOptions::default(), &mut MetricsResources)
        .unwrap()
        .result
        .is_ok());
}
#[test]
fn nested_render_outcomes_preserve_ieee_dimension_transport_and_protocol_boundaries() {
    let mut doc = document("numbers", Arc::new(AtomicUsize::new(0)));
    let outcome = doc
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    let json = serde_json::to_value(&outcome).unwrap();
    assert_eq!(
        json["result"]["Ok"]["$numbers"]["width"],
        "7ff8000000000042"
    );
    assert_eq!(
        json["result"]["Ok"]["$numbers"]["height"],
        "8000000000000000"
    );
    assert_eq!(json["result"]["Ok"]["width"], serde_json::Value::Null);
    assert_eq!(count(&outcome.graph), 1.);
    let mut bad = document("invalid", Arc::new(AtomicUsize::new(0)));
    assert!(bad
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .is_err());
    assert_eq!(bad.revision(), 1);
    assert!(bad.snapshot().is_err());
}

#[test]
fn borrowed_and_fallback_snapshots_preserve_validation_revisions_and_recovery() {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut doc = document("borrowed-invalid", calls.clone());
    assert!(matches!(
        doc.snapshot_view().unwrap(),
        std::borrow::Cow::Borrowed(_)
    ));
    // Borrowing does not implicitly declare repeated apply to be a no-op.
    assert!(!doc.unchanged_public_graph_is_noop());
    let repair = doc.snapshot().unwrap();
    assert!(doc
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .is_err());
    assert_eq!(doc.revision(), 1);
    assert!(doc.snapshot_view().is_err());
    assert_eq!(
        doc.render_operation(0, &RenderOptions::default(), &mut MetricsResources)
            .unwrap_err()
            .message,
        "Stale public model revision"
    );
    // Invalid borrowed graph rejects before another render dispatch.
    assert!(doc
        .render_operation(1, &RenderOptions::default(), &mut MetricsResources)
        .is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(doc.revision(), 1);
    doc.apply(1, repair).unwrap();
    assert_eq!(doc.revision(), 2);
    assert_eq!(count(&doc.snapshot_view().unwrap()), 0.);

    let fallback_calls = Arc::new(AtomicUsize::new(0));
    let mut fallback = document("export-error", fallback_calls.clone());
    assert_eq!(
        fallback.snapshot_view().unwrap_err().message,
        "original export failure"
    );
    assert_eq!(
        fallback.snapshot().unwrap_err().message,
        "original export failure"
    );
    assert_eq!(
        fallback
            .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
            .unwrap_err()
            .message,
        "original export failure"
    );
    assert_eq!(fallback.revision(), 0);
    assert_eq!(fallback_calls.load(Ordering::SeqCst), 0);
}
