//! Actual public FlowDb helper calls during both Flowchart and Swimlane rendering.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-flow-subgraph-live.json"
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
    Iterator(Rc<RefCell<(Value, usize)>>),
    Atom(Atom),
    String(JsString),
}
struct Read {
    graph: Rc<RefCell<ModelGraph>>,
    reads: Rc<RefCell<Vec<Json>>>,
    context: Rc<Context>,
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
    fn retain_value(&mut self, value: &Value) -> Result<u32, PublicError> {
        let mut slots = self.context.slots.borrow_mut();
        let id = slots.len() as u32;
        slots.push(value.clone());
        Ok(id)
    }
    fn retained_value(&mut self, id: u32) -> Result<Value, PublicError> {
        Ok(self.context.slots.borrow()[id as usize].clone())
    }
    fn iterator_begin(&mut self, value: &Value) -> Result<Value, PublicError> {
        Ok(Value::Iterator(Rc::new(RefCell::new((value.clone(), 0)))))
    }
    fn iterator_next(&mut self, it: &Value) -> Result<Option<Value>, PublicError> {
        let Value::Iterator(it) = it else {
            panic!("iterator")
        };
        let mut it = it.borrow_mut();
        let length = self.get(&it.0, "length")?;
        let n = self.number(&length).unwrap() as usize;
        if it.1 >= n {
            return Ok(None);
        }
        let value = self.index(&it.0, it.1)?;
        it.1 += 1;
        Ok(Some(value))
    }
    fn iterator_close(&mut self, _: &Value) -> Result<(), PublicError> {
        Ok(())
    }
    fn call(
        &mut self,
        function: &Value,
        receiver: &Value,
        args: &[LiveCallArgument<Value>],
        _host: &mut dyn RenderResources,
        _message: &'static str,
    ) -> Result<Value, PublicError> {
        let receiver = atom_value(&self.context, receiver);
        assert!(same(&receiver, &self.context.graph.borrow().root));
        let id = match &args[0] {
            LiveCallArgument::Text(s) => s.well_formed().unwrap(),
            LiveCallArgument::Value(Value::String(s)) => s.well_formed().unwrap(),
            _ => panic!("ID"),
        };
        let result = match function {
            Value::Atom(Atom::HostReference(id_function)) => {
                lookup(&self.context, &id, *id_function)?
            }
            Value::Atom(Atom::BuiltinFunction(19)) => native_lookup(&self.context, &id),
            _ => panic!("callable"),
        };
        sync(&self.context);
        Ok(self.own(result))
    }
    fn retained_graph_value(&mut self, node: u32) -> Result<Value, PublicError> {
        assert!((node as usize) < self.graph.borrow().nodes.len());
        Ok(Value::Atom(Atom::Ref(node)))
    }
    fn root(&self) -> Value {
        self.own(self.graph.borrow().root.clone())
    }
    fn get(&mut self, receiver: &Value, key: &str) -> Result<Value, PublicError> {
        self.reads.borrow_mut().push(json!(key));
        Ok(match receiver {
            Value::Atom(atom) => {
                let g = self.graph.borrow();
                let value = field(&g, atom, key);
                if !matches!(value, Atom::Undefined) {
                    self.own(value)
                } else if matches!(atom,Atom::Ref(id)if matches!(g.nodes[*id as usize],Node::Instance{prototype:PrototypeId(1),..}))
                {
                    self.context
                        .owner
                        .borrow()
                        .model_schema()
                        .prototypes
                        .iter()
                        .find(|p| p.id == PrototypeId(1))
                        .and_then(|p| p.members.iter().find(|m| m.name == key))
                        .and_then(|m| match m.member {
                            MemberKind::Method { method } => {
                                Some(Value::Atom(Atom::BuiltinFunction(method.0)))
                            }
                            _ => None,
                        })
                        .unwrap_or(Value::Atom(Atom::Undefined))
                } else {
                    Value::Atom(Atom::Undefined)
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
                message: JsString::from("Agentflow live read requires string"),
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
struct Context {
    owner: RefCell<PublicDocument>,
    graph: Rc<RefCell<ModelGraph>>,
    slots: RefCell<Vec<Value>>,
    trace: RefCell<Vec<Json>>,
    fired: std::cell::Cell<bool>,
    redrawn: std::cell::Cell<bool>,
    mode: String,
    raw: bool,
    nested: RefCell<Option<Json>>,
    groups: Atom,
    later: Atom,
    outer: Atom,
}
fn same(a: &Atom, b: &Atom) -> bool {
    matches!((a,b),(Atom::Ref(x),Atom::Ref(y)) if x==y)
}
fn sync(c: &Context) {
    *c.graph.borrow_mut() = c.owner.borrow().snapshot().unwrap();
}
fn active_atom(c: &Context, op: PublicActiveOperation) -> Atom {
    let v = c
        .owner
        .borrow_mut()
        .active_operation(op, &mut PublicActiveBudget::default())
        .unwrap();
    sync(c);
    let PublicActiveValue::Atom(v) = v else {
        panic!("atom")
    };
    v
}
fn atom_value(c: &Context, v: &Value) -> Atom {
    match v {
        Value::Atom(a) => a.clone(),
        Value::String(s) => active_atom(c, PublicActiveOperation::Intern(s.clone())),
        _ => panic!("scalar"),
    }
}
fn native_lookup(c: &Context, id: &str) -> Atom {
    let arg = active_atom(c, PublicActiveOperation::Intern(id.into()));
    let root = c.graph.borrow().root.clone();
    active_atom(
        c,
        PublicActiveOperation::Builtin(MethodCall {
            method: MethodId(19),
            receiver: root,
            arguments: vec![arg],
        }),
    )
}
fn edit(c: &Context, f: impl FnOnce(&mut ModelGraph)) {
    let mut graph = c.owner.borrow().snapshot().unwrap();
    f(&mut graph);
    {
        let mut d = c.owner.borrow_mut();
        let rev = d.revision();
        d.apply(rev, graph).unwrap();
    }
    sync(c);
}
fn record(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    let out = Atom::Ref(id);
    for (k, v) in fields {
        set_at(g, &out, k, v);
    }
    out
}
fn lookup(c: &Rc<Context>, id: &str, function: u32) -> Result<Atom, PublicError> {
    c.trace.borrow_mut().push(json!([
        if function == 902 {
            "replacement"
        } else {
            "subgraph"
        },
        id,
        true
    ]));
    if function == 902 {
        return Ok(native_lookup(c, id));
    }
    if !c.fired.replace(true) {
        match c.mode.as_str() {
            "label" => edit(c, |g| {
                let root = g.root.clone();
                set_text(g, &root, "title", "Changed title");
                set_text(g, &c.later, "text", "Changed Beta");
            }),
            "captured-groups" => edit(c, |g| {
                let root = g.root.clone();
                let empty = array(g, vec![]);
                set_at(g, &root, "subgraphs", empty);
                set_text(g, &c.outer, "title", "Captured outer");
            }),
            "append-member" => edit(c, |g| {
                let nodes = field(g, &c.outer, "nodes");
                let Atom::Number(n) = field(g, &nodes, "length") else {
                    panic!()
                };
                let id = text(g, "Late");
                put_index(g, &nodes, n as usize, id);
            }),
            "throw-object" | "throw-undefined" => {
                edit(c, |g| {
                    let root = g.root.clone();
                    set_text(g, &root, "title", "Saved");
                });
                return Err(PublicError::HostException(HostExceptionToken(77)));
            }
            "nested" => {
                let value = run(c, false, false)?;
                *c.nested.borrow_mut() = Some(value);
                edit(c, |g| {
                    let root = g.root.clone();
                    set_text(g, &root, "title", "After nested");
                });
            }
            _ => (),
        }
    }
    if c.mode == "false-parent" {
        return Ok(Atom::Undefined);
    }
    if c.mode == "collapsed-return" && id == "Outer" {
        let mut out = Atom::Undefined;
        edit(c, |g| {
            let view = text(g, "collapsed");
            let metadata = record(g, vec![("view", view)]);
            out = record(g, vec![("metadata", metadata)]);
        });
        return Ok(out);
    }
    Ok(native_lookup(c, id))
}
struct Widths {
    context: Rc<Context>,
    trace: bool,
}
impl RenderResources for Widths {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        if self.trace {
            self.context.trace.borrow_mut().push(json!([
                "width",
                r.text,
                format!("{:016x}", r.size.to_bits()),
                r.style
            ]));
            if self.context.mode == "redraw-replace" && !self.context.redrawn.replace(true) {
                edit(&self.context, |g| {
                    let root = g.root.clone();
                    set_at(g, &root, "subgraph", Atom::HostReference(902));
                    set_text(g, &self.context.later, "text", "After width");
                });
            }
        }
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn run(c: &Rc<Context>, responsive: bool, trace: bool) -> Result<Json, PublicError> {
    let job = {
        let mut d = c.owner.borrow_mut();
        let revision = d.revision();
        d.prepare_detached_render(revision, request(c.raw, responsive))?
            .unwrap()
    };
    let mut read = Read {
        graph: c.graph.clone(),
        reads: Rc::new(RefCell::new(vec![])),
        context: c.clone(),
    };
    let done = job.run(
        &mut read,
        &mut Widths {
            context: c.clone(),
            trace,
        },
    );
    let result = c.owner.borrow_mut().finish_detached_render(done)?.result;
    sync(c);
    result.map(|v| drawing(&rendered(v)))
}
fn source(family: &str) -> String {
    format!("{} LR\nsubgraph Outer[Outer group]\nsubgraph Inner[Inner group]\nA[Alpha] --> B[Beta]\nend\nC[Gamma]\nend\nB --> C",if family=="flowchart"{"flowchart"}else{"swimlane-beta"})
}
fn state(c: &Context) -> Json {
    let g = c.graph.borrow();
    let mut out = serde_json::Map::new();
    for key in [
        "title",
        "accTitle",
        "accDescr",
        "direction",
        "nodes",
        "edges",
        "subgraphs",
        "subgraphLookup",
    ] {
        let value = field(&g, &g.root, key);
        if matches!(value, Atom::Undefined) {
            continue;
        }
        let v = if matches!(key, "nodes" | "subgraphLookup") {
            let Atom::Ref(id) = value else { panic!() };
            let Node::Map { entries, .. } = &g.nodes[id as usize] else {
                panic!()
            };
            Json::Array(
                entries
                    .iter()
                    .map(|(k, v)| json!([encoded(&g, k), encoded(&g, v)]))
                    .collect(),
            )
        } else {
            encoded(&g, &value)
        };
        out.insert(
            if key == "subgraphLookup" {
                "lookup"
            } else {
                key
            }
            .into(),
            v,
        );
    }
    Json::Object(out)
}
#[test]
fn flowchart_and_swimlane_live_subgraph_override_phases_match80_oracle_rows() {
    let fixture = fixture();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 80);
    for row in fixture["rows"].as_array().unwrap() {
        let family = row["family"].as_str().unwrap();
        let mode = row["mode"].as_str().unwrap();
        let raw = row["raw"].as_bool().unwrap();
        let responsive = row["responsive"].as_bool().unwrap();
        let label = format!("{family}/{mode}/{raw}/{responsive}");
        let mut owner = PublicEngine::with_builtins()
            .parse(&source(family), &Default::default())
            .unwrap();
        let mut g = owner.snapshot().unwrap();
        let groups = field(&g, &g.root, "subgraphs");
        let later = map_value(&g, "nodes", "B");
        let outer = map_value(&g, "subgraphLookup", "Outer");
        let inner = map_value(&g, "subgraphLookup", "Inner");
        let view = text(&mut g, "collapsed");
        let metadata = record(&mut g, vec![("view", view)]);
        set_at(&mut g, &inner, "metadata", metadata);
        let root = g.root.clone();
        set_at(&mut g, &root, "subgraph", Atom::HostReference(901));
        owner.apply(owner.revision(), g).unwrap();
        let graph = Rc::new(RefCell::new(owner.snapshot().unwrap()));
        let c = Rc::new(Context {
            owner: RefCell::new(owner),
            graph,
            slots: RefCell::new(vec![]),
            trace: RefCell::new(vec![]),
            fired: Default::default(),
            redrawn: Default::default(),
            mode: mode.into(),
            raw,
            nested: RefCell::new(None),
            groups,
            later,
            outer,
        });
        let first = match run(&c, responsive, true) {
            Ok(mut v) => {
                v.as_object_mut().unwrap().insert("ok".into(), json!(true));
                v
            }
            Err(PublicError::HostException(HostExceptionToken(77))) => {
                json!({"ok":false,"original":true,"undefined":mode=="throw-undefined"})
            }
            Err(e) => panic!("{label}: {e:?}"),
        };
        assert_json(&first, &row["first"], &label);
        assert_json(&json!(*c.trace.borrow()), &row["trace"], &label);
        assert_json(&state(&c), &row["after"], &label);
        assert!(c.fired.get());
        assert_eq!(c.redrawn.get(), row["redrawn"].as_bool().unwrap());
        {
            let g = c.graph.borrow();
            assert_eq!(
                same(&c.groups, &field(&g, &g.root, "subgraphs")),
                row["groupsCurrent"].as_bool().unwrap()
            );
            assert_eq!(
                same(&c.later, &map_value(&g, "nodes", "B")),
                row["laterCurrent"].as_bool().unwrap()
            );
        }
        if let Some(n) = &*c.nested.borrow() {
            assert_json(n, &row["nested"], &label);
        }
        edit(&c, |g| {
            let root = g.root.clone();
            set_at(g, &root, "subgraph", Atom::BuiltinFunction(19));
        });
        assert_json(
            &run(&c, responsive, false).unwrap(),
            &row["recovery"],
            &label,
        );
        assert_json(&state(&c), &row["afterRecovery"], &label);
    }
}
