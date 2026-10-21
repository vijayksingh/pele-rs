//! Public Swimlane normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE: &str = "swimlane-beta LR\nA[Alpha] -->|first edge| B[Beta]\nB --> C[Gamma]\nstyle B fill:#113355\nclassDef custom color:#224466\nclass C custom";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-swimlane-live-render.json"
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
    Iterator(Rc<RefCell<(Value, usize)>>),
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
    fn iterator_begin(&mut self, v: &Value) -> Result<Value, PublicError> {
        Ok(Value::Iterator(Rc::new(RefCell::new((v.clone(), 0)))))
    }
    fn iterator_next(&mut self, it: &Value) -> Result<Option<Value>, PublicError> {
        let Value::Iterator(it) = it else {
            panic!("iterator")
        };
        let mut it = it.borrow_mut();
        let length = self.get(&it.0, "length")?;
        if it.1 >= self.number(&length).unwrap() as usize {
            return Ok(None);
        }
        let v = self.index(&it.0, it.1)?;
        it.1 += 1;
        Ok(Some(v))
    }
    fn iterator_close(&mut self, _: &Value) -> Result<(), PublicError> {
        Ok(())
    }
    fn call(
        &mut self,
        function: &Value,
        receiver: &Value,
        args: &[LiveCallArgument<Value>],
        _: &mut dyn RenderResources,
        _: &'static str,
    ) -> Result<Value, PublicError> {
        assert!(matches!(function, Value::Atom(Atom::BuiltinFunction(19))));
        let LiveCallArgument::Text(id) = &args[0] else {
            panic!("ID")
        };
        let lookup = self.get(receiver, "subgraphLookup")?;
        self.map_get(&lookup, &Value::String(id.clone()))
    }
    fn root(&self) -> Value {
        self.own(self.graph.borrow().root.clone())
    }
    fn get(&mut self, receiver: &Value, key: &str) -> Result<Value, PublicError> {
        self.reads.borrow_mut().push(json!(key));
        Ok(match receiver {
            Value::Atom(atom) => {
                let value = field(&self.graph.borrow(), atom, key);
                if key == "subgraph" && matches!(value, Atom::Undefined) {
                    Value::Atom(Atom::BuiltinFunction(19))
                } else {
                    self.own(value)
                }
            }
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
        match &g.nodes[*id as usize] {
            Node::Map { entries, .. } => {
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
            Node::Array { length, .. } => {
                assert!((*length as usize) <= max);
                Ok((0..*length)
                    .map(|i| self.own(field(&g, &Atom::Ref(*id), &i.to_string())))
                    .collect())
            }
            _ => panic!("iterable"),
        }
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
                message: JsString::from("Swimlane live read requires string"),
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
        "direction",
        "nodes",
        "edges",
        "subgraphs",
        "classes",
        "tooltips",
        "defaultEdgeStyle",
    ] {
        let value = field(g, &g.root, key);
        if matches!(value, Atom::Undefined) {
            continue;
        }
        let observed = if matches!(key, "nodes" | "classes" | "tooltips") {
            let Atom::Ref(id) = value else {
                panic!("top-level Map")
            };
            let Node::Map { entries, .. } = &g.nodes[id as usize] else {
                panic!("top-level Map")
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
        out.insert(key.into(), observed);
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
        let later = map_value(&g, "nodes", "B");
        match self.mode {
            "baseline" => {}
            "late-title" => set_text(&mut g, &root, "title", "Changed title"),
            "late-accessibility" => {
                set_text(&mut g, &root, "accTitle", "Changed access");
                set_text(&mut g, &root, "accDescr", "Changed description");
            }
            "copied-label" => set_text(&mut g, &later, "text", "Changed later"),
            "copied-edge" => {
                let list = field(&g, &root, "edges");
                let edge = field(&g, &list, "0");
                set_text(&mut g, &edge, "text", "Changed edge");
                set_text(&mut g, &edge, "end", "C");
            }
            "style-mutate" | "compiled-class" => {
                let v = if self.mode == "compiled-class" {
                    map_value(&g, "classes", "custom")
                } else {
                    later.clone()
                };
                let a = field(&g, &v, "styles");
                let Atom::Number(n) = field(&g, &a, "length") else {
                    panic!()
                };
                let texts = if self.mode == "compiled-class" {
                    ["fill:#ff00ff", "font-size:25px"]
                } else {
                    ["fill:#ff0000", "font-size:22px"]
                };
                for (i, t) in texts.iter().enumerate() {
                    let v = text(&mut g, t);
                    put_index(&mut g, &a, n as usize + i, v);
                }
            }
            "style-replace" => {
                let values = ["fill:#00ff00", "font-size:24px"]
                    .iter()
                    .map(|t| text(&mut g, t))
                    .collect();
                let a = array(&mut g, values);
                set_at(&mut g, &later, "styles", a);
            }
            "direction" => set_text(&mut g, &root, "direction", "TB"),
            "replace-maps" => {
                let key = text(&mut g, "B");
                let id = g.nodes.len() as u32;
                g.nodes.push(Node::Map {
                    entries: vec![(key, later)],
                    properties: vec![],
                });
                set_at(&mut g, &root, "nodes", Atom::Ref(id));
            }
            "append-edge" => {
                let edges = field(&g, &root, "edges");
                let Atom::Ref(first) = field(&g, &edges, "0") else {
                    panic!()
                };
                let edge = g.nodes[first as usize].clone();
                let id = g.nodes.len() as u32;
                g.nodes.push(edge);
                let edge = Atom::Ref(id);
                set_text(&mut g, &edge, "start", "C");
                set_text(&mut g, &edge, "end", "A");
                set_text(&mut g, &edge, "id", "new-edge");
                let Atom::Number(n) = field(&g, &edges, "length") else {
                    panic!()
                };
                put_index(&mut g, &edges, n as usize, edge);
            }
            "default-style" => {
                let v = text(&mut g, "stroke:#ff0000");
                let a = array(&mut g, vec![v]);
                set_at(&mut g, &root, "defaultEdgeStyle", a);
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, &root, "title", "Saved");
                set_text(&mut g, &later, "text", "Saved later");
            }
            "budget-compiled" => {
                let class = map_value(&g, "classes", "custom");
                let huge = text(&mut g, &"x".repeat(6_000_000));
                let styles = array(&mut g, vec![huge]);
                set_at(&mut g, &class, "styles", styles);
                for name in ["A", "B", "C"] {
                    let node = map_value(&g, "nodes", name);
                    let v = text(&mut g, "custom");
                    let classes = array(&mut g, vec![v]);
                    set_at(&mut g, &node, "classes", classes);
                }
            }
            "budget-edge" => {
                let huge = text(&mut g, &"x".repeat(9_000_000));
                let styles = array(&mut g, vec![huge]);
                set_at(&mut g, &root, "defaultEdgeStyle", styles);
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
                "Swimlane callback sentinel",
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
        let hit = r.text == "Alpha";
        let result = self.plain.text_width(r)?;
        if !self.fired && hit {
            self.mutate()?;
        }
        Ok(result)
    }
}
#[test]
fn swimlane_copied_graph_late_metadata_responsive_redraw_and_recovery() {
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
            let later = map_value(&g, "nodes", "B");
            (
                field(&g, &g.root, "nodes"),
                later.clone(),
                field(&g, &later, "styles"),
                field(&g, &g.root, "edges"),
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
                matches!(outcome.result,Err(PublicError::Pele(ref e)) if e.message=="Swimlane callback sentinel"),
                "{context}"
            );
        }
        let g = doc.snapshot().unwrap();
        assert_json(&state(&g), &case["after"], &context);
        let later = map_value(&g, "nodes", "B");
        for (name, a, b) in [
            ("nodesCurrent", &captured.0, field(&g, &g.root, "nodes")),
            ("laterCurrent", &captured.1, later.clone()),
            ("stylesCurrent", &captured.2, field(&g, &later, "styles")),
            ("edgesCurrent", &captured.3, field(&g, &g.root, "edges")),
        ] {
            assert_eq!(
                same(a, &b),
                case[name].as_bool().unwrap(),
                "{context}/{name}"
            );
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

#[test]
fn swimlane_callback_expanded_style_limits_preserve_edits_and_recover() {
    for mode in ["budget-compiled", "budget-edge"] {
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let baseline = doc.snapshot().unwrap();
        let graph = Rc::new(RefCell::new(baseline.clone()));
        let mut read = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
        let job = doc
            .prepare_detached_render(doc.revision(), request(false, true))
            .unwrap()
            .unwrap();
        let done = job.run(&mut read, &mut Plain::default());
        let expected = rendered(doc.finish_detached_render(done).unwrap().result.unwrap());
        let job = doc
            .prepare_detached_render(doc.revision(), request(false, true))
            .unwrap()
            .unwrap();
        let mut callback = Callback {
            doc: &mut doc,
            graph: graph.clone(),
            mode,
            raw: false,
            fired: false,
            plain: Plain::default(),
            nested: None,
        };
        let done = job.run(&mut read, &mut callback);
        assert!(callback.fired);
        drop(callback);
        let outcome = doc.finish_detached_render(done).unwrap();
        assert!(
            matches!(outcome.result,Err(PublicError::Pele(ref e))if e.code==ErrorCode::Limit),
            "{mode}: {:?}",
            outcome.result.as_ref().err()
        );
        assert!(doc.snapshot().unwrap().nodes.len() > baseline.nodes.len());
        doc.apply(doc.revision(), baseline).unwrap();
        *graph.borrow_mut() = doc.snapshot().unwrap();
        let job = doc
            .prepare_detached_render(doc.revision(), request(false, true))
            .unwrap()
            .unwrap();
        let done = job.run(&mut read, &mut Plain::default());
        let actual = rendered(doc.finish_detached_render(done).unwrap().result.unwrap());
        assert_eq!(actual.svg, expected.svg);
    }
}
