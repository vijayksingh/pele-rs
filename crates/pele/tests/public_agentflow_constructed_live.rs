//! Public Agentflow normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-agentflow-constructed-live.json"
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
        host: &mut dyn RenderResources,
        _: &'static str,
    ) -> Result<Value, PublicError> {
        let Value::Atom(Atom::BuiltinFunction(method)) = function else {
            panic!("fixture native method")
        };
        let receiver = atom_value(&self.context, receiver);
        let mut arguments = Vec::new();
        for arg in args {
            arguments.push(match arg {
                LiveCallArgument::Value(v) => atom_value(&self.context, v),
                LiveCallArgument::Text(s) => {
                    active_atom(&self.context, PublicActiveOperation::Intern(s.clone()))
                }
                LiveCallArgument::Record(fields) => {
                    let fields = fields
                        .iter()
                        .map(|(k, v)| (k.clone(), atom_value(&self.context, v)))
                        .collect();
                    active_atom(&self.context, PublicActiveOperation::Record(fields))
                }
                _ => panic!("fixture argument"),
            });
        }
        let result = invoke(
            &self.context,
            MethodCall {
                method: MethodId(*method),
                receiver,
                arguments,
            },
            host,
        )?;
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
    mode: String,
    nested: RefCell<Option<Json>>,
}
fn sync(c: &Context) {
    *c.graph.borrow_mut() = c.owner.borrow().snapshot().unwrap();
}
fn active_atom(c: &Context, operation: PublicActiveOperation) -> Atom {
    let value = c
        .owner
        .borrow_mut()
        .active_operation(operation, &mut PublicActiveBudget::default())
        .unwrap();
    sync(c);
    let PublicActiveValue::Atom(atom) = value else {
        panic!("atom")
    };
    atom
}
fn atom_value(c: &Context, value: &Value) -> Atom {
    match value {
        Value::Atom(atom) => atom.clone(),
        Value::String(s) => active_atom(c, PublicActiveOperation::Intern(s.clone())),
        _ => panic!("fixture scalar"),
    }
}
fn owned_reply(c: &Context, atom: Atom) -> PublicHostValue {
    match atom {
        Atom::Undefined => PublicHostValue::Undefined,
        Atom::Null => PublicHostValue::Null,
        Atom::Bool(v) => PublicHostValue::Bool(v),
        Atom::Number(v) => PublicHostValue::Number(v),
        Atom::String(id) => PublicHostValue::String(c.graph.borrow().strings[id as usize].clone()),
        Atom::HostReference(id) => PublicHostValue::Opaque(id),
        v => PublicHostValue::Graph(v),
    }
}
struct Widths(Rc<Context>);
impl RenderResources for Widths {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.0.trace.borrow_mut().push(json!([
            "width",
            r.text,
            format!("{:016x}", r.size.to_bits())
        ]));
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn edit(c: &Context, f: impl FnOnce(&mut ModelGraph)) {
    let mut g = c.owner.borrow().snapshot().unwrap();
    f(&mut g);
    let mut d = c.owner.borrow_mut();
    let rev = d.revision();
    d.apply(rev, g).unwrap();
    drop(d);
    sync(c);
}
fn run(c: &Rc<Context>) -> Result<Json, PublicError> {
    let job = {
        let mut d = c.owner.borrow_mut();
        let rev = d.revision();
        d.prepare_detached_render(rev, request(true, false))?
            .unwrap()
    };
    let mut read = Read {
        graph: c.graph.clone(),
        reads: Rc::new(RefCell::new(vec![])),
        context: c.clone(),
    };
    let done = job.run(&mut read, &mut Widths(c.clone()));
    let result = {
        let mut d = c.owner.borrow_mut();
        d.finish_detached_render(done)?.result
    };
    sync(c);
    result.map(|v| drawing(&rendered(v)))
}
fn captured(c: &Rc<Context>) -> Json {
    match run(c) {
        Ok(mut value) => {
            value
                .as_object_mut()
                .unwrap()
                .insert("ok".into(), json!(true));
            value
        }
        Err(PublicError::HostException(HostExceptionToken(77))) => {
            json!({"ok":false,"original":true,"undefined":c.mode=="throw-undefined"})
        }
        Err(e) => panic!("unexpected {e:?}"),
    }
}
fn log(c: &Rc<Context>, arguments: Vec<Atom>) -> Result<PublicHostValue, PublicError> {
    let (level, message) = {
        let g = c.graph.borrow();
        (encoded(&g, &arguments[0]), encoded(&g, &arguments[1]))
    };
    c.trace.borrow_mut().push(json!(["log", level, message]));
    if c.fired.replace(true) {
        return Ok(PublicHostValue::Undefined);
    }
    match c.mode.as_str() {
        "baseline" => (),
        "title" => edit(c, |g| {
            let root = g.root.clone();
            set_text(g, &root, "title", "Changed title");
        }),
        "later-label" => edit(c, |g| {
            let b = map_value(g, "nodes", "B");
            set_text(g, &b, "text", "Changed Beta");
        }),
        "style-replace" => edit(c, |g| {
            let b = map_value(g, "nodes", "B");
            let style = text(g, "fill:red");
            let values = array(g, vec![style]);
            set_at(g, &b, "styles", values);
        }),
        "add-node" => add_node(c, "C", "Gamma", "task")?,
        "throw-object" | "throw-undefined" => {
            return Err(PublicError::HostException(HostExceptionToken(77)))
        }
        "nested" | "nested-throw" => {
            *c.nested.borrow_mut() = Some(captured(c));
            if c.mode == "nested-throw" {
                return Err(PublicError::HostException(HostExceptionToken(77)));
            }
        }
        _ => panic!("mode"),
    }
    Ok(PublicHostValue::Undefined)
}
fn invoke(
    c: &Rc<Context>,
    call: MethodCall,
    host: &mut dyn RenderResources,
) -> Result<Atom, PublicError> {
    let job = {
        let mut d = c.owner.borrow_mut();
        let rev = d.revision();
        d.prepare_detached_invocation(rev, call.clone())?
    };
    let Some(mut job) = job else {
        let mut d = c.owner.borrow_mut();
        let rev = d.revision();
        let result = d.invoke_read(rev, call, host)?.result;
        drop(d);
        sync(c);
        return result;
    };
    let mut reply = None;
    loop {
        let step = c
            .owner
            .borrow_mut()
            .resume_detached_invocation(&mut job, reply.take())?;
        sync(c);
        let operation = match step {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Resource(op) => {
                reply = Some(op.run(host));
                continue;
            }
            DetachedInvocationStep::Host(op) => op,
        };
        reply = Some(match operation {
            PublicHostOperation::Get {
                receiver: Atom::HostReference(1),
                key,
            } => Ok(if key == "log" {
                PublicHostValue::Opaque(2)
            } else {
                PublicHostValue::Undefined
            }),
            PublicHostOperation::Get { receiver, key } => {
                let atom = field(&c.graph.borrow(), &receiver, key);
                Ok(owned_reply(c, atom))
            }
            PublicHostOperation::Call {
                function: Atom::HostReference(2),
                arguments,
                ..
            } => log(c, arguments),
            PublicHostOperation::Truthy { value } => Ok(PublicHostValue::Bool(!matches!(
                value,
                Atom::Undefined | Atom::Null | Atom::Bool(false)
            ))),
            PublicHostOperation::ToString {
                value: Atom::String(id),
            } => Ok(PublicHostValue::String(
                c.graph.borrow().strings[id as usize].clone(),
            )),
            PublicHostOperation::GreaterEqual {
                right: Atom::Undefined,
                ..
            } => Ok(PublicHostValue::Bool(false)),
            op => panic!("unexpected operation {op:?}"),
        });
    }
    let result = c
        .owner
        .borrow_mut()
        .finish_detached_invocation(&mut job)?
        .result;
    sync(c);
    result
}
fn call(c: &Rc<Context>, method: u32, args: Vec<Atom>) -> Result<Atom, PublicError> {
    let receiver = c.graph.borrow().root.clone();
    invoke(
        c,
        MethodCall {
            method: MethodId(method),
            receiver,
            arguments: args,
        },
        &mut Widths(c.clone()),
    )
}
fn record(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    let value = Atom::Ref(id);
    for (k, v) in fields {
        set_at(g, &value, k, v);
    }
    value
}
fn add_node(c: &Rc<Context>, id: &str, label: &str, shape: &str) -> Result<(), PublicError> {
    let mut args = vec![];
    edit(c, |g| {
        let id = text(g, id);
        let label = text(g, label);
        let kind = text(g, "text");
        let label = record(g, vec![("text", label), ("type", kind)]);
        let shape = text(g, shape);
        args = vec![id, label, shape];
    });
    call(c, 3, args)?;
    Ok(())
}
fn initial(trigger: &str, mode: &str) -> Rc<Context> {
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramId::from(DiagramType::Agentflow), PrototypeId(1))
        .unwrap()
        .unwrap();
    let mut construction = factory
        .begin_construct_with(ConstructorArgument::Opaque(1))
        .unwrap();
    let mut reply = None;
    loop {
        match construction.resume(reply.take()).unwrap() {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Host(PublicHostOperation::Get { receiver, key }) => {
                reply = Some(Ok(if matches!(receiver, Atom::HostReference(1)) {
                    assert_eq!(key, "lineOffset");
                    PublicHostValue::Undefined
                } else {
                    assert_eq!(key, "options");
                    PublicHostValue::Opaque(1)
                }));
            }
            _ => panic!("unexpected construction primitive"),
        }
    }
    let owner = construction.finish().unwrap().unwrap();
    let graph = Rc::new(RefCell::new(owner.snapshot().unwrap()));
    let c = Rc::new(Context {
        owner: RefCell::new(owner),
        graph,
        slots: RefCell::new(vec![]),
        trace: RefCell::new(vec![]),
        fired: Default::default(),
        mode: mode.into(),
        nested: RefCell::new(None),
    });
    edit(&c, |g| {
        let root = g.root.clone();
        set_text(g, &root, "title", "Initial");
    });
    add_node(
        &c,
        "A",
        "Alpha",
        if trigger == "shape" { "circle" } else { "task" },
    )
    .unwrap();
    add_node(&c, "B", "Beta", "task").unwrap();
    let mut args = vec![];
    edit(&c, |g| {
        let a = text(g, "A");
        let b = text(g, "B");
        let kind = text(g, "arrow_point");
        let stroke = text(g, "normal");
        let info = record(
            g,
            vec![
                ("type", kind),
                ("stroke", stroke),
                ("length", Atom::Number(1.)),
            ],
        );
        args = vec![a, b, info];
    });
    call(&c, 4, args).unwrap();
    if trigger == "containment" {
        for (id, children, title) in [("X", ["Y", "A"], "Group X"), ("Y", ["X", "B"], "Group Y")] {
            let mut args = vec![];
            edit(&c, |g| {
                let id = text(g, id);
                let id = record(g, vec![("text", id)]);
                let values = children.iter().map(|s| text(g, s)).collect();
                let values = array(g, values);
                let title = text(g, title);
                let kind = text(g, "text");
                let title = record(g, vec![("text", title), ("type", kind)]);
                args = vec![id, values, title];
            });
            call(&c, 17, args).unwrap();
        }
    }
    assert!(!c.fired.get());
    assert!(c.trace.borrow().is_empty());
    c
}
fn state(c: &Context, old_styles: &Atom) -> Json {
    let g = c.graph.borrow();
    let root = &g.root;
    let Atom::Ref(map) = field(&g, root, "nodes") else {
        panic!()
    };
    let Node::Map { entries, .. } = &g.nodes[map as usize] else {
        panic!()
    };
    let nodes=entries.iter().map(|(k,v)|json!([encoded(&g,k),{"text":encoded(&g,&field(&g,v,"text")),"styles":encoded(&g,&field(&g,v,"styles"))}])).collect::<Vec<_>>();
    let Atom::Ref(reported) = field(&g, root, "reported") else {
        panic!("AgentFlow reported Map")
    };
    let Node::Map { entries, .. } = &g.nodes[reported as usize] else {
        panic!("AgentFlow reported Map")
    };
    json!({"title":encoded(&g,&field(&g,root,"title")),"nodes":nodes,"diagnostics":encoded(&g,&field(&g,root,"diagnostics")),"reported":entries.iter().map(|(key,value)|json!([encoded(&g,key),encoded(&g,value)])).collect::<Vec<_>>(),"oldStyles":encoded(&g,old_styles)})
}
#[test]
fn constructed_agentflow_diagnostics_cache_and_nested_recovery_match18_oracle_rows() {
    let fixture = fixture();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 18);
    for row in fixture["rows"].as_array().unwrap() {
        let trigger = row["trigger"].as_str().unwrap();
        let mode = row["mode"].as_str().unwrap();
        let c = initial(trigger, mode);
        let old_styles = {
            let g = c.graph.borrow();
            field(&g, &map_value(&g, "nodes", "B"), "styles")
        };
        let first = captured(&c);
        assert_json(&first, &row["first"], &format!("{trigger}/{mode}/first"));
        assert!(c.fired.get());
        assert_json(
            &json!(std::mem::take(&mut *c.trace.borrow_mut())),
            &row["firstTrace"],
            "trace",
        );
        assert_json(&state(&c, &old_styles), &row["after"], "state");
        assert_json(&json!(*c.nested.borrow()), &row["nested"], "nested");
        edit(&c, |g| {
            let b = map_value(g, "nodes", "B");
            set_text(g, &b, "text", "After cache");
            let Atom::Number(n) = field(g, &old_styles, "length") else {
                panic!()
            };
            let style = text(g, "stroke:blue");
            put_index(g, &old_styles, n as usize, style);
        });
        assert_json(&captured(&c), &row["recovery"], "recovery");
        assert_json(
            &json!(*c.trace.borrow()),
            &row["recoveryTrace"],
            "recovery trace",
        );
        assert_json(
            &state(&c, &old_styles),
            &row["afterRecovery"],
            "recovery state",
        );
    }
}
