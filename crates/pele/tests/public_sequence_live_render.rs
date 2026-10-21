//! Sequence callback phases through actual public normal/raw jobs.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-sequence-live-render.json"
    ))
    .unwrap()
}
fn request(raw: bool, responsive: bool) -> DetachedRenderRequest {
    let settings = RenderSettings {
        max_width: responsive.then_some(180.),
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
    json!({"svg":v.svg,"width":format!("{:016x}",v.width.to_bits()),"height":format!("{:016x}",v.height.to_bits()),"links":v.links})
}
fn field(graph: &ModelGraph, receiver: &Atom, key: &str) -> Atom {
    if let Atom::Ref(id) = receiver {
        if key == "size" {
            if let Node::Map { entries, .. } = &graph.nodes[*id as usize] {
                return Atom::Number(entries.len() as f64);
            }
        }
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
    Pair(Box<(Value, Value)>),
    Iterator(Rc<RefCell<(u32, usize)>>),
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
            Value::Pair(p) => match key {
                "0" => p.0.clone(),
                "1" => p.1.clone(),
                _ => Value::Atom(Atom::Undefined),
            },
            _ => Value::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, receiver: &Value, index: usize) -> Result<Value, PublicError> {
        self.get(receiver, &index.to_string())
    }
    fn iterator_begin(&mut self, v: &Value) -> Result<Value, PublicError> {
        let Value::Atom(Atom::Ref(id)) = v else {
            panic!("map")
        };
        Ok(Value::Iterator(Rc::new(RefCell::new((*id, 0)))))
    }
    fn iterator_next(&mut self, v: &Value) -> Result<Option<Value>, PublicError> {
        let Value::Iterator(state) = v else {
            panic!("iterator")
        };
        let mut state = state.borrow_mut();
        let g = self.graph.borrow();
        let Node::Map { entries, .. } = &g.nodes[state.0 as usize] else {
            panic!("map")
        };
        let pair = entries
            .get(state.1)
            .map(|(k, v)| Value::Pair(Box::new((self.own(k.clone()), self.own(v.clone())))));
        state.1 += 1;
        Ok(pair)
    }
    fn map_get_text(&mut self, v: &Value, key: &str) -> Result<Value, PublicError> {
        let Value::Atom(Atom::Ref(id)) = v else {
            panic!("map")
        };
        let g = self.graph.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("map")
        };
        Ok(self.own(
            entries
                .iter()
                .find_map(|(k, v)| {
                    if let Atom::String(id) = k {
                        g.strings[*id as usize].is(key).then(|| v.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or(Atom::Undefined),
        ))
    }
    fn object_identity(&mut self, value: &Value) -> Result<Option<u64>, PublicError> {
        Ok(if let Value::Atom(Atom::Ref(i)) = value {
            Some(u64::from(*i))
        } else {
            None
        })
    }
    fn has_index(&mut self, value: &Value, index: usize) -> Result<bool, PublicError> {
        Ok(if let Value::Atom(Atom::Ref(i)) = value {
            let g = self.graph.borrow();
            g.nodes[*i as usize]
                .properties()
                .iter()
                .any(|p| g.strings[p.key as usize].is(&index.to_string()))
        } else {
            false
        })
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
                message: JsString::from("Sequence live read requires string"),
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
        Json::Object(values) if values.contains_key("$map") => {
            let entries = values["$map"]
                .as_array()
                .unwrap()
                .iter()
                .map(|pair| (input(g, &pair[0]), input(g, &pair[1])))
                .collect();
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Map {
                entries,
                properties: vec![],
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
            Node::Map { entries, .. } => {
                json!({"$map":entries.iter().map(|(k,v)|json!([output(g,k),output(g,v)])).collect::<Vec<_>>()})
            }
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
fn map_value(g: &ModelGraph, map: &Atom, key: &str) -> Atom {
    let Node::Map { entries, .. } = &g.nodes[id(map) as usize] else {
        panic!("map")
    };
    entries
        .iter()
        .find(|(k, _)| matches!(k,Atom::String(i)if g.strings[*i as usize].is(key)))
        .map(|(_, v)| v.clone())
        .unwrap_or(Atom::Undefined)
}
fn map_set(g: &mut ModelGraph, map: &Atom, key: &str, value: Atom) {
    let k = text(g, key);
    let index = {
        let Node::Map { entries, .. } = &g.nodes[id(map) as usize] else {
            panic!("map")
        };
        entries
            .iter()
            .position(|(k, _)| matches!(k,Atom::String(i)if g.strings[*i as usize].is(key)))
    };
    let Node::Map { entries, .. } = &mut g.nodes[id(map) as usize] else {
        panic!("map")
    };
    if let Some(i) = index {
        entries[i].1 = value
    } else {
        entries.push((k, value));
    }
}
fn message(g: &ModelGraph, messages: &Atom, text: &str) -> Atom {
    let Node::Array { length, .. } = &g.nodes[id(messages) as usize] else {
        panic!("array")
    };
    (0..*length)
        .map(|i| field(g, messages, &i.to_string()))
        .find(|v| matches!(field(g,v,"message"),Atom::String(i)if g.strings[i as usize].is(text)))
        .unwrap()
}
fn state(g: &ModelGraph) -> Json {
    output(g, &g.root)
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
        let trigger = match self.mode {
            "message-fields" => r.text == "Hello",
            "note-placement" => r.text == "Detail",
            "frame-label" => r.text == "loop",
            "box-late" => r.text == "Group",
            "actor-class" => r.text == "Reply",
            _ => true,
        };
        let width = self.plain.text_width(r)?;
        if !self.fired && trigger {
            self.fired = true;
            let mut g = self.doc.snapshot().unwrap();
            let root = id(&g.root);
            let actors = field(&g, &g.root, "actors");
            let alpha = map_value(&g, &actors, "A");
            let beta = map_value(&g, &actors, "B");
            let boxed = field(&g, &alpha, "box");
            let messages = field(&g, &g.root, "messages");
            let hello = message(&g, &messages, "Hello");
            let note = message(&g, &messages, "Detail");
            let frame = message(&g, &messages, "Count");
            let mut calls = vec![];
            match self.mode {
                "actor-label" => {
                    set_text(&mut g, id(&alpha), "description", "Changed Alpha");
                    set_text(&mut g, id(&beta), "description", "Changed Beta");
                }
                "actor-map-append" => {
                    let v = input(
                        &mut g,
                        &json!({"name":"C","description":"Gamma","wrap":false,"type":"participant","links":{"$map":[]},"properties":{"$map":[]}}),
                    );
                    map_set(&mut g, &actors, "C", v);
                }
                "actor-map-delete" => {
                    let Node::Map { entries, .. } = &mut g.nodes[id(&actors) as usize] else {
                        panic!("map")
                    };
                    entries.remove(1);
                }
                "actor-map-replace" => {
                    let b = text(&mut g, "B");
                    let a = text(&mut g, "A");
                    let n = g.nodes.len() as u32;
                    g.nodes.push(Node::Map {
                        entries: vec![(b, beta.clone()), (a, alpha.clone())],
                        properties: vec![],
                    });
                    set(&mut g, root, "actors", Atom::Ref(n));
                }
                "lifecycle" => {
                    let created = field(&g, &g.root, "createdActors");
                    let destroyed = field(&g, &g.root, "destroyedActors");
                    map_set(&mut g, &created, "B", Atom::Number(2.));
                    map_set(&mut g, &destroyed, "B", Atom::Number(4.));
                }
                "message-replace" => {
                    let v = input(&mut g, &json!([]));
                    set(&mut g, root, "messages", v);
                }
                "message-append" => {
                    let raw = output(&g, &hello);
                    let v = input(&mut g, &raw);
                    set_text(&mut g, id(&v), "id", "appended");
                    set_text(&mut g, id(&v), "message", "Appended");
                    append(&mut g, id(&messages), v);
                }
                "message-fields" => {
                    set_text(&mut g, id(&hello), "id", "changed-message");
                    set(&mut g, id(&hello), "centralConnection", Atom::Number(61.));
                    set_text(&mut g, id(&hello), "message", "Changed Hello");
                    set_text(&mut g, id(&note), "message", "Changed detail");
                }
                "box-replace" => {
                    let v = input(
                        &mut g,
                        &json!({"name":"Replacement","fill":"green","wrap":false,"actorKeys":["B"]}),
                    );
                    set(&mut g, id(&beta), "box", v);
                }
                "box-late" => {
                    set_text(&mut g, id(&boxed), "name", "Changed box");
                    set_text(&mut g, id(&boxed), "fill", "green");
                }
                "actor-class" => {
                    for (a, label) in [(&alpha, "late-alpha"), (&beta, "late-beta")] {
                        let properties = field(&g, a, "properties");
                        let v = text(&mut g, label);
                        map_set(&mut g, &properties, "class", v);
                    }
                }
                "note-placement" => {
                    set(&mut g, id(&note), "placement", Atom::Number(0.));
                    set_text(&mut g, id(&note), "id", "changed-note");
                }
                "frame-label" => set_text(&mut g, id(&frame), "message", "Changed frame"),
                "metadata" => {
                    set_text(&mut g, root, "title", "Changed title");
                    set_text(&mut g, root, "accTitle", "Changed access");
                    set_text(&mut g, root, "accDescr", "Changed description");
                }
                "helper" => {
                    let a = text(&mut g, "C");
                    let b = text(&mut g, "Gamma");
                    let c = input(&mut g, &json!({"text":"Gamma","type":"participant"}));
                    calls.push((8, vec![a, b, c]));
                }
                "throw" => {
                    set_text(&mut g, root, "title", "Before throw");
                    set_text(&mut g, id(&beta), "description", "Saved Beta");
                }
                "nested" => set_text(&mut g, root, "title", "Nested title"),
                "baseline" => {}
                _ => panic!("mode"),
            }
            if self.mode != "baseline" {
                self.doc.apply(self.doc.revision(), g).unwrap();
                for (method, arguments) in calls {
                    let result = self
                        .doc
                        .invoke(
                            self.doc.revision(),
                            MethodCall {
                                method: MethodId(method),
                                receiver: Atom::Ref(root),
                                arguments,
                            },
                            &mut Plain::default(),
                        )
                        .unwrap();
                    assert!(result.result.is_ok());
                }
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
            if self.mode == "throw" {
                return Err(PeleError::new(
                    "Sequence callback sentinel",
                    ErrorCode::Semantic,
                ));
            }
            if self.mode == "nested" {
                let job = self
                    .doc
                    .prepare_detached_render(self.doc.revision(), request(self.raw, false))
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
                set_text(&mut g, root, "title", "After nested");
                self.doc.apply(self.doc.revision(), g).unwrap();
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
        }
        Ok(width)
    }
}
#[test]
fn sequence_live_fields_and_coordinate_iteration_match_oracle() {
    let fixture = fixture();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 76);
    assert_eq!(
        fixture["oracleSHA256"],
        "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d"
    );
    for case in fixture["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        // Native graph Maps expose current coordinates, not JavaScript tombstone history.
        // The deleted/reinserted visited-key row is proved by the real browser iterator.
        if mode == "actor-map-reinsert" {
            continue;
        }
        let mut doc = PublicEngine::with_builtins()
            .parse(fixture["source"].as_str().unwrap(), &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = id(&initial.root);
        set_text(&mut initial, root, "title", "Initial");
        let actors = field(&initial, &initial.root, "actors");
        let alpha = map_value(&initial, &actors, "A");
        let beta = map_value(&initial, &actors, "B");
        let boxed = field(&initial, &alpha, "box");
        let messages = field(&initial, &initial.root, "messages");
        let hello = message(&initial, &messages, "Hello");
        let note = message(&initial, &messages, "Detail");
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let mut read = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .expect("selected Sequence");
        let mut cb = Callback {
            doc: &mut doc,
            graph: graph.clone(),
            mode,
            fired: false,
            plain: Plain::default(),
            nested: None,
            raw,
        };
        let done = job.run(&mut read, &mut cb);
        let fired = cb.fired;
        let trace = cb.plain.trace.clone();
        let nested = cb.nested.clone();
        drop(cb);
        let (revision, result) = {
            let outcome = doc.finish_detached_render(done).unwrap();
            (outcome.revision, outcome.result)
        };
        assert_eq!(revision, doc.revision());
        if mode == "throw" {
            assert!(
                matches!(result,Err(PublicError::Pele(ref e)) if e.message=="Sequence callback sentinel")
            );
            assert_eq!(case["first"]["original"], true)
        } else {
            let actual = drawing(&rendered(result.unwrap()));
            for key in ["svg", "width", "height", "links"] {
                assert_eq!(
                    actual[key], case["first"][key],
                    "{mode}/{raw}/{responsive}/{key}"
                );
            }
        }
        assert!(fired);
        assert_json(&json!(trace), &case["trace"]);
        let after = doc.snapshot().unwrap();
        assert_json(&state(&after), &case["after"]);
        for (name, value) in [
            ("originalActors", &actors),
            ("originalAlpha", &alpha),
            ("originalBeta", &beta),
            ("originalBox", &boxed),
            ("originalMessages", &messages),
            ("originalHello", &hello),
            ("originalNote", &note),
        ] {
            assert_json(&output(&after, value), &case[name]);
        }
        for (name, old, current) in [
            (
                "actorsCurrent",
                &actors,
                field(&after, &after.root, "actors"),
            ),
            (
                "messagesCurrent",
                &messages,
                field(&after, &after.root, "messages"),
            ),
            ("alphaCurrent", &alpha, map_value(&after, &actors, "A")),
            ("betaCurrent", &beta, map_value(&after, &actors, "B")),
            ("boxCurrent", &boxed, field(&after, &beta, "box")),
        ] {
            assert_eq!(
                matches!((old,&current),(Atom::Ref(a),Atom::Ref(b))if a==b),
                case[name].as_bool().unwrap()
            );
        }
        if let Some(nested) = nested {
            assert_json(&nested, &case["nested"])
        }
        *graph.borrow_mut() = after;
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut plain = Plain::default();
        let done = job.run(&mut read, &mut plain);
        let outcome = doc.finish_detached_render(done).unwrap();
        assert_json(
            &drawing(&rendered(outcome.result.unwrap())),
            &case["recovery"],
        );
        assert_json(&json!(plain.trace), &case["recoveryTrace"]);
    }
}
