//! Public ownership boundary: equivalent publication, errors, revisions and resources.
use pele::compat::{public::*, *};
use serde::Serialize;
use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

#[derive(Serialize)]
struct Model;
struct Feature {
    borrowed: bool,
    mode: &'static str,
    events: Arc<Mutex<Vec<&'static str>>>,
}
impl DiagramModule for Feature {
    type Model = Model;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: DiagramId::extension("outcome-view"),
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
        Ok(drawing())
    }
}
fn drawing() -> Rendered {
    Rendered {
        svg: "<svg data-text=\"漢😀&quot;\"/>".into(),
        width: f64::from_bits(0x7ff8000000000042),
        height: -0.,
        links: vec![],
    }
}
impl PublicDiagramModule for Feature {
    type PublicState = ModelGraph;
    fn create_public(&self, _: &Model) -> Result<ModelGraph, PeleError> {
        let mut writer = GraphWriter::default();
        let field = writer.property("calls", Atom::Number(0.));
        let root = writer.node(Node::Object {
            properties: vec![field],
        });
        writer.finish(Atom::Ref(root))
    }
    fn export_public(&self, state: &ModelGraph) -> Result<ModelGraph, PeleError> {
        self.events.lock().unwrap().push("export");
        Ok(state.clone())
    }
    fn borrow_public_graph<'a>(&self, state: &'a ModelGraph) -> Option<&'a ModelGraph> {
        if self.borrowed {
            self.events.lock().unwrap().push("borrow");
            Some(state)
        } else {
            None
        }
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
        Ok(drawing())
    }
    fn render_public_operation(
        &self,
        state: &mut ModelGraph,
        _: &Object,
        _: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.events.lock().unwrap().push("render");
        let Atom::Ref(root) = state.root else {
            panic!("root")
        };
        if let Atom::Number(n) = &mut state.nodes[root as usize].properties_mut()[0].value {
            *n += 1.;
        }
        match self.mode {
            "invalid" => {
                state.nodes[root as usize].properties_mut()[0].value = Atom::Ref(u32::MAX);
                Ok(drawing())
            }
            "throw" => Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString(vec![0xd800, 65]),
            }),
            "host" => {
                host.text_width(TextRequest {
                    text: "callback",
                    size: 16.,
                    style: 0,
                    font_family: None,
                    font_family_mono: None,
                })?;
                Ok(drawing())
            }
            _ => Ok(drawing()),
        }
    }
}
struct Host {
    events: Arc<Mutex<Vec<&'static str>>>,
    fail: bool,
}
impl RenderResources for Host {
    fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
        self.events.lock().unwrap().push("width");
        if self.fail {
            Err(PeleError::new("original callback", ErrorCode::Semantic))
        } else {
            Ok(10.)
        }
    }
    fn icon(&mut self, _: &str) -> Result<Option<TrustedSvg>, PeleError> {
        Ok(None)
    }
    fn presentation_budget(&self) -> Option<PresentationBudget> {
        Some(PresentationBudget::default())
    }
    fn capture_presentation(&mut self, _: PresentationOutcome) {
        self.events.lock().unwrap().push("publish");
    }
}
fn document(borrowed: bool, mode: &'static str) -> (PublicDocument, Host) {
    let events = Arc::new(Mutex::new(vec![]));
    let mut engine = PublicEngine::empty();
    engine.register(Feature {
        borrowed,
        mode,
        events: events.clone(),
    });
    let document = engine
        .parse_as(
            &DiagramId::extension("outcome-view"),
            "source",
            &ParseOptions::default(),
        )
        .unwrap();
    events.lock().unwrap().clear();
    (
        document,
        Host {
            events,
            fail: mode == "host",
        },
    )
}
fn result(value: Result<Rendered, PublicError>) -> serde_json::Value {
    match value {
        Ok(value) => {
            serde_json::json!({"svg":value.svg,"width":value.width.to_bits(),"height":value.height.to_bits(),"links":value.links})
        }
        Err(error) => serde_json::to_value(error).unwrap(),
    }
}
fn frame(revision: u64, graph: &ModelGraph) -> Vec<u8> {
    encode_binary_exchange(revision, graph, b"", DEFAULT_BINARY_BUDGET).unwrap()
}

