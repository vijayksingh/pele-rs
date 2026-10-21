//! Shared builtin class permits a retained raw renderer without changing ownership.
use pele::compat::{
    diagrams::flowchart::{Flowchart, FlowchartModel},
    public::*,
    *,
};
struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
struct Delegate;
impl DiagramModule for Delegate {
    type Model = FlowchartModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Flowchart.descriptor()
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<FlowchartModel, PeleError> {
        Flowchart.parse(c)
    }
    fn render(
        &self,
        m: &FlowchartModel,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Flowchart.render(m, c, o, h)
    }
}
impl PublicDiagramModule for Delegate {
    type PublicState = <Flowchart as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Flowchart.model_schema()
    }
    fn public_constructor(
        &self,
        p: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        Flowchart.public_constructor(p)
    }
    fn create_public(&self, m: &FlowchartModel) -> Result<Self::PublicState, PeleError> {
        Flowchart.create_public(m)
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Flowchart.export_public(s)
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        Flowchart.apply_public(s, g)
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Flowchart.render_public(s, c, o, h)
    }
}

#[test]
fn shared_class_raw_binding_retains_target_and_rejects_forwarded_authority() {
    let mut engine = PublicEngine::with_builtins();
    let flow = engine
        .capture_constructor(&DiagramType::Flowchart.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    let swim = engine
        .capture_constructor(&DiagramType::Swimlane.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    let sankey = engine
        .capture_constructor(&DiagramType::Sankey.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    let mut made = flow.construct().unwrap();
    let mut native_swim = swim.construct().unwrap();
    assert!(swim.accepts_raw_document(&made));
    assert!(!sankey.accepts_raw_document(&made));
    let before = made.revision();
    assert!(sankey
        .render_raw_operation_view(
            &mut made,
            before,
            &Object::default(),
            &RenderSettings::default(),
            &mut Host
        )
        .is_err());
    assert_eq!(made.revision(), before);
    engine.register(Delegate);
    assert!(engine
        .capture_constructor(&DiagramType::Flowchart.into(), PrototypeId(1))
        .unwrap()
        .is_none());
    let delegated = engine
        .parse("flowchart TD\nA-->B", &ParseOptions::default())
        .unwrap();
    assert!(!swim.accepts_raw_document(&delegated));
    drop(delegated);
    drop(engine);
    let expected = native_swim
        .render_raw_operation(
            native_swim.revision(),
            &Object::default(),
            &RenderSettings::default(),
            &mut Host,
        )
        .unwrap()
        .result
        .unwrap();
    let actual = swim
        .render_raw_operation_view(
            &mut made,
            before,
            &Object::default(),
            &RenderSettings::default(),
            &mut Host,
        )
        .unwrap()
        .result
        .unwrap();
    assert_eq!(actual.svg, expected.svg);
    assert_eq!(actual.width.to_bits(), expected.width.to_bits());
    assert_eq!(actual.height.to_bits(), expected.height.to_bits());
    assert_eq!(made.diagram_type(), &DiagramType::Flowchart.into());
    assert!(made.is_constructor_created());
    assert_eq!(made.revision(), before + 1);
    // The original owner's helper registration is still usable after target render.
    let mut graph = made.snapshot().unwrap();
    let title = graph.strings.len() as u32;
    graph.strings.push(JsString::from("kept owner"));
    let root = graph.root.clone();
    made.apply(made.revision(), graph).unwrap();
    let method = made
        .model_schema()
        .methods
        .iter()
        .find(|m| m.name == "setAccTitle")
        .unwrap()
        .id;
    assert!(made
        .invoke(
            made.revision(),
            MethodCall {
                receiver: root,
                method,
                arguments: vec![Atom::String(title)]
            },
            &mut Host
        )
        .unwrap()
        .result
        .is_ok());
}
