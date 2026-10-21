//! Actual pinned Info output through public detached normal/raw rendering.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE: &str =
    "info\ntitle Before\naccTitle: Before accessible\naccDescr: Before description";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-info-live-render.json"
    ))
    .unwrap()
}
fn request(raw: bool) -> DetachedRenderRequest {
    if raw {
        DetachedRenderRequest::Raw {
            config: Object::default(),
            settings: RenderSettings::default(),
        }
    } else {
        DetachedRenderRequest::Normal {
            options: RenderOptions::default(),
            limit: None,
            enforce_native_bytes: true,
        }
    }
}
fn rendered(value: DetachedRenderValue) -> Rendered {
    match value {
        DetachedRenderValue::Normal(v) => v.rendered,
        DetachedRenderValue::Raw(v) => v,
    }
}
fn drawing(value: &Rendered) -> Json {
    json!({"svg":value.svg,"width":format!("{:016x}",value.width.to_bits()),"height":format!("{:016x}",value.height.to_bits())})
}
fn field(graph: &ModelGraph, receiver: &Atom, key: &str) -> Atom {
    if let Atom::Ref(id) = receiver {
        if let Some(p) = graph.nodes[*id as usize]
            .properties()
            .iter()
            .find(|p| graph.strings[p.key as usize].is(key))
        {
            return p.value.clone();
        }
    }
    Atom::Undefined
}
#[derive(Clone)]
enum Value {
    Atom(Atom),
    String(JsString),
}
struct Read {
    graph: Rc<RefCell<ModelGraph>>,
    reads: Rc<RefCell<Vec<Json>>>,
}
impl Read {
    fn own(&self, atom: Atom) -> Value {
        if let Atom::String(id) = atom {
            Value::String(self.graph.borrow().strings[id as usize].clone())
        } else {
            Value::Atom(atom)
        }
    }
}
impl LiveRenderRead for Read {
    type Value = Value;
    fn root(&self) -> Value {
        self.own(self.graph.borrow().root.clone())
    }
    fn get(&mut self, receiver: &Value, key: &str) -> Result<Value, PublicError> {
        self.reads.borrow_mut().push(json!(key));
        Ok(match receiver {
            Value::Atom(atom) => self.own(field(&self.graph.borrow(), atom, key)),
            _ => Value::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, receiver: &Value, index: usize) -> Result<Value, PublicError> {
        self.get(receiver, &index.to_string())
    }
    fn kind(&self, value: &Value) -> LiveValueKind {
        match value {
            Value::String(_) => LiveValueKind::String,
            Value::Atom(Atom::Undefined) => LiveValueKind::Undefined,
            Value::Atom(Atom::Null) => LiveValueKind::Null,
            Value::Atom(Atom::Bool(_)) => LiveValueKind::Boolean,
            Value::Atom(Atom::Number(_)) => LiveValueKind::Number,
            Value::Atom(Atom::BuiltinFunction(_)) => LiveValueKind::Function,
            _ => LiveValueKind::Object,
        }
    }
    fn number(&self, value: &Value) -> Option<f64> {
        if let Value::Atom(Atom::Number(n)) = value {
            Some(*n)
        } else {
            None
        }
    }
    fn truthy(&self, value: &Value) -> bool {
        match value {
            Value::String(s) => !s.0.is_empty(),
            Value::Atom(Atom::Undefined | Atom::Null) => false,
            Value::Atom(Atom::Bool(b)) => *b,
            Value::Atom(Atom::Number(n)) => *n != 0. && !n.is_nan(),
            _ => true,
        }
    }
    fn string_units(&self, value: &Value) -> Result<Option<JsString>, PublicError> {
        Ok(if let Value::String(s) = value {
            Some(s.clone())
        } else {
            None
        })
    }
    fn to_string(&mut self, value: &Value) -> Result<JsString, PublicError> {
        self.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "e",
            },
        )
    }
    fn require_string(&mut self, value: &Value, _: LiveStringUse) -> Result<JsString, PublicError> {
        self.string_units(value)?
            .ok_or_else(|| PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString::from("Info live read requires string"),
            })
    }
}
fn replace_text(graph: &mut ModelGraph, key: &str, value: &str) {
    let id = graph.strings.len() as u32;
    graph.strings.push(JsString::from(value));
    let Atom::Ref(root) = graph.root else {
        panic!("root")
    };
    let key_id = graph.strings.iter().position(|s| s.is(key)).unwrap() as u32;
    graph.nodes[root as usize]
        .properties_mut()
        .iter_mut()
        .find(|p| p.key == key_id)
        .unwrap()
        .value = Atom::String(id);
}
struct Plain;
impl RenderResources for Plain {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
struct Callback<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    reads: Rc<RefCell<Vec<Json>>>,
    mode: &'a str,
    raw: bool,
    fired: bool,
    trace: Vec<Json>,
    nested: Option<Json>,
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.trace
            .push(json!([r.text, format!("{:016x}", r.size.to_bits())]));
        self.reads.borrow_mut().push(json!(["width", r.text]));
        if !self.fired
            && self.mode != "normal"
            && (self.mode != "version-mutate" || r.text == "Pele v1.6.0")
        {
            self.fired = true;
            // Snapshot and apply occur while the outer job owns no document borrow.
            let mut graph = self.doc.snapshot().unwrap();
            for (key, value) in [
                ("title", "After"),
                ("accTitle", "After accessible"),
                ("accDescr", "After description"),
            ] {
                replace_text(&mut graph, key, value);
            }
            self.doc.apply(self.doc.revision(), graph).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            if self.mode == "throw" {
                return Err(PeleError::new(
                    "Info callback sentinel",
                    ErrorCode::Semantic,
                ));
            }
            if self.mode == "nested" {
                let job = self
                    .doc
                    .prepare_detached_render(self.doc.revision(), request(self.raw))
                    .unwrap()
                    .unwrap();
                let mut reader = Read {
                    graph: self.graph.clone(),
                    reads: Rc::new(RefCell::new(vec![])),
                };
                let done = job.run(&mut reader, &mut Plain);
                let outcome = self.doc.finish_detached_render(done).unwrap();
                self.nested = Some(drawing(&rendered(outcome.result.unwrap())));
            }
        }
        Plain.text_width(r)
    }
}
#[test]
fn info_live_callbacks_match_pinned_pele_and_keep_edits_after_errors() {
    let fixture = fixture();
    for raw in [false, true] {
        for case in fixture["cases"].as_array().unwrap() {
            let mode = case["mode"].as_str().unwrap();
            let mut doc = PublicEngine::with_builtins()
                .parse(SOURCE, &Default::default())
                .unwrap();
            let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
            let reads = Rc::new(RefCell::new(vec![]));
            let job = doc
                .prepare_detached_render(doc.revision(), request(raw))
                .unwrap()
                .unwrap();
            let mut reader = Read {
                graph: graph.clone(),
                reads: reads.clone(),
            };
            let mut host = Callback {
                doc: &mut doc,
                graph: graph.clone(),
                reads,
                mode,
                raw,
                fired: false,
                trace: vec![],
                nested: None,
            };
            let done = job.run(&mut reader, &mut host);
            assert_eq!(json!(host.trace), case["trace"], "{mode}/{raw}");
            assert_eq!(host.nested, case.get("nested").cloned());
            drop(host);
            let prior = doc.revision();
            let outcome = doc.finish_detached_render(done).unwrap();
            assert_eq!(outcome.revision, prior + 1);
            if case["result"]["ok"] == true {
                let actual = drawing(&rendered(outcome.result.unwrap()));
                for key in ["svg", "width", "height"] {
                    assert_eq!(actual[key], case["result"][key], "{mode}/{raw}/{key}");
                }
            } else {
                let PublicError::Pele(error) = outcome.result.err().unwrap() else {
                    panic!("error domain")
                };
                assert_eq!(error.message, "Info callback sentinel");
            }
            assert!(!doc.has_active_detached_render());
            let graph_now = doc.snapshot().unwrap();
            for key in ["title", "accTitle", "accDescr"] {
                let value = field(&graph_now, &graph_now.root, key);
                assert_eq!(
                    graph_now.text(&value).unwrap().well_formed().unwrap(),
                    case["after"][key]
                );
            }
            *graph.borrow_mut() = graph_now;
            let recovery = doc
                .prepare_detached_render(doc.revision(), request(raw))
                .unwrap()
                .unwrap();
            let done = recovery.run(&mut reader, &mut Plain);
            let outcome = doc.finish_detached_render(done).unwrap();
            assert_eq!(
                drawing(&rendered(outcome.result.unwrap())),
                case["recovery"]
            );
            // The immutable/typed path shares exactly the same geometry and SVG.
            assert_eq!(
                drawing(
                    &doc.render(&Default::default(), &mut Plain)
                        .unwrap()
                        .rendered
                ),
                case["recovery"]
            );
        }
    }
}
#[test]
fn info_live_phase_preserves_read_order_and_javascript_error_domain() {
    let mut doc = PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap();
    let mut graph = doc.snapshot().unwrap();
    for (key, value) in [
        ("title", "Title"),
        ("accTitle", "accTitle"),
        ("accDescr", "accDescr"),
    ] {
        replace_text(&mut graph, key, value);
    }
    doc.apply(doc.revision(), graph.clone()).unwrap();
    let graph = Rc::new(RefCell::new(graph));
    let reads = Rc::new(RefCell::new(vec![]));
    let mut reader = Read {
        graph: graph.clone(),
        reads: reads.clone(),
    };
    let job = doc
        .prepare_detached_render(doc.revision(), request(true))
        .unwrap()
        .unwrap();
    let mut host = Callback {
        doc: &mut doc,
        graph: graph.clone(),
        reads: reads.clone(),
        mode: "normal",
        raw: true,
        fired: false,
        trace: vec![],
        nested: None,
    };
    let done = job.run(&mut reader, &mut host);
    drop(host);
    assert!(doc.finish_detached_render(done).unwrap().result.is_ok());
    assert_eq!(json!(*reads.borrow()), fixture()["readOrder"]);
    // Live read errors must not be flattened into PeleError by the shared renderer.
    let mut next = doc.snapshot().unwrap();
    let Atom::Ref(root) = next.root else { panic!() };
    let id = next.strings.iter().position(|s| s.is("accTitle")).unwrap() as u32;
    next.nodes[root as usize]
        .properties_mut()
        .iter_mut()
        .find(|p| p.key == id)
        .unwrap()
        .value = Atom::Number(1.);
    doc.apply(doc.revision(), next.clone()).unwrap();
    *graph.borrow_mut() = next;
    let job = doc
        .prepare_detached_render(doc.revision(), request(true))
        .unwrap()
        .unwrap();
    let done = job.run(&mut reader, &mut Plain);
    assert!(matches!(
        doc.finish_detached_render(done).unwrap().result,
        Err(PublicError::JavaScript {
            name: JavaScriptErrorKind::TypeError,
            ..
        })
    ));
}
