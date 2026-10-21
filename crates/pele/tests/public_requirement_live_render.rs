//! Requirement callback phases through actual public normal/raw jobs.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-requirement-live-render.json"
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
    Sequence(Vec<Value>),
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
        if let Value::Sequence(v) = receiver {
            return Ok(v
                .get(index)
                .cloned()
                .unwrap_or(Value::Atom(Atom::Undefined)));
        }
        self.get(receiver, &index.to_string())
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
            .map(|(k, v)| Value::Sequence(vec![self.own(k.clone()), self.own(v.clone())]))
            .collect())
    }
    fn array_join(&mut self, value: &Value, separator: &str) -> Result<JsString, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("Array")
        };
        let g = self.graph.borrow();
        let Node::Array { length, .. } = &g.nodes[*id as usize] else {
            panic!("Array")
        };
        let parts = (0..*length)
            .map(|i| match field(&g, &Atom::Ref(*id), &i.to_string()) {
                Atom::String(s) => g.strings[s as usize].well_formed().unwrap(),
                Atom::Undefined | Atom::Null => String::new(),
                _ => panic!("fixture string"),
            })
            .collect::<Vec<_>>();
        Ok(JsString::from(parts.join(separator).as_str()))
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
                message: JsString::from("Requirement live read requires string"),
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
            Node::Map { entries, .. } => {
                json!({"$map":entries.iter().map(|(k,v)|json!([output(g,k),output(g,v)])).collect::<Vec<_>>()})
            }
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
fn mapped(g: &ModelGraph, map: &Atom, key: &str) -> Atom {
    let Node::Map { entries, .. } = &g.nodes[id(map) as usize] else {
        panic!("map")
    };
    entries
        .iter()
        .find_map(|(k, v)| {
            matches!(k,Atom::String(i) if g.strings[*i as usize].is(key)).then(|| v.clone())
        })
        .unwrap_or(Atom::Undefined)
}
fn map(g: &mut ModelGraph, entries: Vec<(Atom, Atom)>) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Map {
        entries,
        properties: vec![],
    });
    Atom::Ref(id)
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
        let trigger = self.mode != "fields-after" || r.text == "ID";
        let width = self.plain.text_width(r)?;
        if !self.fired && trigger {
            self.fired = true;
            let mut g = self.doc.snapshot().unwrap();
            let root = id(&g.root);
            let requirements = field(&g, &g.root, "requirements");
            let alpha = mapped(&g, &requirements, "Alpha");
            let beta = mapped(&g, &requirements, "Beta");
            let elements = field(&g, &g.root, "elements");
            let element = mapped(&g, &elements, "Service");
            let relations = field(&g, &g.root, "relations");
            let relation = field(&g, &relations, "0");
            let styles = field(&g, &beta, "cssStyles");
            let classes = field(&g, &beta, "classes");
            match self.mode {
                "name-kind" => {
                    set_text(&mut g, id(&alpha), "name", "Changed Alpha");
                    set_text(&mut g, id(&beta), "name", "Changed Beta");
                    set_text(&mut g, id(&beta), "type", "Changed type");
                }
                "fields-before" => {
                    set_text(&mut g, id(&alpha), "requirementId", "Changed id");
                    set_text(&mut g, id(&alpha), "text", "Changed field");
                    set_text(&mut g, id(&alpha), "risk", "Changed risk");
                    set_text(&mut g, id(&element), "docRef", "Changed doc");
                }
                "fields-after" => {
                    set_text(&mut g, id(&alpha), "text", "After capture");
                    set_text(&mut g, id(&alpha), "risk", "After risk");
                    set_text(&mut g, id(&beta), "text", "Second changed");
                }
                "styles-replace" => {
                    let v = input(&mut g, &json!(["fill:green"]));
                    set(&mut g, id(&beta), "cssStyles", v);
                }
                "styles-mutate" => set_text(&mut g, id(&styles), "0", "fill:green"),
                "classes" => {
                    set_text(&mut g, id(&classes), "2", "after");
                    if let Node::Array { length, .. } = &mut g.nodes[id(&classes) as usize] {
                        *length = 3
                    };
                    let v = input(&mut g, &json!(["replacement"]));
                    set(&mut g, id(&beta), "classes", v);
                }
                "maps-replace" => {
                    let key = text(&mut g, "Beta");
                    let v = map(&mut g, vec![(key, beta)]);
                    set(&mut g, root, "requirements", v);
                    let v = map(&mut g, vec![]);
                    set(&mut g, root, "elements", v);
                }
                "append" => {
                    let mut extra = output(&g, &beta);
                    extra["name"] = json!("Extra");
                    extra["classes"] = json!([]);
                    extra["cssStyles"] = json!([]);
                    let extra = input(&mut g, &extra);
                    let key = text(&mut g, "Extra");
                    if let Node::Map { entries, .. } = &mut g.nodes[id(&requirements) as usize] {
                        entries.push((key, extra));
                    }
                }
                "relations" => {
                    set_text(&mut g, id(&relation), "type", "satisfies");
                    set_text(&mut g, id(&relation), "src", "Beta");
                    let v = input(
                        &mut g,
                        &json!([{"type":"contains","src":"Service","dst":"Beta"}]),
                    );
                    set(&mut g, root, "relations", v);
                }
                "direction" => set_text(&mut g, root, "direction", "TB"),
                "title-accessibility" => {
                    set_text(&mut g, root, "title", "Changed title");
                    set_text(&mut g, root, "accTitle", "Changed access");
                    set_text(&mut g, root, "accDescr", "Changed description");
                }
                "throw" => {
                    set_text(&mut g, root, "title", "Before throw");
                    set_text(&mut g, id(&alpha), "text", "Saved field");
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
                    "Requirement callback sentinel",
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
fn requirement_live_descriptors_fields_nested_callbacks_and_recovery_match_oracle() {
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
        set_text(&mut initial, root, "title", "Initial");
        set_text(&mut initial, root, "direction", "LR");
        let requirements = field(&initial, &initial.root, "requirements");
        let alpha = mapped(&initial, &requirements, "Alpha");
        let beta = mapped(&initial, &requirements, "Beta");
        let elements = field(&initial, &initial.root, "elements");
        let element = mapped(&initial, &elements, "Service");
        let relations = field(&initial, &initial.root, "relations");
        let relation = field(&initial, &relations, "0");
        let styles = input(&mut initial, &json!(["fill:red"]));
        set(&mut initial, id(&beta), "cssStyles", styles.clone());
        let classes = input(&mut initial, &json!(["default", "before"]));
        set(&mut initial, id(&beta), "classes", classes.clone());
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let reads = Rc::new(RefCell::new(vec![]));
        let mut read = Read {
            graph: graph.clone(),
            reads: reads.clone(),
        };
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .expect("selected Requirement");
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
                matches!(result,Err(PublicError::Pele(ref e)) if e.message=="Requirement callback sentinel")
            );
            assert_eq!(case["first"]["original"], true)
        } else {
            let actual = drawing(&rendered(result.unwrap()));
            for key in ["svg", "width", "height"] {
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
            ("originalRequirements", &requirements),
            ("originalAlpha", &alpha),
            ("originalBeta", &beta),
            ("originalElements", &elements),
            ("originalElement", &element),
            ("originalRelations", &relations),
            ("originalRelation", &relation),
            ("originalStyles", &styles),
            ("originalClasses", &classes),
        ] {
            assert_json(&output(&after, value), &case[name]);
        }
        for (name, old, current) in [
            (
                "requirementsCurrent",
                &requirements,
                field(&after, &after.root, "requirements"),
            ),
            (
                "relationsCurrent",
                &relations,
                field(&after, &after.root, "relations"),
            ),
            ("stylesCurrent", &styles, field(&after, &beta, "cssStyles")),
            ("classesCurrent", &classes, field(&after, &beta, "classes")),
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
