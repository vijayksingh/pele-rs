//! Treemap callback phases through actual public normal/raw jobs.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-treemap-live-render.json"
    ))
    .unwrap()
}
fn request(raw: bool, responsive: bool) -> DetachedRenderRequest {
    let settings = RenderSettings {
        max_width: responsive.then_some(300.),
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
fn rendered(v: DetachedRenderValue) -> Rendered {
    match v {
        DetachedRenderValue::Normal(v) => v.rendered,
        DetachedRenderValue::Raw(v) => v,
    }
}
fn drawing(v: &Rendered) -> Json {
    json!({"svg":v.svg,"width":format!("{:016x}",v.width.to_bits()),"height":format!("{:016x}",v.height.to_bits())})
}
fn field(graph: &ModelGraph, receiver: &Atom, key: &str) -> Atom {
    if let Atom::Ref(id) = receiver {
        if key == "length" {
            if let Node::Array { length, .. } = &graph.nodes[*id as usize] {
                return Atom::Number(f64::from(*length));
            }
        }
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
                message: JsString::from("Treemap live read requires string"),
            })
    }
}

fn id(v: &Atom) -> u32 {
    if let Atom::Ref(id) = v {
        *id
    } else {
        panic!("reference")
    }
}
fn set(g: &mut ModelGraph, node: u32, key: &str, value: Atom) {
    let keyid = if let Some(id) = g.strings.iter().position(|s| s.is(key)) {
        id as u32
    } else {
        let id = g.strings.len() as u32;
        g.strings.push(JsString::from(key));
        id
    };
    // String IDs are history entries; equal property names may use different IDs.
    if let Some(index) = g.nodes[node as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key))
    {
        g.nodes[node as usize].properties_mut()[index].value = value;
    } else {
        g.nodes[node as usize].properties_mut().push(Property {
            key: keyid,
            value,
            flags: 7,
        });
    }
}
fn append(g: &mut ModelGraph, node: u32, value: Atom) {
    let Node::Array { length, .. } = g.nodes[node as usize] else {
        panic!("array")
    };
    set(g, node, &length.to_string(), value);
    if let Node::Array { length, .. } = &mut g.nodes[node as usize] {
        *length += 1;
    }
}
fn text(g: &mut ModelGraph, value: &str) -> Atom {
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(value));
    Atom::String(id)
}
fn set_text(g: &mut ModelGraph, node: u32, key: &str, value: &str) {
    let v = text(g, value);
    set(g, node, key, v);
}
fn input(g: &mut ModelGraph, v: &Json) -> Atom {
    match v {
        Json::Null => Atom::Null,
        Json::Bool(v) => Atom::Bool(*v),
        Json::Number(v) => Atom::Number(v.as_f64().unwrap()),
        Json::String(v) => text(g, v),
        Json::Array(values) => {
            let mut properties = vec![];
            for (i, v) in values.iter().enumerate() {
                let Atom::String(key) = text(g, &i.to_string()) else {
                    panic!()
                };
                properties.push(Property {
                    key,
                    value: input(g, v),
                    flags: 7,
                });
            }
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Array {
                length: values.len() as u32,
                properties,
            });
            Atom::Ref(id)
        }
        Json::Object(values) => {
            let mut properties = vec![];
            for (key, v) in values {
                let Atom::String(key) = text(g, key) else {
                    panic!()
                };
                properties.push(Property {
                    key,
                    value: input(g, v),
                    flags: 7,
                });
            }
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Object { properties });
            Atom::Ref(id)
        }
    }
}
fn output(g: &ModelGraph, v: &Atom) -> Json {
    match v {
        Atom::Undefined | Atom::Null => Json::Null,
        Atom::Bool(v) => json!(v),
        Atom::Number(v) => json!(v),
        Atom::String(id) => json!(g.strings[*id as usize].well_formed().unwrap()),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { length, .. } => (0..*length)
                .map(|i| output(g, &field(g, v, &i.to_string())))
                .collect(),
            node => node
                .properties()
                .iter()
                .filter(|p| !matches!(p.value, Atom::Undefined))
                .map(|p| {
                    (
                        g.strings[p.key as usize].well_formed().unwrap(),
                        output(g, &p.value),
                    )
                })
                .collect(),
        },
        _ => panic!("fixture atom"),
    }
}
fn state(g: &ModelGraph) -> Json {
    ["title", "accTitle", "accDescr", "roots"]
        .into_iter()
        .map(|k| (k.to_string(), output(g, &field(g, &g.root, k))))
        .collect()
}
fn assert_json(a: &Json, b: &Json) {
    match (a, b) {
        (Json::Number(a), Json::Number(b)) => {
            assert_eq!(a.as_f64().unwrap().to_bits(), b.as_f64().unwrap().to_bits())
        }
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                assert_json(a, b);
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
            for (k, a) in a {
                assert_json(a, &b[k]);
            }
        }
        _ => assert_eq!(a, b),
    }
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
    raw: bool,
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let trigger = !self.mode.starts_with("late-") || r.text == "Alpha";
        let width = self.plain.text_width(r)?;
        if !self.fired && trigger {
            self.fired = true;
            let mut g = self.doc.snapshot().unwrap();
            let root = id(&g.root);
            let roots = id(&field(&g, &g.root, "roots"));
            let group = id(&field(&g, &Atom::Ref(roots), "0"));
            let children = id(&field(&g, &Atom::Ref(group), "children"));
            let first = id(&field(&g, &Atom::Ref(children), "0"));
            let second = id(&field(&g, &Atom::Ref(children), "1"));
            match self.mode {
                "name" => {
                    set_text(&mut g, group, "name", "Changed group");
                    set_text(&mut g, first, "name", "Changed Alpha");
                }
                "next-name" => set_text(&mut g, second, "name", "Changed Beta"),
                "values" => {
                    set(&mut g, first, "value", Atom::Number(40.));
                    set(&mut g, second, "value", Atom::Number(20.));
                }
                "children-replace" => {
                    let v = input(&mut g, &json!([{"name":"Replacement","value":9}]));
                    set(&mut g, group, "children", v);
                }
                "roots-replace" => {
                    let v = input(&mut g, &json!([{"name":"Replacement root","value":9}]));
                    set(&mut g, root, "roots", v);
                }
                "append" => {
                    let v = input(&mut g, &json!({"name":"Appended","value":8}));
                    append(&mut g, children, v);
                }
                "styles" => {
                    let v = input(&mut g, &json!(["fill:green", "color:white"]));
                    set(&mut g, first, "cssCompiledStyles", v);
                    set_text(&mut g, first, "classSelector", "changed");
                }
                "late-class" => {
                    set_text(&mut g, first, "classSelector", "late");
                    set_text(&mut g, first, "name", "After captured name");
                }
                "late-styles" => {
                    let styles = id(&field(&g, &Atom::Ref(first), "cssCompiledStyles"));
                    set_text(&mut g, styles, "0", "fill:green");
                }
                "shape" => {
                    let key = g.strings.iter().position(|s| s.is("children")).unwrap() as u32;
                    g.nodes[group as usize]
                        .properties_mut()
                        .retain(|p| p.key != key);
                    set(&mut g, group, "value", Atom::Number(8.));
                }
                "title-accessibility" => {
                    set_text(&mut g, root, "title", "Changed title");
                    set_text(&mut g, root, "accTitle", "Changed access");
                    set_text(&mut g, root, "accDescr", "Changed description");
                }
                "throw" => {
                    set_text(&mut g, root, "title", "Before throw");
                    set_text(&mut g, first, "name", "Saved Alpha");
                }
                "nested" => set_text(&mut g, root, "title", "Nested title"),
                "baseline" => {}
                _ => panic!("mode"),
            }
            if self.mode != "baseline" {
                self.doc.apply(self.doc.revision(), g).unwrap();
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
            if self.mode == "throw" {
                return Err(PeleError::new(
                    "Treemap callback sentinel",
                    ErrorCode::Semantic,
                ));
            }
            if self.mode == "nested" {
                let job = self
                    .doc
                    .prepare_detached_render(self.doc.revision(), request(self.raw, false))
                    .unwrap()
                    .unwrap();
                let mut reader = Read {
                    graph: self.graph.clone(),
                    reads: Rc::new(RefCell::new(vec![])),
                };
                let done = job.run(&mut reader, &mut Plain::default());
                let outcome = self.doc.finish_detached_render(done).unwrap();
                self.nested = Some(drawing(&rendered(outcome.result.unwrap())));
                let mut g = self.doc.snapshot().unwrap();
                set_text(&mut g, root, "title", "After nested");
                self.doc.apply(self.doc.revision(), g).unwrap();
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
        }
        Ok(width)
    }
}
#[test]
fn treemap_live_phases_preserve_captured_arrays_late_fields_errors_and_recovery() {
    let fixture = fixture();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 56);
    assert_eq!(
        fixture["oracleSHA256"],
        "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d"
    );
    for case in fixture["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let mut doc = PublicEngine::with_builtins()
            .parse(fixture["source"].as_str().unwrap(), &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = id(&initial.root);
        let roots = field(&initial, &initial.root, "roots");
        let group = field(&initial, &roots, "0");
        let children = field(&initial, &group, "children");
        let first = field(&initial, &children, "0");
        let second = field(&initial, &children, "1");
        let styles = input(&mut initial, &json!(["fill:red", "color:blue"]));
        set_text(&mut initial, root, "title", "Initial");
        set(
            &mut initial,
            id(&first),
            "cssCompiledStyles",
            styles.clone(),
        );
        set(
            &mut initial,
            id(&second),
            "cssCompiledStyles",
            styles.clone(),
        );
        doc.apply(doc.revision(), initial).unwrap();
        let initial = doc.snapshot().unwrap();
        let graph = Rc::new(RefCell::new(initial));
        let mut reader = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut host = Callback {
            doc: &mut doc,
            graph: graph.clone(),
            mode,
            fired: false,
            plain: Plain::default(),
            nested: None,
            raw,
        };
        let done = job.run(&mut reader, &mut host);
        assert!(host.fired);
        assert_json(&json!(host.plain.trace), &case["trace"]);
        assert_eq!(host.nested, case.get("nested").cloned());
        drop(host);
        let prior = doc.revision();
        let outcome = doc.finish_detached_render(done).unwrap();
        assert_eq!(outcome.revision, prior + 1);
        if case["first"]["ok"] == true {
            let result = drawing(&rendered(outcome.result.unwrap()));
            for k in ["svg", "width", "height"] {
                assert_eq!(result[k], case["first"][k], "{mode}/{raw}/{responsive}/{k}");
            }
        } else {
            let PublicError::Pele(e) = outcome.result.err().unwrap() else {
                panic!("error kind")
            };
            assert_eq!(e.message, "Treemap callback sentinel");
        }
        let after = doc.snapshot().unwrap();
        assert_json(&state(&after), &case["after"]);
        for (name, value) in [
            ("originalRoots", &roots),
            ("originalChildren", &children),
            ("originalFirst", &first),
            ("originalStyles", &styles),
        ] {
            assert_json(&output(&after, value), &case[name]);
        }
        let current_roots = field(&after, &after.root, "roots");
        let current_group = field(&after, &current_roots, "0");
        let current_children = field(&after, &current_group, "children");
        let current_first = field(&after, &current_children, "0");
        for (name, equal) in [
            (
                "rootsCurrent",
                matches!((&roots, &current_roots), (Atom::Ref(a), Atom::Ref(b)) if a == b),
            ),
            (
                "childrenCurrent",
                matches!((&children, &current_children), (Atom::Ref(a), Atom::Ref(b)) if a == b),
            ),
            (
                "firstCurrent",
                matches!((&first, &current_first), (Atom::Ref(a), Atom::Ref(b)) if a == b),
            ),
            (
                "stylesCurrent",
                matches!((&styles, field(&after, &first, "cssCompiledStyles")), (Atom::Ref(a), Atom::Ref(b)) if *a == b),
            ),
        ] {
            assert_eq!(equal, case[name].as_bool().unwrap());
        }
        *graph.borrow_mut() = after;
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut host = Plain::default();
        let done = job.run(&mut reader, &mut host);
        let result = doc.finish_detached_render(done).unwrap().result.unwrap();
        assert_eq!(drawing(&rendered(result)), case["recovery"]);
        assert_json(&json!(host.trace), &case["recoveryTrace"]);
        let mut options = RenderOptions::default();
        options.render.max_width = responsive.then_some(300.);
        assert_eq!(
            drawing(
                &doc.render(&options, &mut Plain::default())
                    .unwrap()
                    .rendered
            ),
            case["recovery"]
        );
    }
}
