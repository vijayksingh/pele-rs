//! Public detached-render lifecycle with real Sankey helpers and captured providers.
use pele::compat::{
    diagrams::sankey::{Sankey, SankeyModel, SankeyPublicState},
    public::*,
    text::MetricsResources,
    *,
};
use std::{cell::RefCell, rc::Rc};

const SOURCE: &str = "sankey\nA,B,5\nB,C,3\nA,C,1";
fn parsed() -> PublicDocument {
    PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap()
}
fn request() -> DetachedRenderRequest {
    DetachedRenderRequest::Normal {
        options: RenderOptions::default(),
        limit: None,
        enforce_native_bytes: true,
    }
}
fn ref_id(atom: &Atom) -> u32 {
    if let Atom::Ref(id) = atom {
        *id
    } else {
        panic!("expected object")
    }
}
fn field(graph: &ModelGraph, receiver: &Atom, key: &str) -> Atom {
    if let Atom::Ref(id) = receiver {
        let node = &graph.nodes[*id as usize];
        if let Some(property) = node
            .properties()
            .iter()
            .find(|p| graph.strings[p.key as usize].is(key))
        {
            return property.value.clone();
        }
        if key == "length" {
            if let Node::Array { length, .. } = node {
                return Atom::Number(f64::from(*length));
            }
        }
    }
    Atom::Undefined
}
struct Read(Rc<RefCell<ModelGraph>>);
impl LiveRenderRead for Read {
    type Value = Atom;
    fn root(&self) -> Atom {
        self.0.borrow().root.clone()
    }
    fn get(&mut self, value: &Atom, key: &str) -> Result<Atom, PublicError> {
        Ok(field(&self.0.borrow(), value, key))
    }
    fn index(&mut self, value: &Atom, index: usize) -> Result<Atom, PublicError> {
        self.get(value, &index.to_string())
    }
    fn kind(&self, value: &Atom) -> LiveValueKind {
        match value {
            Atom::Undefined => LiveValueKind::Undefined,
            Atom::Null => LiveValueKind::Null,
            Atom::Bool(_) => LiveValueKind::Boolean,
            Atom::Number(_) => LiveValueKind::Number,
            Atom::String(_) => LiveValueKind::String,
            Atom::BuiltinFunction(_) => LiveValueKind::Function,
            Atom::Ref(_) | Atom::HostReference(_) => LiveValueKind::Object,
        }
    }
    fn number(&self, value: &Atom) -> Option<f64> {
        if let Atom::Number(n) = value {
            Some(*n)
        } else {
            None
        }
    }
    fn truthy(&self, value: &Atom) -> bool {
        match value {
            Atom::Undefined | Atom::Null => false,
            Atom::Bool(v) => *v,
            Atom::Number(v) => *v != 0. && !v.is_nan(),
            Atom::String(id) => !self.0.borrow().strings[*id as usize].0.is_empty(),
            _ => true,
        }
    }
    fn string_units(&self, value: &Atom) -> Result<Option<JsString>, PublicError> {
        Ok(if let Atom::String(id) = value {
            Some(self.0.borrow().strings[*id as usize].clone())
        } else {
            None
        })
    }
    fn to_string(&mut self, value: &Atom) -> Result<JsString, PublicError> {
        if let Some(text) = self.string_units(value)? {
            return Ok(text);
        }
        let text = match value {
            Atom::Undefined => "undefined".into(),
            Atom::Null => "null".into(),
            Atom::Bool(v) => v.to_string(),
            Atom::Number(v) => v.to_string(),
            _ => "[object Object]".into(),
        };
        Ok(JsString::from(text.as_str()))
    }
    fn require_string(&mut self, value: &Atom, _: LiveStringUse) -> Result<JsString, PublicError> {
        self.string_units(value)?.ok_or_else(|| {
            PeleError::new("Expected string fixture value", ErrorCode::Semantic).into()
        })
    }
}
struct NestedHelper<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    argument: u32,
    partial: bool,
    fired: bool,
    result: Option<Atom>,
}
impl RenderResources for NestedHelper<'_> {
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
        if !self.fired {
            self.fired = true;
            let receiver = self.graph.borrow().root.clone();
            let outcome = self
                .doc
                .invoke(
                    self.doc.revision(),
                    MethodCall {
                        method: MethodId(1),
                        receiver,
                        arguments: vec![Atom::String(self.argument)],
                    },
                    &mut MetricsResources,
                )
                .unwrap();
            assert_eq!(outcome.result.is_err(), self.partial);
            self.result = outcome.result.ok();
            *self.graph.borrow_mut() = outcome.graph;
        }
        MetricsResources.text_width(request)
    }
}

