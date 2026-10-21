//! Public Pie normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE: &str = "pie\ntitle Initial title\n\"Alpha\" : 1\n\"Beta\" : 2\n\"Gamma\" : 3";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-pie-live-render.json"
    ))
    .unwrap()
}
// JSON has one Number type; serde_json also remembers integer/f64 spelling.
// Compare the represented IEEE values, never round or stringify them. Numeric
// bit strings in the oracle remain exact strings (including NaN payloads).
fn assert_json(actual: &Json, expected: &Json, context: &str) {
    match (actual, expected) {
        (Json::Number(a), Json::Number(b)) => assert_eq!(
            a.as_f64().unwrap().to_bits(),
            b.as_f64().unwrap().to_bits(),
            "{context}: numeric bits differ"
        ),
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{context}: array length");
            for (index, (a, b)) in a.iter().zip(b).enumerate() {
                assert_json(a, b, &format!("{context}/{index}"));
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{context}: object fields"
            );
            for (key, a) in a {
                assert_json(a, &b[key], &format!("{context}/{key}"));
            }
        }
        _ => assert_eq!(actual, expected, "{context}"),
    }
}
fn request(raw: bool, responsive: bool) -> DetachedRenderRequest {
    let settings = RenderSettings {
        max_width: responsive.then_some(200.),
        ..Default::default()
    };
    if raw {
        DetachedRenderRequest::Raw {
            config: Object::default(),
            settings,
        }
    } else {
        DetachedRenderRequest::Normal {
            options: RenderOptions {
                render: settings,
                ..Default::default()
            },
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
    Pair(std::sync::Arc<[Value; 2]>),
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
            Value::Pair(pair) => key
                .parse::<usize>()
                .ok()
                .and_then(|n| pair.get(n))
                .cloned()
                .unwrap_or(Value::Atom(Atom::Undefined)),
            _ => Value::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, receiver: &Value, index: usize) -> Result<Value, PublicError> {
        self.get(receiver, &index.to_string())
    }
    fn snapshot_iterable(
        &mut self,
        value: &Value,
        max_count: usize,
    ) -> Result<Vec<Value>, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("Map")
        };
        let graph = self.graph.borrow();
        let Node::Map { entries, .. } = &graph.nodes[*id as usize] else {
            panic!("Map")
        };
        assert!(entries.len() <= max_count);
        Ok(entries
            .iter()
            .map(|(key, value)| {
                Value::Pair(std::sync::Arc::new([
                    self.own(key.clone()),
                    self.own(value.clone()),
                ]))
            })
            .collect())
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
        match value {
            Value::String(text) => Ok(text.clone()),
            Value::Atom(Atom::Number(n)) => Ok(JsString::from(n.to_string().as_str())),
            _ => panic!("fixture conversion"),
        }
    }
    fn require_string(&mut self, value: &Value, _: LiveStringUse) -> Result<JsString, PublicError> {
        self.string_units(value)?
            .ok_or_else(|| PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString::from("Pie live read requires string"),
            })
    }
}

