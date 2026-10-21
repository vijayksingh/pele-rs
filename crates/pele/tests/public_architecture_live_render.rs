//! Public Architecture normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE:&str="architecture-beta\ngroup g(cloud)[Cluster]\nservice a(server)[First] in g\nservice b(database)[Second] in g\nservice c(disk)[Outside]\na:R -- L:b\nb:B -- T:c";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-architecture-live-render.json"
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
        max_width: responsive.then_some(220.),
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
            Value::Pair(values) => key
                .parse::<usize>()
                .ok()
                .and_then(|i| values.get(i))
                .cloned()
                .unwrap_or(Value::Atom(Atom::Undefined)),
            _ => Value::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, receiver: &Value, index: usize) -> Result<Value, PublicError> {
        self.get(receiver, &index.to_string())
    }
    fn has_index(&mut self, receiver: &Value, index: usize) -> Result<bool, PublicError> {
        let Value::Atom(Atom::Ref(id)) = receiver else {
            return Ok(false);
        };
        let g = self.graph.borrow();
        Ok(g.nodes[*id as usize]
            .properties()
            .iter()
            .any(|p| g.strings[p.key as usize].is(&index.to_string())))
    }
    fn snapshot_iterable(&mut self, value: &Value, max: usize) -> Result<Vec<Value>, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("Map")
        };
        let g = self.graph.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("Map")
        };
        assert!(entries.len() <= max);
        Ok(entries
            .iter()
            .map(|(k, v)| {
                Value::Pair(std::sync::Arc::new([
                    self.own(k.clone()),
                    self.own(v.clone()),
                ]))
            })
            .collect())
    }
    fn map_get_text(&mut self, value: &Value, key: &str) -> Result<Value, PublicError> {
        self.map_get(value, &Value::String(JsString::from(key)))
    }
    fn map_get(&mut self, value: &Value, key: &Value) -> Result<Value, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("Map")
        };
        let g = self.graph.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("Map")
        };
        Ok(entries
            .iter()
            .find_map(|(k, v)| match (k, key) {
                (Atom::String(i), Value::String(s)) if g.strings[*i as usize] == *s => {
                    Some(self.own(v.clone()))
                }
                _ => None,
            })
            .unwrap_or(Value::Atom(Atom::Undefined)))
    }
    fn array_join(&mut self, value: &Value, separator: &str) -> Result<JsString, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("array")
        };
        let g = self.graph.borrow();
        let Node::Array { length, .. } = g.nodes[*id as usize] else {
            panic!("array")
        };
        let values = (0..length)
            .map(|i| match field(&g, &Atom::Ref(*id), &i.to_string()) {
                Atom::String(id) => g.strings[id as usize].well_formed().unwrap(),
                Atom::Undefined | Atom::Null => String::new(),
                _ => panic!("fixture join"),
            })
            .collect::<Vec<_>>();
        Ok(JsString::from(values.join(separator).as_str()))
    }
    fn object_identity(&mut self, value: &Value) -> Result<Option<u64>, PublicError> {
        Ok(if let Value::Atom(Atom::Ref(id)) = value {
            Some(*id as u64)
        } else {
            None
        })
    }
    fn is_array(&mut self, value: &Value) -> Result<bool, PublicError> {
        Ok(match value {
            Value::Atom(Atom::Ref(id)) => {
                matches!(self.graph.borrow().nodes[*id as usize], Node::Array { .. })
            }
            _ => false,
        })
    }
    fn own_keys(&mut self, value: &Value) -> Result<Vec<JsString>, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("record")
        };
        let g = self.graph.borrow();
        let mut keys: Vec<_> = g.nodes[*id as usize]
            .properties()
            .iter()
            .filter(|p| p.flags & 1 != 0)
            .map(|p| g.strings[p.key as usize].clone())
            .collect();
        keys.sort_by_key(|s| {
            let text = s.well_formed().unwrap();
            text.parse::<u32>()
                .ok()
                .filter(|n| *n < u32::MAX && n.to_string() == text)
                .map_or((1, 0), |n| (0, n))
        });
        Ok(keys)
    }
    fn snapshot_map_values(
        &mut self,
        value: &Value,
        max: usize,
    ) -> Result<Vec<Value>, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("Map")
        };
        let g = self.graph.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("Map")
        };
        assert!(entries.len() <= max);
        Ok(entries.iter().map(|(_, v)| self.own(v.clone())).collect())
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
                message: JsString::from("Architecture live read requires string"),
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
fn put_index(g: &mut ModelGraph, array: &Atom, index: usize, value: Atom) {
    set_at(g, array, &index.to_string(), value);
    let Atom::Ref(id) = array else { panic!() };
    if let Node::Array { length, .. } = &mut g.nodes[*id as usize] {
        *length = (*length).max(index as u32 + 1);
    }
}
fn encoded(g: &ModelGraph, a: &Atom) -> Json {
    match a {
        Atom::Undefined => Json::Null,
        Atom::Null => Json::Null,
        Atom::Bool(b) => json!(b),
        Atom::Number(n) => json!(n),
        Atom::String(id) => json!(g.strings[*id as usize].well_formed().unwrap()),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { length, .. } => Json::Array(
                (0..*length)
                    .map(|i| encoded(g, &field(g, a, &i.to_string())))
                    .collect(),
            ),
            node => Json::Object(
                node.properties()
                    .iter()
                    .filter(|p| !matches!(p.value, Atom::Undefined))
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

fn array(g: &mut ModelGraph, values: Vec<Atom>) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Array {
        length: 0,
        properties: vec![],
    });
    let a = Atom::Ref(id);
    for (i, v) in values.into_iter().enumerate() {
        put_index(g, &a, i, v);
    }
    a
}
fn map_value(g: &ModelGraph, key: &str, name: &str) -> Atom {
    let Atom::Ref(id) = field(g, &g.root, key) else {
        panic!("Map")
    };
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        panic!("Map")
    };
    entries
        .iter()
        .find_map(|(k, v)| {
            matches!(k,Atom::String(id) if g.strings[*id as usize].is(name)).then_some(v.clone())
        })
        .unwrap_or(Atom::Undefined)
}
fn state(g: &ModelGraph) -> Json {
    let mut out = serde_json::Map::new();
    for key in [
        "title",
        "accTitle",
        "accDescr",
        "nodes",
        "groups",
        "edges",
        "layoutHints",
    ] {
        let value = field(g, &g.root, key);
        let observed = if matches!(key, "nodes" | "groups") {
            let Atom::Ref(id) = value else { panic!() };
            let Node::Map { entries, .. } = &g.nodes[id as usize] else {
                panic!()
            };
            Json::Array(
                entries
                    .iter()
                    .map(|(k, v)| Json::Array(vec![encoded(g, k), encoded(g, v)]))
                    .collect(),
            )
        } else {
            encoded(g, &value)
        };
        if !matches!(value, Atom::Undefined) {
            out.insert(key.into(), observed);
        }
    }
    Json::Object(out)
}
fn same(a: &Atom, b: &Atom) -> bool {
    matches!((a,b),(Atom::Ref(x),Atom::Ref(y))if x==y)
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
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.trace.push(json!(["icon", name]));
        Ok(Some(TrustedSvg::from_trusted_markup(
            "<path d=\"M0,0H2\"/>".into(),
        )))
    }
}
struct Callback<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    raw: bool,
    fired: bool,
    plain: Plain,
    nested: Option<Json>,
}
impl Callback<'_> {
    fn mutate(&mut self) -> Result<(), PeleError> {
        self.fired = true;
        let mut g = self.doc.snapshot()?;
        let root = g.root.clone();
        let later = map_value(&g, "nodes", "b");
        let group = map_value(&g, "groups", "g");
        let edges = field(&g, &root, "edges");
        match self.mode {
            "baseline" => {}
            "late-title" => set_text(&mut g, &root, "title", "Changed title"),
            "late-accessibility" => {
                set_text(&mut g, &root, "accTitle", "Changed access");
                set_text(&mut g, &root, "accDescr", "Changed description");
            }
            "later-label" => set_text(&mut g, &later, "title", "Changed second"),
            "replace-maps" => {
                let c = map_value(&g, "nodes", "c");
                let bk = text(&mut g, "b");
                let ck = text(&mut g, "c");
                let n = g.nodes.len() as u32;
                g.nodes.push(Node::Map {
                    entries: vec![(bk, later.clone()), (ck, c)],
                    properties: vec![],
                });
                set_at(&mut g, &root, "nodes", Atom::Ref(n));
                let n = g.nodes.len() as u32;
                g.nodes.push(Node::Map {
                    entries: vec![],
                    properties: vec![],
                });
                set_at(&mut g, &root, "groups", Atom::Ref(n));
            }
            "append-node" => {
                let mut fields = vec![];
                for (k, v) in [
                    ("id", "d"),
                    ("type", "service"),
                    ("icon", "server"),
                    ("title", "Appended"),
                ] {
                    fields.push((k, text(&mut g, v)));
                }
                let n = object(&mut g, fields);
                let k = text(&mut g, "d");
                let Atom::Ref(id) = field(&g, &root, "nodes") else {
                    panic!()
                };
                let Node::Map { entries, .. } = &mut g.nodes[id as usize] else {
                    panic!()
                };
                entries.push((k, n));
            }
            "edge-title" => {
                let e = field(&g, &edges, "0");
                set_text(&mut g, &e, "title", "Changed edge");
            }
            "replace-edges" => {
                let e = field(&g, &edges, "1");
                let Atom::Ref(id) = e else { panic!() };
                let node = g.nodes[id as usize].clone();
                let n = g.nodes.len() as u32;
                g.nodes.push(node);
                let e = Atom::Ref(n);
                set_text(&mut g, &e, "title", "Replacement edge");
                let a = array(&mut g, vec![e]);
                set_at(&mut g, &root, "edges", a);
            }
            "group-title" => set_text(&mut g, &group, "title", "Changed cluster"),
            "group-icon" => set_text(&mut g, &group, "icon", "disk"),
            "node-icon" => set_text(&mut g, &later, "icon", "internet"),
            "icon-group-id" => {
                set_text(&mut g, &group, "id", "changed-group");
                set_text(&mut g, &root, "title", "After icon");
            }
            "icon-later-node" => {
                set_text(&mut g, &later, "title", "Changed after icon");
                set_text(&mut g, &later, "icon", "internet");
                set_text(&mut g, &later, "id", "changed-node");
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, &root, "title", "Saved");
                set_text(&mut g, &later, "title", "Saved later");
            }
            "nested" => set_text(&mut g, &root, "title", "Nested"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if self.mode.starts_with("throw-") {
            return Err(PeleError::new(
                "Architecture callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested" {
            let job = self
                .doc
                .prepare_detached_render(self.doc.revision(), request(self.raw, false))?
                .unwrap();
            let mut read = Read {
                graph: self.graph.clone(),
                reads: Rc::new(RefCell::new(vec![])),
            };
            let done = job.run(&mut read, &mut Plain::default());
            let out = self.doc.finish_detached_render(done)?;
            self.nested = Some(drawing(&rendered(out.result.unwrap())));
            let mut g = self.doc.snapshot()?;
            let root = g.root.clone();
            set_text(&mut g, &root, "title", "After nested");
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        Ok(())
    }
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let hit = r.text == "First" && !self.mode.starts_with("icon-");
        let result = self.plain.text_width(r)?;
        if !self.fired && hit {
            self.mutate()?;
        }
        Ok(result)
    }
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        let result = self.plain.icon(name)?;
        if !self.fired && self.mode.starts_with("icon-") && name == "cloud" {
            self.mutate()?;
        }
        Ok(result)
    }
}
#[test]
fn architecture_captured_collections_live_paint_and_recovery() {
    for case in fixture()["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{mode}/{raw}/{responsive}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = initial.root.clone();
        set_text(&mut initial, &root, "title", "Initial");
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let captured = {
            let g = graph.borrow();
            (
                field(&g, &g.root, "nodes"),
                field(&g, &g.root, "groups"),
                map_value(&g, "nodes", "b"),
                field(&g, &g.root, "edges"),
                map_value(&g, "nodes", "a"),
                map_value(&g, "groups", "g"),
            )
        };
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
            raw,
            fired: false,
            plain: Plain::default(),
            nested: None,
        };
        let done = job.run(&mut read, &mut callback);
        assert!(callback.fired, "{context}");
        assert_json(&json!(callback.plain.trace), &case["trace"], &context);
        let nested = callback.nested.take();
        drop(callback);
        let outcome = doc.finish_detached_render(done).unwrap();
        if case["first"]["ok"] == true {
            let d = drawing(&rendered(outcome.result.unwrap()));
            for key in ["svg", "width", "height"] {
                assert_eq!(d[key], case["first"][key], "{context}/{key}");
            }
        } else {
            assert!(
                matches!(outcome.result,Err(PublicError::Pele(ref e)) if e.message=="Architecture callback sentinel"),
                "{context}"
            );
        }
        let g = doc.snapshot().unwrap();
        assert_json(&state(&g), &case["after"], &context);
        for (name, a, b) in [
            ("nodesCurrent", &captured.0, field(&g, &g.root, "nodes")),
            ("groupsCurrent", &captured.1, field(&g, &g.root, "groups")),
            ("laterCurrent", &captured.2, map_value(&g, "nodes", "b")),
            ("edgesCurrent", &captured.3, field(&g, &g.root, "edges")),
        ] {
            assert_eq!(
                same(a, &b),
                case[name].as_bool().unwrap(),
                "{context}/{name}"
            );
        }
        for (key, value) in [
            ("originalFirst", &captured.4),
            ("originalLater", &captured.2),
            ("originalGroup", &captured.5),
        ] {
            assert_json(&encoded(&g, value), &case[key], &format!("{context}/{key}"));
        }
        if let Some(n) = nested {
            assert_json(&n, &case["nested"], &context);
        }
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut plain = Plain::default();
        let done = job.run(&mut read, &mut plain);
        let result = rendered(doc.finish_detached_render(done).unwrap().result.unwrap());
        assert_json(&drawing(&result), &case["recovery"], &context);
        assert_json(&json!(plain.trace), &case["recoveryTrace"], &context);
    }
}