#[test]
fn owned_and_borrowed_outcomes_preserve_publication_and_failure_order() {
    for borrowed in [false, true] {
        for mode in ["ok", "throw", "host", "invalid"] {
            for limited in [false, true] {
                let (mut owned, mut a) = document(borrowed, mode);
                let (mut viewed, mut b) = document(borrowed, mode);
                let repair = owned.snapshot().unwrap();
                a.events.lock().unwrap().clear();
                let limit = limited.then_some(OutputLimit::Number(1.));
                let old =
                    owned.render_operation_bounded(0, &RenderOptions::default(), limit, &mut a);
                let new = viewed.render_operation_bounded_view(
                    0,
                    &RenderOptions::default(),
                    limit,
                    &mut b,
                );
                match (old, new) {
                    (Ok(old), Ok(new)) => {
                        assert_eq!(old.revision, new.revision);
                        assert!(matches!(&new.graph, Cow::Borrowed(_)) == borrowed);
                        assert_eq!(
                            frame(old.revision, &old.graph),
                            frame(new.revision, &new.graph)
                        );
                        assert_eq!(
                            result(old.result.map(|r| r.rendered)),
                            result(new.result.map(|r| r.rendered))
                        );
                    }
                    (Err(old), Err(new)) => assert_eq!(
                        serde_json::to_value(old).unwrap(),
                        serde_json::to_value(new).unwrap()
                    ),
                    _ => panic!("ownership changed outcome: {borrowed}/{mode}/{limited}"),
                }
                assert_eq!(*a.events.lock().unwrap(), *b.events.lock().unwrap());
                let events = a.events.lock().unwrap();
                if let Some(publish) = events.iter().position(|e| *e == "publish") {
                    assert_eq!(publish, events.len() - 1);
                    assert!(matches!(events[publish - 1], "borrow" | "export"));
                }
                drop(events);
                assert_eq!(owned.revision(), viewed.revision());
                assert_eq!(viewed.revision(), 1);
                let before = b.events.lock().unwrap().len();
                assert!(viewed
                    .render_operation_view(0, &RenderOptions::default(), &mut b)
                    .is_err());
                assert_eq!(before, b.events.lock().unwrap().len());
                if mode == "invalid" {
                    assert!(!b.events.lock().unwrap().contains(&"publish"));
                    assert!(viewed.snapshot().is_err());
                    viewed.apply(1, repair).unwrap();
                    assert_eq!(viewed.revision(), 2);
                    assert!(viewed.snapshot().is_ok());
                }
            }
        }
    }
}

#[test]
fn raw_owned_and_viewed_outcomes_keep_exact_bits_and_retained_errors() {
    for borrowed in [false, true] {
        for mode in ["ok", "throw", "host"] {
            let (mut owned, mut a) = document(borrowed, mode);
            let (mut viewed, mut b) = document(borrowed, mode);
            let old = owned
                .render_raw_operation(0, &Object::default(), &RenderSettings::default(), &mut a)
                .unwrap();
            let new = viewed
                .render_raw_operation_view(
                    0,
                    &Object::default(),
                    &RenderSettings::default(),
                    &mut b,
                )
                .unwrap();
            assert!(matches!(&new.graph, Cow::Borrowed(_)) == borrowed);
            assert_eq!(
                frame(old.revision, &old.graph),
                frame(new.revision, &new.graph)
            );
            assert_eq!(result(old.result), result(new.result));
            assert_eq!(*a.events.lock().unwrap(), *b.events.lock().unwrap());
            assert!(!b.events.lock().unwrap().contains(&"publish"));
        }
    }
}