fn text(graph: &mut ModelGraph, value: &str) -> Atom {
    let id = graph.strings.len() as u32;
    graph.strings.push(JsString::from(value));
    Atom::String(id)
}
fn set(graph: &mut ModelGraph, key: &str, value: Atom) {
    let Atom::Ref(root) = graph.root else {
        panic!()
    };
    let id = graph.strings.iter().position(|s| s.is(key)).unwrap() as u32;
    graph.nodes[root as usize]
        .properties_mut()
        .iter_mut()
        .find(|p| p.key == id)
        .unwrap()
        .value = value;
}
fn set_text(graph: &mut ModelGraph, key: &str, value: &str) {
    let atom = text(graph, value);
    set(graph, key, atom)
}
fn sections(graph: &ModelGraph) -> u32 {
    let Atom::Ref(id) = field(graph, &graph.root, "sections") else {
        panic!()
    };
    id
}
fn entries(graph: &ModelGraph, id: u32, bits: bool) -> Json {
    let Node::Map { entries, .. } = &graph.nodes[id as usize] else {
        panic!()
    };
    Json::Array(
        entries
            .iter()
            .map(|(key, value)| {
                let Atom::Number(n) = value else { panic!() };
                json!([
                    graph.text(key).unwrap().well_formed().unwrap(),
                    if bits {
                        json!(format!("{:016x}", n.to_bits()))
                    } else {
                        json!(n)
                    }
                ])
            })
            .collect(),
    )
}
fn state(graph: &ModelGraph) -> Json {
    let value = |key| match field(graph, &graph.root, key) {
        Atom::String(id) => json!(graph.strings[id as usize].well_formed().unwrap()),
        Atom::Bool(v) => json!(v),
        _ => Json::Null,
    };
    json!({"title":value("title"),"showData":value("showData"),"accTitle":value("accTitle"),"accDescr":value("accDescr"),"sections":entries(graph,sections(graph),true)})
}
#[derive(Default)]
struct Plain {
    trace: Vec<Json>,
}
impl RenderResources for Plain {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.trace.push(json!([
            r.text,
            format!("{:016x}", r.size.to_bits()),
            r.style
        ]));
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
struct Callback<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    fired: bool,
    plain: Plain,
    nested: Option<Json>,
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let width = self.plain.text_width(r)?;
        if !self.fired {
            self.fired = true;
            let mut graph = self.doc.snapshot().unwrap();
            match self.mode {
                "title" => set_text(&mut graph, "title", "Changed title"),
                "showData" => set(&mut graph, "showData", Atom::Bool(true)),
                "accessibility" => {
                    set_text(&mut graph, "accTitle", "Changed access");
                    set_text(&mut graph, "accDescr", "Changed description");
                }
                "map-edit" => {
                    let delta = text(&mut graph, "Delta");
                    let id = sections(&graph);
                    let strings = graph.strings.clone();
                    let Node::Map { entries, .. } = &mut graph.nodes[id as usize] else {
                        panic!()
                    };
                    for (key, value) in entries.iter_mut() {
                        if let Atom::String(id) = key {
                            if strings[*id as usize].is("Beta") {
                                *value = Atom::Number(20.);
                            }
                        }
                    }
                    entries.retain(|(key,_)|!matches!(key,Atom::String(id) if strings[*id as usize].is("Gamma")));
                    entries.push((delta, Atom::Number(4.)));
                }
                "map-replace" => {
                    let key = text(&mut graph, "Replacement");
                    let id = graph.nodes.len() as u32;
                    graph.nodes.push(Node::Map {
                        entries: vec![(key, Atom::Number(8.))],
                        properties: vec![],
                    });
                    set(&mut graph, "sections", Atom::Ref(id));
                }
                "throw-object" | "throw-undefined" => {
                    set_text(&mut graph, "title", "Persisted before throw");
                    set(&mut graph, "showData", Atom::Bool(true));
                    let key = text(&mut graph, "Added");
                    let id = sections(&graph);
                    let Node::Map { entries, .. } = &mut graph.nodes[id as usize] else {
                        panic!()
                    };
                    entries.push((key, Atom::Number(7.)));
                }
                "nested-render" => set_text(&mut graph, "title", "Nested title"),
                "baseline" => {}
                _ => panic!("mode"),
            }
            if self.mode != "baseline" {
                self.doc.apply(self.doc.revision(), graph).unwrap();
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
            if self.mode.starts_with("throw-") {
                return Err(PeleError::new("Pie callback sentinel", ErrorCode::Semantic));
            }
            if self.mode == "nested-render" {
                let job = self
                    .doc
                    .prepare_detached_render(self.doc.revision(), request(true, false))
                    .unwrap()
                    .unwrap();
                let mut read = Read {
                    graph: self.graph.clone(),
                    reads: Rc::new(RefCell::new(vec![])),
                };
                let done = job.run(&mut read, &mut Plain::default());
                let outcome = self.doc.finish_detached_render(done).unwrap();
                self.nested = Some(drawing(&rendered(outcome.result.unwrap())));
                let mut graph = self.doc.snapshot().unwrap();
                set_text(&mut graph, "title", "After nested");
                self.doc.apply(self.doc.revision(), graph).unwrap();
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
        }
        Ok(width)
    }
}
#[test]
fn pie_live_callbacks_match_pinned_pele_and_preserve_edits_and_nested_revisions() {
    for case in fixture()["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{mode}/{raw}/{responsive}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let original = sections(&graph.borrow());
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut read = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
        let mut callback = Callback {
            doc: &mut doc,
            graph: graph.clone(),
            mode,
            fired: false,
            plain: Plain::default(),
            nested: None,
        };
        let done = job.run(&mut read, &mut callback);
        assert_eq!(
            json!(callback.plain.trace),
            case["trace"],
            "{context}/trace"
        );
        assert_eq!(
            callback.nested,
            case.get("nested").cloned(),
            "{context}/nested"
        );
        drop(callback);
        let revision = doc.revision();
        let outcome = doc.finish_detached_render(done).unwrap();
        assert_eq!(outcome.revision, revision + 1);
        if case["first"]["ok"] == true {
            let actual = drawing(&rendered(outcome.result.unwrap()));
            for key in ["svg", "width", "height"] {
                assert_eq!(actual[key], case["first"][key], "{context}/{key}");
            }
        } else {
            let PublicError::Pele(error) = outcome.result.err().unwrap() else {
                panic!("error domain")
            };
            assert_eq!(error.message, "Pie callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_json(&state(&now), &case["after"], &format!("{context}/state"));
        assert_eq!(
            json!(sections(&now) == original),
            case["originalMapStillCurrent"],
            "{context}/alias"
        );
        assert_json(
            &entries(&now, original, false),
            &case["originalMap"],
            &format!("{context}/old map"),
        );
        *graph.borrow_mut() = now;
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut plain = Plain::default();
        let done = job.run(&mut read, &mut plain);
        let outcome = doc.finish_detached_render(done).unwrap();
        assert_eq!(
            drawing(&rendered(outcome.result.unwrap())),
            case["recovery"],
            "{context}/recovery"
        );
        assert_eq!(
            json!(plain.trace),
            case["recoveryTrace"],
            "{context}/recovery trace"
        );
        let options = RenderOptions {
            render: RenderSettings {
                max_width: responsive.then_some(200.),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            drawing(
                &doc.render(&options, &mut Plain::default())
                    .unwrap()
                    .rendered
            ),
            case["recovery"],
            "{context}/typed geometry"
        );
    }
}
