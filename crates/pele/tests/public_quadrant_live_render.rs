//! Public Quadrant normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE: &str = "quadrantChart\ntitle Initial title\nx-axis Low --> High\ny-axis Bottom --> Top\nquadrant-1 Q1\nquadrant-2 Q2\nquadrant-3 Q3\nquadrant-4 Q4\nAlpha: [0.2, 0.3]\nBeta: [0.7, 0.8]";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-quadrant-live-render.json"
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
        max_width: responsive.then_some(320.),
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
        if key == "length" {
            if let Node::Array { length, .. } = &graph.nodes[*id as usize] {
                return Atom::Number(*length as f64);
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
    fn map_get(&mut self, receiver: &Value, key: &Value) -> Result<Value, PublicError> {
        let Value::Atom(Atom::Ref(id)) = receiver else {
            panic!("Map")
        };
        let graph = self.graph.borrow();
        let Node::Map { entries, .. } = &graph.nodes[*id as usize] else {
            panic!("Map")
        };
        let found = entries
            .iter()
            .find(|(k, _)| match (k, key) {
                (Atom::String(id), Value::String(s)) => graph.strings[*id as usize] == *s,
                _ => false,
            })
            .map(|(_, v)| v.clone())
            .unwrap_or(Atom::Undefined);
        Ok(self.own(found))
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
                message: JsString::from("Quadrant live read requires string"),
            })
    }
}

