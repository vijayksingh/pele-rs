//! State callback phases through actual public normal/raw jobs.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-state-live-render.json"
    ))
    .unwrap()
}
fn request(raw: bool, responsive: bool) -> DetachedRenderRequest {
    let settings = RenderSettings {
        max_width: responsive.then_some(160.),
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
        if key == "direction" {
            if let Value::Atom(Atom::Ref(id)) = receiver {
                let graph = self.graph.borrow();
                let node = &graph.nodes[*id as usize];
                if matches!(
                    node,
                    Node::Instance {
                        prototype: PrototypeId(1),
                        ..
                    }
                ) && !node
                    .properties()
                    .iter()
                    .any(|p| graph.strings[p.key as usize].is(key))
                {
                    // Test host uses the public native getter capability, not a
                    // second handwritten State direction algorithm.
                    let owner = PublicDocument::from_graph(
                        pele::compat::diagrams::state::State,
                        graph.clone(),
                        Object::default(),
                    )?;
                    return match owner
                        .read_declared_getter(&Atom::Ref(*id), MethodId(1))
                        .expect("State getter")?
                    {
                        PublicActiveValue::String(text) => Ok(Value::String(text)),
                        PublicActiveValue::Atom(atom) => Ok(self.own(atom)),
                    };
                }
            }
        }
        Ok(match receiver {
            Value::Atom(atom) => self.own(field(&self.graph.borrow(), atom, key)),
            _ => Value::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, receiver: &Value, index: usize) -> Result<Value, PublicError> {
        self.get(receiver, &index.to_string())
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
                message: JsString::from("State live read requires string"),
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
            "description-replace" => r.text == "Alpha",
            "description-append" => r.text == "Detail one",
            "late-fields" | "edge-late" => r.text == "transition",
            _ => true,
        };
        let width = self.plain.text_width(r)?;
        if !self.fired && trigger {
            self.fired = true;
            let mut g = self.doc.snapshot().unwrap();
            let root = id(&g.root);
            let graph = field(&g, &g.root, "graph");
            let nodes = field(&g, &graph, "nodes");
            let group = field(&g, &nodes, "0");
            let alpha = field(&g, &nodes, "1");
            let beta = field(&g, &nodes, "2");
            let edges = field(&g, &graph, "edges");
            let edge = field(&g, &edges, "0");
            let description = field(&g, &alpha, "description");
            let styles = field(&g, &beta, "cssStyles");
            let mut calls = vec![];
            match self.mode {
                "node-label" => {
                    set_text(&mut g, id(&alpha), "label", "Changed Alpha");
                    set_text(&mut g, id(&beta), "label", "Changed Beta");
                }
                "description-replace" => {
                    let v = input(&mut g, &json!(["Replacement one", "Replacement two"]));
                    set(&mut g, id(&alpha), "description", v);
                }
                "description-append" => {
                    let v = text(&mut g, "Appended detail");
                    append(&mut g, id(&description), v);
                    let v = input(&mut g, &json!(["Replaced after capture"]));
                    set(&mut g, id(&alpha), "description", v);
                }
                "styles-replace" => {
                    let v = input(&mut g, &json!(["fill:green"]));
                    set(&mut g, id(&beta), "cssStyles", v);
                }
                "styles-mutate" => set_text(&mut g, id(&styles), "0", "fill:green"),
                "late-fields" => {
                    set_text(&mut g, id(&beta), "cssClasses", "late");
                    set_text(&mut g, id(&beta), "id", "Changed B");
                }
                "graph-replace" => {
                    let v = input(&mut g, &json!({"nodes":[],"edges":[]}));
                    set(&mut g, root, "graph", v);
                }
                "edges-replace" => {
                    let v = input(
                        &mut g,
                        &json!([{"id":"new-edge","start":"B","end":"A","label":"Replacement edge","classes":"transition"}]),
                    );
                    set(&mut g, id(&graph), "edges", v);
                }
                "edge-late" => {
                    set_text(&mut g, id(&edge), "id", "Changed edge");
                    set_text(&mut g, id(&edge), "pattern", "dashed");
                    set_text(&mut g, id(&edge), "label", "After captured label");
                    let v = input(
                        &mut g,
                        &json!({"id":"new-edge","start":"B","end":"A","label":"Appended edge","classes":"transition"}),
                    );
                    append(&mut g, id(&edges), v);
                }
                "links" => {
                    let map = field(&g, &g.root, "links");
                    let k = text(&mut g, "A");
                    let v = input(
                        &mut g,
                        &json!({"url":"https://example.com/alpha","tooltip":"Alpha link"}),
                    );
                    let Node::Map { entries, .. } = &mut g.nodes[id(&map) as usize] else {
                        panic!("map")
                    };
                    entries.push((k, v));
                }
                "direction" => {
                    set_text(&mut g, id(&group), "dir", "TB");
                    let v = text(&mut g, "TB");
                    calls.push((14, vec![v]));
                }
                "title-accessibility" => {
                    set_text(&mut g, root, "title", "Changed title");
                    set_text(&mut g, root, "accTitle", "Changed access");
                    set_text(&mut g, root, "accDescr", "Changed description");
                }
                "helper" => {
                    let name = text(&mut g, "Added");
                    let detail = text(&mut g, "Saved helper");
                    calls.push((5, vec![name.clone()]));
                    calls.push((9, vec![name, detail]));
                }
                "throw" => {
                    set_text(&mut g, root, "title", "Before throw");
                    set_text(&mut g, id(&alpha), "label", "Saved Alpha");
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
                    "State callback sentinel",
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
fn state_live_identity_fields_nested_callbacks_and_recovery_match_oracle() {
    let fixture = fixture();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 64);
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
        set_text(&mut initial, root, "title", "Initial");
        let original_graph = field(&initial, &initial.root, "graph");
        let nodes = field(&initial, &original_graph, "nodes");
        let group = field(&initial, &nodes, "0");
        let alpha = field(&initial, &nodes, "1");
        let beta = field(&initial, &nodes, "2");
        let edges = field(&initial, &original_graph, "edges");
        let edge = field(&initial, &edges, "0");
        let description = field(&initial, &alpha, "description");
        let styles = field(&initial, &beta, "cssStyles");
        set_text(&mut initial, id(&group), "dir", "LR");
        let v = text(&mut initial, "fill:red");
        append(&mut initial, id(&styles), v);
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let mut read = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .expect("selected State");
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
                matches!(result,Err(PublicError::Pele(ref e)) if e.message=="State callback sentinel")
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
            ("originalGraph", &original_graph),
            ("originalNodes", &nodes),
            ("originalAlpha", &alpha),
            ("originalBeta", &beta),
            ("originalEdges", &edges),
            ("originalEdge", &edge),
            ("originalDescription", &description),
            ("originalStyles", &styles),
        ] {
            assert_json(&output(&after, value), &case[name]);
        }
        for (name, old, current) in [
            (
                "graphCurrent",
                &original_graph,
                field(&after, &after.root, "graph"),
            ),
            (
                "edgesCurrent",
                &edges,
                field(&after, &original_graph, "edges"),
            ),
            (
                "descriptionCurrent",
                &description,
                field(&after, &alpha, "description"),
            ),
            ("stylesCurrent", &styles, field(&after, &beta, "cssStyles")),
        ] {
            assert_eq!(
                matches!((old,&current),(Atom::Ref(a),Atom::Ref(b)) if a==b),
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

struct DirectionCallback<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    fired: bool,
    plain: Plain,
}
impl RenderResources for DirectionCallback<'_> {
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
        let width = self.plain.text_width(request)?;
        if !self.fired {
            self.fired = true;
            if self.mode == "callback-undefined" {
                let mut graph = self.doc.snapshot()?;
                let root = id(&graph.root);
                set(&mut graph, root, "direction", Atom::Undefined);
                let key = graph
                    .strings
                    .iter()
                    .position(|s| s.is("direction"))
                    .unwrap() as u32;
                graph.nodes[root as usize]
                    .properties_mut()
                    .iter_mut()
                    .find(|p| p.key == key)
                    .unwrap()
                    .flags = 6;
                self.doc.apply(self.doc.revision(), graph)?;
            } else if self.mode == "callback-helper" {
                let mut graph = self.doc.snapshot()?;
                let argument = text(&mut graph, "TB");
                self.doc.apply(self.doc.revision(), graph)?;
                self.doc
                    .invoke(
                        self.doc.revision(),
                        MethodCall {
                            method: MethodId(14),
                            receiver: self.doc.snapshot()?.root,
                            arguments: vec![argument],
                        },
                        &mut Plain::default(),
                    )
                    .map_err(|e| PeleError::new(format!("{e:?}"), ErrorCode::Semantic))?
                    .result
                    .map_err(|e| PeleError::new(format!("{e:?}"), ErrorCode::Semantic))?;
            }
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        Ok(width)
    }
}
#[test]
fn state_declared_direction_getter_preserves_own_shadowing_and_callback_repair() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-state-direction-accessor.json"
    ))
    .unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 20);
    for expected in fixture["rows"].as_array().unwrap() {
        let mode = expected["mode"].as_str().unwrap();
        if mode.starts_with("getter-") {
            continue;
        } // Browser exercises live host accessors.
        let raw = expected["raw"].as_bool().unwrap();
        let mut document = PublicEngine::with_builtins()
            .parse(fixture["source"].as_str().unwrap(), &Default::default())
            .unwrap();
        assert_eq!(
            document.detached_public_accessors(),
            &[(PrototypeId(1), MethodId(1))]
        );
        let mut graph = document.snapshot().unwrap();
        let root = id(&graph.root);
        if mode == "last-direction" {
            let doc = field(&graph, &graph.root, "rootDoc");
            let item = input(&mut graph, &json!({"stmt":"dir","value":"TB"}));
            append(&mut graph, id(&doc), item);
        }
        if mode == "own-undefined" || mode.starts_with("nonenumerable-") {
            let value = if mode == "nonenumerable-TB" {
                text(&mut graph, "TB")
            } else {
                Atom::Undefined
            };
            set(&mut graph, root, "direction", value);
            if mode.starts_with("nonenumerable-") {
                let key = graph
                    .strings
                    .iter()
                    .position(|s| s.is("direction"))
                    .unwrap() as u32;
                graph.nodes[root as usize]
                    .properties_mut()
                    .iter_mut()
                    .find(|p| p.key == key)
                    .unwrap()
                    .flags = 6;
            }
        }
        document.apply(document.revision(), graph).unwrap();
        let original = document.snapshot().unwrap();
        let revision = document.revision();
        let getter = document
            .read_declared_getter(&original.root, MethodId(1))
            .unwrap()
            .unwrap();
        assert!(
            matches!(getter,PublicActiveValue::String(ref s) if s.is(if mode=="last-direction"{"TB"}else{"LR"}))
        );
        assert_eq!(document.revision(), revision);
        assert_eq!(
            serde_json::to_value(document.snapshot().unwrap()).unwrap(),
            serde_json::to_value(&original).unwrap()
        );
        assert!(document
            .read_declared_getter(&original.root, MethodId(u32::MAX))
            .is_none());
        let graph = Rc::new(RefCell::new(original));
        let mut read = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
        let make_request = || {
            let settings = RenderSettings {
                max_width: Some(180.),
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
        };
        let job = document
            .prepare_detached_render(document.revision(), make_request())
            .unwrap()
            .unwrap();
        let mut resources = DirectionCallback {
            doc: &mut document,
            graph: graph.clone(),
            mode,
            fired: false,
            plain: Plain::default(),
        };
        let completed = job.run(&mut read, &mut resources);
        let trace = resources.plain.trace.clone();
        assert!(resources.fired);
        drop(resources);
        let output = document
            .finish_detached_render(completed)
            .unwrap()
            .result
            .unwrap();
        let actual = drawing(&rendered(output));
        for key in ["svg", "width", "height", "links"] {
            assert_eq!(actual[key], expected["first"][key], "{raw}/{mode}/{key}");
        }
        assert_eq!(json!(trace), expected["trace"], "{raw}/{mode}/trace");
        let mut repaired = document.snapshot().unwrap();
        let root = id(&repaired.root);
        let key = repaired
            .strings
            .iter()
            .position(|s| s.is("direction"))
            .map(|i| i as u32);
        repaired.nodes[root as usize]
            .properties_mut()
            .retain(|p| Some(p.key) != key);
        let value = text(&mut repaired, "LR");
        document.apply(document.revision(), repaired).unwrap();
        document
            .invoke(
                document.revision(),
                MethodCall {
                    method: MethodId(14),
                    receiver: Atom::Ref(root),
                    arguments: vec![value],
                },
                &mut Plain::default(),
            )
            .unwrap()
            .result
            .unwrap();
        *graph.borrow_mut() = document.snapshot().unwrap();
        let job = document
            .prepare_detached_render(document.revision(), make_request())
            .unwrap()
            .unwrap();
        let mut plain = Plain::default();
        let completed = job.run(&mut read, &mut plain);
        let output = document
            .finish_detached_render(completed)
            .unwrap()
            .result
            .unwrap();
        assert_eq!(
            drawing(&rendered(output)),
            expected["recovery"],
            "{raw}/{mode}/recovery"
        );
        assert_eq!(json!(plain.trace), expected["recoveryTrace"]);
    }
}