#[test]
fn detached_render_allows_nested_helpers_and_keeps_partial_lookup_aliases() {
    for partial in [false, true] {
        let mut document = parsed();
        let mut graph = document.snapshot().unwrap();
        let argument = graph.strings.len() as u32;
        graph.strings.push(JsString::from("X"));
        let nodes = ref_id(&field(&graph, &graph.root, "nodes"));
        if partial {
            let key = graph.strings.len() as u32;
            graph.strings.push(JsString::from("length"));
            let Node::Array { length, properties } = &mut graph.nodes[nodes as usize] else {
                panic!()
            };
            properties.push(Property {
                key,
                value: Atom::Number(f64::from(*length)),
                flags: 0,
            });
        }
        document.apply(0, graph).unwrap();
        let expected = document
            .render(&Default::default(), &mut MetricsResources)
            .unwrap();
        let graph = Rc::new(RefCell::new(document.snapshot().unwrap()));
        let job = document
            .prepare_detached_render(1, request())
            .unwrap()
            .unwrap();
        assert!(document.has_active_detached_render());
        let mut reader = Read(graph.clone());
        let mut resources = NestedHelper {
            doc: &mut document,
            graph: graph.clone(),
            argument,
            partial,
            fired: false,
            result: None,
        };
        let completion = job.run(&mut reader, &mut resources);
        assert!(resources.fired);
        let helper_result = resources.result.take();
        drop(resources);
        assert_eq!(document.revision(), 2);
        {
            let outcome = document.finish_detached_render(completion).unwrap();
            assert_eq!(outcome.revision, 3);
            let DetachedRenderValue::Normal(actual) = outcome.result.unwrap() else {
                panic!()
            };
            assert_eq!(actual, expected);
        }
        assert!(!document.has_active_detached_render());
        let graph = document.snapshot().unwrap();
        let lookup = ref_id(&field(&graph, &graph.root, "lookup"));
        let Node::Map { entries, .. } = &graph.nodes[lookup as usize] else {
            panic!()
        };
        let alias = entries
            .iter()
            .find_map(|(key, value)| match key {
                Atom::String(id) if graph.strings[*id as usize].is("X") => Some(ref_id(value)),
                _ => None,
            })
            .unwrap();
        let Node::Array { length, properties } = &graph.nodes[nodes as usize] else {
            panic!()
        };
        assert_eq!(*length, if partial { 3 } else { 4 });
        assert_eq!(
            properties
                .iter()
                .any(|p| matches!(p.value, Atom::Ref(id) if id == alias)),
            !partial
        );
        if let Some(value) = helper_result {
            assert_eq!(ref_id(&value), alias);
        }
        let retry = document
            .invoke(
                3,
                MethodCall {
                    method: MethodId(1),
                    receiver: graph.root,
                    arguments: vec![Atom::String(argument)],
                },
                &mut MetricsResources,
            )
            .unwrap();
        assert_eq!(ref_id(&retry.result.unwrap()), alias);
        assert_eq!(document.revision(), 4);
    }
}

