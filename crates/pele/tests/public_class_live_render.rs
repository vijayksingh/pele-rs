//! Public Class normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE:&str="classDiagram\ndirection LR\nclass A {\n+first: string\n+second: int\n+run() bool\n}\nclass B {\n+third: string\n}\nA --> B";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-class-live-render.json"
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
fn rendered(value: DetachedRenderValue) -> Rendered {
    match value {
        DetachedRenderValue::Normal(v) => v.rendered,
        DetachedRenderValue::Raw(v) => v,
    }
}
fn drawing(value: &Rendered) -> Json {
    json!({"svg":value.svg,"width":format!("{:016x}",value.width.to_bits()),"height":format!("{:016x}",value.height.to_bits()),"links":value.links})
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
    owner: Rc<RefCell<PublicDocument>>,
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
    fn call(
        &mut self,
        function: &Value,
        receiver: &Value,
        args: &[LiveCallArgument<Value>],
        host: &mut dyn RenderResources,
        _: &'static str,
    ) -> Result<Value, PublicError> {
        assert!(args.is_empty());
        let Value::Atom(Atom::BuiltinFunction(method)) = function else {
            panic!("default fixture builtin")
        };
        let Value::Atom(receiver) = receiver else {
            panic!("receiver")
        };
        let result = {
            let mut owner = self.owner.borrow_mut();
            let revision = owner.revision();
            owner
                .invoke_read(
                    revision,
                    MethodCall {
                        method: MethodId(*method),
                        receiver: receiver.clone(),
                        arguments: vec![],
                    },
                    host,
                )?
                .result?
        };
        *self.graph.borrow_mut() = self.owner.borrow().snapshot()?;
        Ok(self.own(result))
    }
    fn root(&self) -> Value {
        self.own(self.graph.borrow().root.clone())
    }
    fn get(&mut self, receiver: &Value, key: &str) -> Result<Value, PublicError> {
        self.reads.borrow_mut().push(json!(key));
        Ok(match receiver {
            Value::Atom(atom) => {
                let out = self.owner.borrow_mut().active_operation(
                    PublicActiveOperation::Read {
                        receiver: atom.clone(),
                        key: key.into(),
                    },
                    &mut PublicActiveBudget::default(),
                )?;
                match out {
                    PublicActiveValue::Atom(mut value) => {
                        if matches!(value, Atom::Undefined) {
                            let g = self.graph.borrow();
                            if let Atom::Ref(id) = atom {
                                let node = &g.nodes[*id as usize];
                                if !node
                                    .properties()
                                    .iter()
                                    .any(|p| g.strings[p.key as usize].is(key))
                                {
                                    if let Node::Instance { prototype, .. } = node {
                                        let d = self.owner.borrow();
                                        if let Some(member) = d
                                            .model_schema()
                                            .prototype(*prototype)
                                            .and_then(|p| p.members.iter().find(|m| m.name == key))
                                        {
                                            if let MemberKind::Method { method } = member.member {
                                                value = Atom::BuiltinFunction(method.0);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        self.own(value)
                    }
                    _ => panic!("read"),
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
                message: JsString::from("Class live read requires string"),
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
        "classes",
        "relations",
        "notes",
        "namespaces",
        "interfaces",
        "styleClasses",
    ] {
        let value = field(g, &g.root, key);
        let observed = if matches!(key, "classes" | "namespaces" | "styleClasses") {
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

fn run(
    doc: Rc<RefCell<PublicDocument>>,
    graph: Rc<RefCell<ModelGraph>>,
    raw: bool,
    responsive: bool,
    host: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    let job = {
        let mut d = doc.borrow_mut();
        let revision = d.revision();
        d.prepare_detached_render(revision, request(raw, responsive))?
            .unwrap()
    };
    let done = job.run(
        &mut Read {
            graph,
            owner: doc.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        },
        host,
    );
    let mut document = doc.borrow_mut();
    let out = document.finish_detached_render(done)?;
    out.result.map(rendered)
}
struct Callback {
    doc: Rc<RefCell<PublicDocument>>,
    graph: Rc<RefCell<ModelGraph>>,
    mode: String,
    raw: bool,
    responsive: bool,
    fired: bool,
    plain: Plain,
    nested: Option<Json>,
}
impl Callback {
    fn mutate(&mut self) -> Result<(), PeleError> {
        self.fired = true;
        let mut g = self.doc.borrow().snapshot()?;
        let root = g.root.clone();
        let a = map_value(&g, "classes", "A");
        let b = map_value(&g, "classes", "B");
        let members = field(&g, &a, "members");
        let member = field(&g, &members, "0");
        let second = field(&g, &members, "1");
        match self.mode.as_str() {
            "baseline" => {}
            "title" => set_text(&mut g, &root, "title", "Changed"),
            "accessibility" => {
                set_text(&mut g, &root, "accTitle", "Changed access");
                set_text(&mut g, &root, "accDescr", "Changed description");
            }
            "later-label" => set_text(&mut g, &b, "label", "Changed beta"),
            "annotations" => {
                let list = field(&g, &a, "annotations");
                let Atom::Ref(id) = list else { panic!() };
                let Node::Array { length, .. } = g.nodes[id as usize] else {
                    panic!()
                };
                let value = text(&mut g, "Changed annotation");
                put_index(&mut g, &list, length as usize, value);
            }
            "later-member" => set_text(&mut g, &second, "id", "changed: bool"),
            "append-member" => {
                let Atom::Ref(old) = member else { panic!() };
                let n = g.nodes.len() as u32;
                let node = g.nodes[old as usize].clone();
                g.nodes.push(node);
                set_text(&mut g, &Atom::Ref(n), "id", "appended: int");
                let Atom::Ref(id) = members else { panic!() };
                let Node::Array { length, .. } = g.nodes[id as usize] else {
                    panic!()
                };
                put_index(&mut g, &members, length as usize, Atom::Ref(n));
            }
            "member-classifier" | "width-classifier" => {
                set_text(&mut g, &member, "classifier", "*")
            }
            "replace-classes" => {
                let key = text(&mut g, "B");
                let n = g.nodes.len() as u32;
                g.nodes.push(Node::Map {
                    entries: vec![(key, b)],
                    properties: vec![],
                });
                set_at(&mut g, &root, "classes", Atom::Ref(n));
            }
            "relation-label" => {
                let list = field(&g, &root, "relations");
                let edge = field(&g, &list, "0");
                set_text(&mut g, &edge, "title", "Changed edge");
            }
            "late-tooltip" => {
                set_text(&mut g, &a, "tooltip", "Changed tooltip");
                set_text(&mut g, &a, "link", "https://example.test/class");
                set_text(&mut g, &a, "linkTarget", "_blank");
            }
            "throw-object" | "throw-undefined" => set_text(&mut g, &root, "title", "Saved"),
            "nested" => set_text(&mut g, &root, "title", "Nested"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            let mut d = self.doc.borrow_mut();
            let rev = d.revision();
            d.apply(rev, g)?;
            *self.graph.borrow_mut() = d.snapshot()?;
        }
        if self.mode.starts_with("throw-") {
            return Err(PeleError::new(
                "Class callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested" {
            let out = run(
                self.doc.clone(),
                self.graph.clone(),
                self.raw,
                self.responsive,
                &mut Plain::default(),
            )
            .unwrap();
            self.nested = Some(drawing(&out));
            let mut d = self.doc.borrow_mut();
            let mut g = d.snapshot()?;
            let root = g.root.clone();
            set_text(&mut g, &root, "title", "After nested");
            let rev = d.revision();
            d.apply(rev, g)?;
            *self.graph.borrow_mut() = d.snapshot()?;
        }
        Ok(())
    }
}
impl RenderResources for Callback {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let hit = if self.mode == "width-classifier" {
            r.text.contains("first")
        } else {
            r.text == "Alpha"
        };
        let width = self.plain.text_width(r)?;
        if !self.fired && hit {
            self.mutate()?;
        }
        Ok(width)
    }
}
#[test]
fn class_owned_resources_match_sixty_declared_oracle_rows() {
    let mut count = 0;
    for case in fixture()["rows"].as_array().unwrap() {
        let mode = case["mode"].as_str().unwrap();
        if [
            "replace-display",
            "replace-classifier",
            "display-throw",
            "classifier-throw",
            "display-reentry",
        ]
        .contains(&mode)
        {
            continue;
        }
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let context = format!("{mode}/{raw}/{responsive}");
        let mut d = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut g = d.snapshot().unwrap();
        let root = g.root.clone();
        let a = map_value(&g, "classes", "A");
        let b = map_value(&g, "classes", "B");
        let members = field(&g, &a, "members");
        let member = field(&g, &members, "0");
        let second = field(&g, &members, "1");
        let classes = field(&g, &root, "classes");
        let relations = field(&g, &root, "relations");
        set_text(&mut g, &root, "title", "Initial");
        set_text(&mut g, &a, "label", "Alpha");
        set_text(&mut g, &b, "label", "Beta");
        d.apply(d.revision(), g).unwrap();
        let graph = Rc::new(RefCell::new(d.snapshot().unwrap()));
        let doc = Rc::new(RefCell::new(d));
        let mut host = Callback {
            doc: doc.clone(),
            graph: graph.clone(),
            mode: mode.into(),
            raw,
            responsive,
            fired: false,
            plain: Plain::default(),
            nested: None,
        };
        let result = run(doc.clone(), graph.clone(), raw, responsive, &mut host);
        assert!(host.fired, "{context}");
        assert_json(&json!(host.plain.trace), &case["trace"], &context);
        if case["first"]["ok"] == true {
            let actual = drawing(&result.unwrap());
            for key in ["svg", "width", "height", "links"] {
                assert_eq!(actual[key], case["first"][key], "{context}/{key}");
            }
        } else {
            assert!(
                matches!(result,Err(PublicError::Pele(e))if e.message=="Class callback sentinel"),
                "{context}"
            );
        }
        let g = doc.borrow().snapshot().unwrap();
        assert_json(&state(&g), &case["after"], &context);
        for (key, old, current) in [
            ("classes", &classes, field(&g, &g.root, "classes")),
            ("a", &a, map_value(&g, "classes", "A")),
            ("members", &members, field(&g, &a, "members")),
            ("member", &member, field(&g, &members, "0")),
            ("relations", &relations, field(&g, &g.root, "relations")),
        ] {
            assert_eq!(
                same(old, &current),
                case["aliases"][key].as_bool().unwrap(),
                "{context}/{key}"
            );
        }
        for (key, value) in [("a", a), ("b", b), ("member", member), ("second", second)] {
            assert_json(
                &encoded(&g, &value),
                &case["original"][key],
                &format!("{context}/{key}"),
            );
        }
        if let Some(nested) = host.nested {
            assert_json(&nested, &case["nested"], &context);
        }
        let recovery = run(doc, graph, raw, responsive, &mut Plain::default()).unwrap();
        let actual = drawing(&recovery);
        for key in ["svg", "width", "height", "links"] {
            assert_eq!(
                actual[key], case["recovery"][key],
                "{context}/recovery/{key}"
            );
        }
        count += 1;
    }
    assert_eq!(count, 60);
}

#[test]
fn owned_member_continuation_keeps_display_before_reentrant_classifier_and_original_errors() {
    for fail in [false, true] {
        let mut d = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut g = d.snapshot().unwrap();
        let class = map_value(&g, "classes", "A");
        let members = field(&g, &class, "members");
        let member = field(&g, &members, "0");
        set_at(&mut g, &member, "parseClassifier", Atom::HostReference(77));
        d.apply(d.revision(), g).unwrap();
        let mut job = d
            .prepare_detached_invocation(
                d.revision(),
                MethodCall {
                    method: MethodId(26),
                    receiver: member.clone(),
                    arguments: vec![],
                },
            )
            .unwrap()
            .unwrap();
        assert!(matches!(
            d.resume_detached_invocation(&mut job, None).unwrap(),
            DetachedInvocationStep::Host(PublicHostOperation::Get {
                key: "parseClassifier",
                ..
            })
        ));
        assert!(matches!(
            d.resume_detached_invocation(
                &mut job,
                Some(Ok(PublicHostValue::Graph(Atom::HostReference(77))))
            )
            .unwrap(),
            DetachedInvocationStep::Host(PublicHostOperation::Call {
                function: Atom::HostReference(77),
                ..
            })
        ));
        let mut budget = PublicActiveBudget::default();
        let PublicActiveValue::Atom(changed) = d
            .active_operation(
                PublicActiveOperation::Intern("changed after display capture".into()),
                &mut budget,
            )
            .unwrap()
        else {
            panic!()
        };
        d.active_operation(
            PublicActiveOperation::Set {
                receiver: member.clone(),
                key: "id".into(),
                value: changed,
            },
            &mut budget,
        )
        .unwrap();
        let reply = if fail {
            Err(PublicError::HostException(HostExceptionToken(91)))
        } else {
            Ok(PublicHostValue::String("font-style:italic;".into()))
        };
        assert!(matches!(
            d.resume_detached_invocation(&mut job, Some(reply)).unwrap(),
            DetachedInvocationStep::Complete
        ));
        let out = d.finish_detached_invocation(&mut job).unwrap();
        let result = out.result;
        drop(out.graph);
        if fail {
            assert!(matches!(
                result,
                Err(PublicError::HostException(HostExceptionToken(91)))
            ));
        } else {
            let details = result.unwrap();
            let g = d.snapshot().unwrap();
            assert_eq!(
                encoded(&g, &field(&g, &details, "displayText")),
                json!("+first: string")
            );
            assert_eq!(
                encoded(&g, &field(&g, &details, "cssStyle")),
                json!("font-style:italic;")
            );
        }
        let g = d.snapshot().unwrap();
        assert_eq!(
            encoded(&g, &field(&g, &member, "id")),
            json!("changed after display capture")
        );
    }
}
