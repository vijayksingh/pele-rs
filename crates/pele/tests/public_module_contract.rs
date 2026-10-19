//! Downstream opt-in modules use the public document interface without builtin coupling.
use pele::compat::{
    public::{
        Atom, GraphWriter, ModelGraph, Node, PublicDiagramModule, PublicDocument, PublicEngine,
    },
    *,
};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Serialize)]
struct ExtensionModel {
    label: String,
    source: String,
    max_edges: usize,
    line_offset: usize,
}
#[derive(Default)]
struct Calls {
    descriptors: AtomicUsize,
    parses: AtomicUsize,
    applies: AtomicUsize,
    drops: AtomicUsize,
}
struct Extension {
    generation: &'static str,
    calls: Arc<Calls>,
}
impl Drop for Extension {
    fn drop(&mut self) {
        self.calls.drops.fetch_add(1, Ordering::Relaxed);
    }
}
impl DiagramModule for Extension {
    type Model = ExtensionModel;
    fn descriptor(&self) -> ModuleDescriptor {
        // A second metadata read deliberately disagrees with the first.
        let first = self.calls.descriptors.fetch_add(1, Ordering::Relaxed) == 0;
        ModuleDescriptor {
            id: DiagramId::extension(if first {
                "workflow"
            } else {
                "wrong-second-read"
            }),
            section: Some(if first { "consumer" } else { "other" }.into()),
            keep_comments: first,
        }
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<ExtensionModel, PeleError> {
        self.calls.parses.fetch_add(1, Ordering::Relaxed);
        Ok(ExtensionModel {
            label: c.title.unwrap_or(c.source.trim()).into(),
            source: c.source.into(),
            max_edges: c.max_edges,
            line_offset: c.line_offset,
        })
    }
    fn render(
        &self,
        model: &ExtensionModel,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        let width = host.text_width(TextRequest {
            text: &model.label,
            size: options.font_size.unwrap_or(16.),
            style: 1,
            font_family: options.font_family.as_deref(),
            font_family_mono: options.font_family_mono.as_deref(),
        })?;
        let tag = match config
            .get("consumer")
            .and_then(Value::as_object)
            .and_then(|c| c.get("tag"))
        {
            Some(Value::String(tag)) => tag.as_str(),
            _ => "missing",
        };
        Ok(Rendered {
            svg: format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\"><text>{}:{}:{}:{tag}</text></svg>",
                self.generation,
                model.label,
                options.responsive.unwrap()
            ),
            width,
            height: 20.,
            links: vec![],
        })
    }
}
impl PublicDiagramModule for Extension {
    type PublicState = ModelGraph;
    fn create_public(&self, model: &ExtensionModel) -> Result<ModelGraph, PeleError> {
        let mut out = GraphWriter::default();
        let label = out.string(model.label.as_str());
        let source = out.string(model.source.as_str());
        let fields = vec![
            out.property("label", Atom::String(label)),
            out.property("source", Atom::String(source)),
            out.property("maxEdges", Atom::Number(model.max_edges as f64)),
            out.property("lineOffset", Atom::Number(model.line_offset as f64)),
        ];
        let root = out.node(Node::Object { properties: fields });
        out.finish(Atom::Ref(root))
    }
    fn export_public(&self, state: &ModelGraph) -> Result<ModelGraph, PeleError> {
        Ok(state.clone())
    }
    fn apply_public(&self, state: &mut ModelGraph, graph: ModelGraph) -> Result<(), PeleError> {
        self.calls.applies.fetch_add(1, Ordering::Relaxed);
        *state = graph;
        Ok(())
    }
    fn render_public(
        &self,
        state: &mut ModelGraph,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        let label = state
            .field("label")?
            .ok_or_else(|| PeleError::new("Missing extension label", ErrorCode::Semantic))?;
        let label = state
            .text(label)?
            .well_formed()
            .ok_or_else(|| PeleError::new("Invalid extension label", ErrorCode::Semantic))?;
        self.render(
            &ExtensionModel {
                label,
                source: String::new(),
                max_edges: 0,
                line_offset: 0,
            },
            config,
            options,
            host,
        )
    }
}
struct Host {
    fail: bool,
    calls: usize,
}
impl RenderResources for Host {
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
        self.calls += 1;
        if self.fail {
            return Err(PeleError::new(
                "host measurement failed",
                ErrorCode::Semantic,
            ));
        }
        Ok(request.text.encode_utf16().count() as f64 * request.size)
    }
}
fn label(graph: &ModelGraph) -> String {
    graph
        .text(graph.field("label").unwrap().unwrap())
        .unwrap()
        .well_formed()
        .unwrap()
}
fn edit_label(graph: &mut ModelGraph, text: &str) {
    let id = graph.strings.len() as u32;
    graph.strings.push(text.into());
    let Atom::Ref(root) = graph.root else {
        panic!("object root")
    };
    let Node::Object { properties } = &mut graph.nodes[root as usize] else {
        panic!("object root")
    };
    let field = properties
        .iter_mut()
        .find(|p| graph.strings[p.key as usize].is("label"))
        .unwrap();
    field.value = Atom::String(id);
}
fn config(tag: &str) -> Object {
    let mut section = Object::default();
    section.insert("useMaxWidth".into(), Value::Bool(false));
    section.insert("tag".into(), Value::String(tag.into()));
    let mut config = Object::default();
    config.insert("consumer".into(), Value::Object(section));
    config
}
#[test]
fn extension_edits_keep_handler_config_and_resources_after_registry_replacement() {
    let first = Arc::new(Calls::default());
    let mut engine = PublicEngine::empty();
    engine.register(Extension {
        generation: "original",
        calls: first.clone(),
    });
    assert_eq!(first.descriptors.load(Ordering::Relaxed), 1);
    let id = DiagramId::extension("workflow");
    let absent = engine
        .parse_as(
            &DiagramId::extension("absent"),
            "workflow",
            &ParseOptions::default(),
        )
        .err()
        .unwrap();
    assert_eq!(absent.code, ErrorCode::UnsupportedDiagram);
    assert_eq!(absent.diagram_type.as_deref(), Some("absent"));
    let mut bounded = ParseOptions::default();
    bounded.preparation.limit = 1;
    assert_eq!(
        engine
            .parse_as(&id, "workflow", &bounded)
            .err()
            .unwrap()
            .code,
        ErrorCode::Limit
    );
    assert_eq!(first.parses.load(Ordering::Relaxed), 0);
    let mut parse_options = ParseOptions::default();
    parse_options.max_edges = 19;
    let source="---\ntitle: Before\nconfig: {consumer: {useMaxWidth: false, tag: retained}}\n---\nworkflow\n%% keep this\n#35;";
    assert_eq!(
        engine.parse(source, &parse_options).err().unwrap().code,
        ErrorCode::UnsupportedDiagram
    );
    let mut document = engine.parse_as(&id, source, &parse_options).unwrap();
    assert!(!document.unchanged_public_graph_is_noop());
    assert!(matches!(
        document.snapshot_view().unwrap(),
        std::borrow::Cow::Owned(_)
    ));
    assert_eq!(first.parses.load(Ordering::Relaxed), 1);
    assert_eq!(document.diagram_type(), &id);
    let mut graph = document.snapshot().unwrap();
    assert_eq!(label(&graph), "Before");
    assert!(graph
        .text(graph.field("source").unwrap().unwrap())
        .unwrap()
        .well_formed()
        .unwrap()
        .contains("%% keep this"));
    assert!(matches!(graph.field("maxEdges").unwrap(),Some(Atom::Number(n))if *n==19.));
    assert!(matches!(graph.field("lineOffset").unwrap(),Some(Atom::Number(n))if *n==4.));
    edit_label(&mut graph, "After");
    document.apply(0, graph.clone()).unwrap();
    assert_eq!(document.revision(), 1);
    let saved = serde_json::to_value(document.snapshot().unwrap()).unwrap();
    edit_label(&mut graph, "Stale edit");
    assert_eq!(
        document.apply(0, graph).unwrap_err().message,
        "Stale public model revision"
    );
    assert_eq!(
        serde_json::to_value(document.snapshot().unwrap()).unwrap(),
        saved
    );
    assert_eq!(document.revision(), 1);
    assert_eq!(first.applies.load(Ordering::Relaxed), 1);
    let mut invalid = document.snapshot().unwrap();
    invalid.root = Atom::Ref(u32::MAX);
    assert_eq!(
        document.apply(1, invalid).unwrap_err().code,
        ErrorCode::Semantic
    );
    assert_eq!(
        serde_json::to_value(document.snapshot().unwrap()).unwrap(),
        saved
    );
    assert_eq!(document.revision(), 1);
    assert_eq!(first.applies.load(Ordering::Relaxed), 1);
    let mut options = RenderOptions::default();
    options.parse.preparation.limit = 0;
    options.parse.preparation.config = config("ignored-at-render");
    let mut host = Host {
        fail: true,
        calls: 0,
    };
    assert_eq!(
        document.render(&options, &mut host).unwrap_err().message,
        "host measurement failed"
    );
    assert_eq!(
        serde_json::to_value(document.snapshot().unwrap()).unwrap(),
        saved
    );
    host.fail = false;
    let expected = document.render(&options, &mut host).unwrap();
    assert!(expected
        .rendered
        .svg
        .contains("original:After:false:retained"));
    assert_eq!(expected.rendered.width, 80.);
    options.render.output_limit = Some(1);
    assert_eq!(
        document.render(&options, &mut host).unwrap_err().code,
        ErrorCode::Limit
    );
    options.render.output_limit = None;
    assert_eq!(document.render(&options, &mut host).unwrap(), expected);
    let replacement = Arc::new(Calls::default());
    engine.register(Extension {
        generation: "replacement",
        calls: replacement.clone(),
    });
    let mut later = engine
        .parse_as(&id, "workflow Later", &ParseOptions::default())
        .unwrap();
    assert_eq!(replacement.descriptors.load(Ordering::Relaxed), 1);
    assert!(later
        .render(&RenderOptions::default(), &mut host)
        .unwrap()
        .rendered
        .svg
        .contains("replacement:"));
    drop(engine);
    assert_eq!(first.drops.load(Ordering::Relaxed), 0);
    assert_eq!(replacement.drops.load(Ordering::Relaxed), 0);
    assert_eq!(document.render(&options, &mut host).unwrap(), expected);
    assert_eq!(first.descriptors.load(Ordering::Relaxed), 1);
    drop(document);
    drop(later);
    assert_eq!(first.drops.load(Ordering::Relaxed), 1);
    assert_eq!(replacement.drops.load(Ordering::Relaxed), 1);
}
#[test]
fn typed_factory_converts_existing_models_without_reparsing_and_retains_module() {
    let calls = Arc::new(Calls::default());
    let model = ExtensionModel {
        label: "Constructed".into(),
        source: "owned typed source".into(),
        max_edges: 17,
        line_offset: 3,
    };
    let mut document = PublicDocument::from_module(
        Extension {
            generation: "factory",
            calls: calls.clone(),
        },
        &model,
        config("factory-config"),
    )
    .unwrap();
    drop(model);
    assert_eq!(calls.descriptors.load(Ordering::Relaxed), 1);
    assert_eq!(calls.parses.load(Ordering::Relaxed), 0);
    let mut host = Host {
        fail: false,
        calls: 0,
    };
    let mut options = RenderOptions::default();
    options.parse.preparation.limit = 0;
    assert!(document
        .render(&options, &mut host)
        .unwrap()
        .rendered
        .svg
        .contains("factory:Constructed:false:factory-config"));
    assert_eq!(document.diagram_type(), &DiagramId::extension("workflow"));
    drop(document);
    assert_eq!(calls.drops.load(Ordering::Relaxed), 1);
}
#[test]
fn typed_factory_adapts_an_existing_builtin_pie_document() {
    use pele::compat::{
        diagrams::pie::{Pie, PieModel},
        text::MetricsResources,
    };
    let engine = Engine::with_builtins();
    let options = RenderOptions::default();
    let parsed = engine
        .parse("pie title Existing\n\"A\":1\n\"B\":2", &options.parse)
        .unwrap();
    let expected = engine
        .render_document(&parsed, &options, &mut MetricsResources)
        .unwrap();
    let mut public = PublicDocument::from_module(
        Pie,
        parsed.model::<PieModel>().unwrap(),
        parsed.config().clone(),
    )
    .unwrap();
    drop(parsed);
    drop(engine);
    assert_eq!(
        public.render(&options, &mut MetricsResources).unwrap(),
        expected
    );
    let mut graph = public.snapshot().unwrap();
    let Atom::Ref(id) = graph.field("sections").unwrap().unwrap() else {
        panic!("sections Map")
    };
    let id = *id;
    let Node::Map { entries, .. } = &mut graph.nodes[id as usize] else {
        panic!("sections Map")
    };
    entries[0].1 = Atom::Number(5.);
    public.apply(0, graph).unwrap();
    let modified = public.render(&options, &mut MetricsResources).unwrap();
    assert_ne!(modified, expected);
    assert_eq!(
        modified,
        Engine::with_builtins()
            .render_source(
                "pie title Existing\n\"A\":5\n\"B\":2",
                &options,
                &mut MetricsResources
            )
            .unwrap()
    );
}