#[test]
fn detached_frames_reject_foreign_and_out_of_order_finish_and_release_dropped_jobs() {
    let mut document = parsed();
    let graph = Rc::new(RefCell::new(document.snapshot().unwrap()));
    let mut reader = Read(graph);
    let outer = document
        .prepare_detached_render(0, request())
        .unwrap()
        .unwrap();
    let inner = document
        .prepare_detached_render(0, request())
        .unwrap()
        .unwrap();
    let wrong_order = outer.run(&mut reader, &mut MetricsResources);
    assert_eq!(
        document
            .finish_detached_render(wrong_order)
            .err()
            .unwrap()
            .message,
        "Public render jobs must finish in nesting order"
    );
    let inner = inner.run(&mut reader, &mut MetricsResources);
    assert_eq!(document.finish_detached_render(inner).unwrap().revision, 1);
    assert!(!document.has_active_detached_render());
    let abandoned = document
        .prepare_detached_render(1, request())
        .unwrap()
        .unwrap();
    drop(abandoned);
    assert_eq!(document.revision(), 1);
    assert!(!document.has_active_detached_render());
    let completion = document
        .prepare_detached_render(1, request())
        .unwrap()
        .unwrap()
        .run(&mut reader, &mut MetricsResources);
    let mut other = parsed();
    // Give the other document its own live owner marker, so owner identity rather
    // than absence alone rejects the foreign completion.
    let other_job = other
        .prepare_detached_render(0, request())
        .unwrap()
        .unwrap();
    assert_eq!(
        other
            .finish_detached_render(completion)
            .err()
            .unwrap()
            .message,
        "Public render job belongs to another document"
    );
    assert_eq!(other.revision(), 0);
    assert_eq!(document.revision(), 1);
    assert!(!document.has_active_detached_render());
    drop(other_job);
    assert!(!other.has_active_detached_render());
    // Explicitly raw settings use the same low-level orchestration.
    let expected = document
        .render_raw_operation_read(
            1,
            &Object::default(),
            &RenderSettings::default(),
            &mut MetricsResources,
        )
        .unwrap()
        .result
        .unwrap();
    let raw = document
        .prepare_detached_render(
            2,
            DetachedRenderRequest::Raw {
                config: Object::default(),
                settings: RenderSettings::default(),
            },
        )
        .unwrap()
        .unwrap()
        .run(&mut reader, &mut MetricsResources);
    let outcome = document.finish_detached_render(raw).unwrap();
    let DetachedRenderValue::Raw(actual) = outcome.result.unwrap() else {
        panic!()
    };
    assert_eq!(actual, expected);
    assert_eq!(outcome.revision, 3);
}

struct Forwarding;
impl DiagramModule for Forwarding {
    type Model = SankeyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Sankey.descriptor()
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<SankeyModel, PeleError> {
        Sankey.parse(context)
    }
    fn render(
        &self,
        model: &SankeyModel,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render(model, config, settings, host)
    }
}
impl PublicDiagramModule for Forwarding {
    type PublicState = SankeyPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Sankey.model_schema()
    }
    fn create_public(&self, model: &SankeyModel) -> Result<SankeyPublicState, PeleError> {
        Sankey.create_public(model)
    }
    fn export_public(&self, state: &SankeyPublicState) -> Result<ModelGraph, PeleError> {
        Sankey.export_public(state)
    }
    fn apply_public(
        &self,
        state: &mut SankeyPublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        Sankey.apply_public(state, graph)
    }
    fn render_public(
        &self,
        state: &mut SankeyPublicState,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render_public(state, config, settings, host)
    }
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Sankey.detached_public_render_capability()
    }
    fn detached_public_render(&self, state: &SankeyPublicState) -> Option<DetachedPublicRender> {
        Sankey.detached_public_render(state)
    }
}
#[test]
fn captured_creator_rejects_forwarded_detached_capability_without_changing_provider_render() {
    let config = Object::default();
    let model = Sankey
        .parse(ParseContext {
            source: SOURCE,
            config: &config,
            title: None,
            line_offset: 0,
            max_edges: 5000,
        })
        .unwrap();
    let mut direct = PublicDocument::from_module(Sankey, &model, config.clone()).unwrap();
    let mut forwarded = PublicDocument::from_module(Forwarding, &model, config).unwrap();
    assert!(direct.has_detached_public_render());
    assert!(!forwarded.has_detached_public_render());
    assert!(forwarded
        .prepare_detached_render(0, request())
        .unwrap()
        .is_none());
    assert_eq!(forwarded.revision(), 0);
    assert_eq!(
        direct
            .render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        forwarded
            .render(&Default::default(), &mut MetricsResources)
            .unwrap()
    );
}