fn text(g: &mut ModelGraph, s: &str) -> Atom {
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(s));
    Atom::String(id)
}
fn set_at(g: &mut ModelGraph, receiver: &Atom, key: &str, value: Atom) {
    let Atom::Ref(id) = receiver else {
        panic!("record")
    };
    if let Some(p) = g.nodes[*id as usize]
        .properties_mut()
        .iter_mut()
        .find(|p| g.strings[p.key as usize].is(key))
    {
        p.value = value;
        return;
    }
    let Atom::String(key) = text(g, key) else {
        unreachable!()
    };
    g.nodes[*id as usize].properties_mut().push(Property {
        key,
        value,
        flags: 7,
    });
}
fn set_text(g: &mut ModelGraph, receiver: &Atom, key: &str, value: &str) {
    let v = text(g, value);
    set_at(g, receiver, key, v)
}
fn object(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    let receiver = Atom::Ref(id);
    for (k, v) in fields {
        set_at(g, &receiver, k, v);
    }
    receiver
}
fn label(g: &mut ModelGraph, value: &str) -> Atom {
    let t = text(g, value);
    let kind = text(g, "text");
    object(g, vec![("text", t), ("type", kind)])
}
fn point(g: &mut ModelGraph, name: &str, x: f64, y: f64) -> Atom {
    let t = label(g, name);
    let class = text(g, "");
    object(
        g,
        vec![
            ("x", Atom::Number(x)),
            ("y", Atom::Number(y)),
            ("text", t),
            ("className", class),
        ],
    )
}
fn put_index(g: &mut ModelGraph, array: &Atom, index: usize, value: Atom) {
    set_at(g, array, &index.to_string(), value);
    let Atom::Ref(id) = array else { panic!() };
    if let Node::Array { length, .. } = &mut g.nodes[*id as usize] {
        *length = (*length).max(index as u32 + 1);
    }
}
fn map_style(g: &mut ModelGraph, radius: f64, color: &str, stroke: bool) {
    let root = g.root.clone();
    let map = field(g, &root, "classes");
    let color = text(g, color);
    let mut fields = vec![("radius", Atom::Number(radius)), ("color", color)];
    if stroke {
        let sc = text(g, "00ff00");
        let sw = text(g, "2");
        fields.extend([("strokeColor", sc), ("strokeWidth", sw)]);
    }
    let value = object(g, fields);
    let key = text(g, "hot");
    let Atom::Ref(id) = map else { panic!() };
    let strings = &g.strings;
    let Node::Map { entries, .. } = &mut g.nodes[id as usize] else {
        panic!()
    };
    if let Some((_, v)) = entries
        .iter_mut()
        .find(|(k, _)| matches!(k,Atom::String(id)if strings[*id as usize].is("hot")))
    {
        *v = value;
    } else {
        entries.push((key, value));
    }
}
fn encoded(g: &ModelGraph, a: &Atom) -> Json {
    match a {
        Atom::Undefined => json!({"undefined":true}),
        Atom::Null => Json::Null,
        Atom::Bool(b) => json!(b),
        Atom::Number(n) => json!({"number":format!("{:016x}",n.to_bits())}),
        Atom::String(id) => json!(g.strings[*id as usize].well_formed().unwrap()),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { length, .. } => Json::Array(
                (0..*length)
                    .map(|i| encoded(g, &field(g, a, &i.to_string())))
                    .collect(),
            ),
            Node::Map { entries, .. } => {
                json!({"map":entries.iter().map(|(k,v)|vec![encoded(g,k),encoded(g,v)]).collect::<Vec<_>>()})
            }
            node => Json::Object(
                node.properties()
                    .iter()
                    .map(|p| {
                        (
                            g.strings[p.key as usize].well_formed().unwrap(),
                            encoded(g, &p.value),
                        )
                    })
                    .collect(),
            ),
        },
        _ => panic!("fixture"),
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
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let is_alpha = r.text == "Alpha";
        let width = self.plain.text_width(r)?;
        let point_mode = matches!(
            self.mode,
            "point-current" | "point-later" | "points-replace" | "points-append" | "class-style"
        );
        if self.fired || (point_mode && !is_alpha) {
            return Ok(width);
        }
        self.fired = true;
        let mut g = self.doc.snapshot().unwrap();
        let root = g.root.clone();
        let points = field(&g, &root, "points");
        let first = field(&g, &points, "0");
        let later = field(&g, &points, "1");
        match self.mode {
            "baseline" => {}
            "title" => set_text(&mut g, &root, "title", "Changed title"),
            "axes" => {
                let right = label(&mut g, "Changed right");
                let top = label(&mut g, "Changed top");
                set_at(&mut g, &root, "xAxisRight", right);
                set_at(&mut g, &root, "yAxisTop", top);
            }
            "quadrants" => {
                let qs = field(&g, &root, "quadrants");
                let q = label(&mut g, "Changed Q2");
                put_index(&mut g, &qs, 1, q);
            }
            "point-current" => {
                set_at(&mut g, &first, "x", Atom::Number(0.9));
                let t = label(&mut g, "Changed current");
                set_at(&mut g, &first, "text", t);
                set_text(&mut g, &first, "className", "changed");
            }
            "point-later" => {
                set_at(&mut g, &later, "x", Atom::Number(0.1));
                let t = label(&mut g, "Changed later");
                set_at(&mut g, &later, "text", t);
            }
            "points-replace" => {
                let p = point(&mut g, "Replacement", 0.5, 0.5);
                let id = g.nodes.len() as u32;
                g.nodes.push(Node::Array {
                    length: 0,
                    properties: vec![],
                });
                let a = Atom::Ref(id);
                put_index(&mut g, &a, 0, p);
                set_at(&mut g, &root, "points", a);
            }
            "points-append" => {
                let p = point(&mut g, "Appended", 0.4, 0.6);
                put_index(&mut g, &points, 2, p);
            }
            "class-style" => map_style(&mut g, 17., "0000ff", false),
            "accessibility" => {
                set_text(&mut g, &root, "accTitle", "Changed access");
                set_text(&mut g, &root, "accDescr", "Changed description");
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, &root, "title", "Persisted title");
                let label = field(&g, &later, "text");
                set_text(&mut g, &label, "text", "Persisted later");
            }
            "nested-helper" => {}
            "nested-render" => set_text(&mut g, &root, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" && self.mode != "nested-helper" {
            self.doc.apply(self.doc.revision(), g).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        if self.mode == "nested-helper" {
            let mut g = self.doc.snapshot().unwrap();
            let value = label(&mut g, "Nested Q2");
            self.doc.apply(self.doc.revision(), g).unwrap();
            // Public method2 is setQuadrant2Text; live render sees the published result.
            self.doc
                .invoke(
                    self.doc.revision(),
                    MethodCall {
                        method: MethodId(2),
                        receiver: root,
                        arguments: vec![value],
                    },
                    &mut Plain::default(),
                )
                .unwrap()
                .result
                .unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        if self.mode.starts_with("throw-") {
            return Err(PeleError::new(
                "Quadrant callback sentinel",
                ErrorCode::Semantic,
            ));
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
            let mut g = self.doc.snapshot().unwrap();
            let root = g.root.clone();
            set_text(&mut g, &root, "title", "After nested");
            self.doc.apply(self.doc.revision(), g).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        Ok(width)
    }
}
#[test]
fn quadrant_live_callbacks_preserve_phase_identity_partial_state_and_nested_operations() {
    for case in fixture()["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{mode}/{raw}/{responsive}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut g = doc.snapshot().unwrap();
        let root = g.root.clone();
        let points = field(&g, &root, "points");
        let first = field(&g, &points, "0");
        let second = field(&g, &points, "1");
        set_text(&mut g, &first, "className", "hot");
        set_text(&mut g, &second, "className", "hot");
        map_style(&mut g, 6., "ff0000", true);
        doc.apply(doc.revision(), g).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
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
        assert_json(
            &json!(callback.plain.trace),
            &case["trace"],
            &format!("{context}/trace"),
        );
        let nested = callback.nested.take();
        drop(callback);
        let outcome = doc.finish_detached_render(done).unwrap();
        if case["first"]["ok"] == true {
            let result = rendered(outcome.result.unwrap());
            for key in ["svg", "width", "height"] {
                assert_eq!(drawing(&result)[key], case["first"][key], "{context}/{key}");
            }
        } else {
            assert!(outcome.result.is_err(), "{context}");
        }
        let g = doc.snapshot().unwrap();
        assert_json(
            &encoded(&g, &g.root),
            &case["after"],
            &format!("{context}/state"),
        );
        assert_json(
            &encoded(&g, &points),
            &case["capturedPoints"],
            &format!("{context}/captured"),
        );
        let current = field(&g, &g.root, "points");
        assert_eq!(
            matches!((&current,&points),(Atom::Ref(a),Atom::Ref(b)) if a==b),
            case["originalPointsStillCurrent"].as_bool().unwrap(),
            "{context}"
        );
        assert_eq!(
            matches!((field(&g, &current, "0"),&first),(Atom::Ref(a),Atom::Ref(b)) if a==*b),
            case["originalPointStillFirst"].as_bool().unwrap(),
            "{context}"
        );
        if let Some(n) = nested {
            assert_json(&n, &case["nested"], &format!("{context}/nested"));
        }
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut plain = Plain::default();
        let done = job.run(&mut read, &mut plain);
        let result = rendered(doc.finish_detached_render(done).unwrap().result.unwrap());
        assert_json(
            &drawing(&result),
            &case["recovery"],
            &format!("{context}/recovery"),
        );
        assert_json(
            &json!(plain.trace),
            &case["recoveryTrace"],
            &format!("{context}/recoveryTrace"),
        );
    }
}
