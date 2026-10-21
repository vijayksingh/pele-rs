//! Public Gantt callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE:&str="gantt\n title Initial title\n dateFormat YYYY-MM-DD\n section Main\n Alpha :a,2024-01-01,2d\n Beta :b,after a,2d\n section Later\n Gamma :c,after b,1d";
fn field(g: &ModelGraph, a: &Atom, k: &str) -> Atom {
    let Atom::Ref(id) = a else {
        return Atom::Undefined;
    };
    if let Node::Array { length, .. } = &g.nodes[*id as usize] {
        if k == "length" {
            return Atom::Number(*length as f64);
        }
    }
    g.nodes[*id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(k))
        .map_or(Atom::Undefined, |p| p.value.clone())
}
fn id(a: Atom) -> u32 {
    let Atom::Ref(id) = a else {
        panic!("expected ref")
    };
    id
}
#[derive(Clone)]
enum Val {
    Atom(Atom),
    Text(JsString),
    Copy(Arc<Vec<(String, Val)>>),
}
struct Read(Rc<RefCell<ModelGraph>>, Rc<RefCell<PublicDocument>>);
impl Read {
    fn own(&self, a: Atom) -> Val {
        if let Atom::String(i) = a {
            Val::Text(self.0.borrow().strings[i as usize].clone())
        } else {
            Val::Atom(a)
        }
    }
}
impl LiveRenderRead for Read {
    type Value = Val;
    fn is_array(&mut self, v: &Val) -> Result<bool, PublicError> {
        Ok(
            matches!(v,Val::Atom(Atom::Ref(i)) if matches!(self.0.borrow().nodes[*i as usize],Node::Array{..})),
        )
    }
    fn to_number(&mut self, v: &Val, _: &mut dyn RenderResources) -> Result<f64, PublicError> {
        Ok(match v {
            Val::Atom(Atom::Number(n)) => *n,
            Val::Atom(Atom::Undefined) => f64::NAN,
            Val::Atom(Atom::Null) => 0.,
            Val::Atom(Atom::Bool(v)) => {
                if *v {
                    1.
                } else {
                    0.
                }
            }
            Val::Atom(Atom::Ref(id)) => match self.0.borrow().nodes[*id as usize] {
                Node::Date { milliseconds, .. } => milliseconds,
                _ => panic!("fixture Date"),
            },
            _ => panic!("fixture number"),
        })
    }
    fn map_get(&mut self, v: &Val, key: &Val) -> Result<Val, PublicError> {
        let Val::Text(key) = key else { panic!("key") };
        self.map_get_text(v, &key.well_formed().unwrap())
    }
    fn call(
        &mut self,
        f: &Val,
        receiver: &Val,
        args: &[LiveCallArgument<Val>],
        host: &mut dyn RenderResources,
        _: &'static str,
    ) -> Result<Val, PublicError> {
        let Val::Atom(Atom::BuiltinFunction(method)) = f else {
            panic!("builtin")
        };
        let Val::Atom(receiver) = receiver else {
            panic!("receiver")
        };
        let mut arguments = Vec::new();
        for arg in args {
            arguments.push(match arg {
                LiveCallArgument::Number(n) => Atom::Number(*n),
                LiveCallArgument::Date(n) => {
                    let mut owner = self.1.borrow_mut();
                    let PublicActiveValue::Atom(v) = owner.active_operation(
                        PublicActiveOperation::Date(*n),
                        &mut PublicActiveBudget::default(),
                    )?
                    else {
                        panic!()
                    };
                    v
                }
                LiveCallArgument::Text(s) => {
                    let mut owner = self.1.borrow_mut();
                    let PublicActiveValue::Atom(v) = owner.active_operation(
                        PublicActiveOperation::Intern(s.clone()),
                        &mut PublicActiveBudget::default(),
                    )?
                    else {
                        panic!()
                    };
                    v
                }
                LiveCallArgument::Value(Val::Atom(a)) => a.clone(),
                _ => panic!("fixture argument"),
            });
        }
        let mut job = {
            let mut owner = self.1.borrow_mut();
            let revision = owner.revision();
            owner
                .prepare_detached_invocation(
                    revision,
                    MethodCall {
                        method: MethodId(*method),
                        receiver: receiver.clone(),
                        arguments,
                    },
                )?
                .expect("owned scheduler")
        };
        let mut reply = None;
        loop {
            let step = {
                let mut owner = self.1.borrow_mut();
                owner.resume_detached_invocation(&mut job, reply.take())?
            };
            *self.0.borrow_mut() = self.1.borrow().snapshot()?;
            match step {
                DetachedInvocationStep::Complete => break,
                DetachedInvocationStep::Resource(op) => reply = Some(op.run(host)),
                DetachedInvocationStep::Host(_) => panic!("fixture has no configured warn"),
            }
        }
        let result = self
            .1
            .borrow_mut()
            .finish_detached_invocation(&mut job)?
            .result?;
        *self.0.borrow_mut() = self.1.borrow().snapshot()?;
        Ok(self.own(result))
    }

    fn object_identity(&mut self, value: &Val) -> Result<Option<u64>, PublicError> {
        Ok(match value {
            Val::Atom(Atom::Ref(id)) => Some(u64::from(*id)),
            Val::Atom(Atom::BuiltinFunction(id)) => Some((1u64 << 32) + u64::from(*id)),
            _ => None,
        })
    }
    fn root(&self) -> Val {
        self.own(self.0.borrow().root.clone())
    }
    fn get(&mut self, v: &Val, k: &str) -> Result<Val, PublicError> {
        Ok(match v {
            Val::Atom(a) => {
                let mut value = field(&self.0.borrow(), a, k);
                if matches!(value, Atom::Undefined) {
                    let g = self.0.borrow();
                    if let Atom::Ref(id) = a {
                        let node = &g.nodes[*id as usize];
                        if !node
                            .properties()
                            .iter()
                            .any(|p| g.strings[p.key as usize].is(k))
                        {
                            if let Node::Instance { prototype, .. } = node {
                                if let Some(member) = self
                                    .1
                                    .borrow()
                                    .model_schema()
                                    .prototype(*prototype)
                                    .and_then(|p| p.members.iter().find(|m| m.name == k))
                                {
                                    if let MemberKind::Method { method } = member.member {
                                        value = Atom::BuiltinFunction(method.0)
                                    }
                                }
                            }
                        }
                    }
                }
                self.own(value)
            }
            Val::Copy(p) => p
                .iter()
                .find(|(key, _)| key == k)
                .map_or(Val::Atom(Atom::Undefined), |(_, v)| v.clone()),
            _ => Val::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, v: &Val, i: usize) -> Result<Val, PublicError> {
        self.get(v, &i.to_string())
    }
    fn has_index(&mut self, v: &Val, i: usize) -> Result<bool, PublicError> {
        let k = i.to_string();
        Ok(match v {
            Val::Atom(Atom::Ref(id)) => {
                let g = self.0.borrow();
                g.nodes[*id as usize]
                    .properties()
                    .iter()
                    .any(|p| g.strings[p.key as usize].is(&k))
            }
            Val::Copy(p) => p.iter().any(|(key, _)| key == &k),
            _ => false,
        })
    }
    fn map_get_text(&mut self, v: &Val, key: &str) -> Result<Val, PublicError> {
        let Val::Atom(Atom::Ref(id)) = v else {
            panic!("Map")
        };
        let g = self.0.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("Map")
        };
        Ok(entries
            .iter()
            .find_map(|(k, v)| match k {
                Atom::String(i) if g.strings[*i as usize].is(key) => Some(self.own(v.clone())),
                _ => None,
            })
            .unwrap_or(Val::Atom(Atom::Undefined)))
    }
    fn array_join(&mut self, v: &Val, separator: &str) -> Result<JsString, PublicError> {
        let Val::Atom(Atom::Ref(id)) = v else {
            panic!("Array")
        };
        let g = self.0.borrow();
        let Node::Array { length, .. } = &g.nodes[*id as usize] else {
            panic!("Array")
        };
        let values = (0..*length)
            .map(|i| match field(&g, &Atom::Ref(*id), &i.to_string()) {
                Atom::String(i) => g.strings[i as usize].well_formed().unwrap(),
                Atom::Undefined | Atom::Null => String::new(),
                _ => panic!("fixture string"),
            })
            .collect::<Vec<_>>();
        Ok(JsString::from(values.join(separator).as_str()))
    }
    fn snapshot_map_values(&mut self, v: &Val, max: usize) -> Result<Vec<Val>, PublicError> {
        let Val::Atom(Atom::Ref(id)) = v else {
            panic!()
        };
        let g = self.0.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!()
        };
        assert!(entries.len() <= max);
        Ok(entries.iter().map(|(_, v)| self.own(v.clone())).collect())
    }
    fn shallow_copy(&mut self, v: &Val) -> Result<Val, PublicError> {
        Ok(match v {
            Val::Atom(Atom::Ref(id)) => {
                let g = self.0.borrow();
                Val::Copy(Arc::new(
                    g.nodes[*id as usize]
                        .properties()
                        .iter()
                        .filter(|p| p.flags & 1 != 0)
                        .map(|p| {
                            (
                                g.strings[p.key as usize].well_formed().unwrap(),
                                self.own(p.value.clone()),
                            )
                        })
                        .collect(),
                ))
            }
            Val::Copy(p) => Val::Copy(p.clone()),
            _ => Val::Copy(Arc::new(vec![])),
        })
    }
    fn kind(&self, v: &Val) -> LiveValueKind {
        match v {
            Val::Text(_) => LiveValueKind::String,
            Val::Atom(Atom::BuiltinFunction(_)) => LiveValueKind::Function,
            Val::Atom(Atom::Undefined) => LiveValueKind::Undefined,
            Val::Atom(Atom::Null) => LiveValueKind::Null,
            Val::Atom(Atom::Bool(_)) => LiveValueKind::Boolean,
            Val::Atom(Atom::Number(_)) => LiveValueKind::Number,
            _ => LiveValueKind::Object,
        }
    }
    fn number(&self, v: &Val) -> Option<f64> {
        if let Val::Atom(Atom::Number(n)) = v {
            Some(*n)
        } else {
            None
        }
    }
    fn truthy(&self, v: &Val) -> bool {
        match v {
            Val::Atom(Atom::Undefined | Atom::Null) => false,
            Val::Atom(Atom::Bool(b)) => *b,
            Val::Atom(Atom::Number(n)) => *n != 0. && !n.is_nan(),
            Val::Text(s) => !s.0.is_empty(),
            _ => true,
        }
    }
    fn string_units(&self, v: &Val) -> Result<Option<JsString>, PublicError> {
        Ok(if let Val::Text(s) = v {
            Some(s.clone())
        } else {
            None
        })
    }
    fn to_string(&mut self, v: &Val) -> Result<JsString, PublicError> {
        Ok(match v {
            Val::Text(s) => s.clone(),
            Val::Atom(Atom::Number(n)) => JsString::from(live_number_string(*n).as_str()),
            _ => panic!("fixture conversion"),
        })
    }
    fn require_string(&mut self, v: &Val, _: LiveStringUse) -> Result<JsString, PublicError> {
        self.string_units(v)?.ok_or_else(|| {
            PeleError::new("Gantt label requires string", ErrorCode::Semantic).into()
        })
    }
}
fn config() -> Object {
    Object::default()
}
fn request(raw: bool, responsive: bool) -> DetachedRenderRequest {
    let settings = RenderSettings {
        max_width: responsive.then_some(320.),
        now: Some(1704283200000.),
        ..Default::default()
    };
    if raw {
        DetachedRenderRequest::Raw {
            config: config(),
            settings,
        }
    } else {
        DetachedRenderRequest::Normal {
            options: RenderOptions {
                parse: ParseOptions {
                    preparation: PrepareOptions {
                        config: config(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                render: settings,
            },
            limit: None,
            enforce_native_bytes: true,
        }
    }
}
fn prepare(
    doc: &Rc<RefCell<PublicDocument>>,
    raw: bool,
    responsive: bool,
) -> Result<DetachedRenderJob, PeleError> {
    let mut d = doc.borrow_mut();
    let rev = d.revision();
    Ok(d.prepare_detached_render(rev, request(raw, responsive))?
        .unwrap())
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
fn text(g: &mut ModelGraph, s: &str) -> Atom {
    let i = g.strings.len() as u32;
    g.strings.push(JsString::from(s));
    Atom::String(i)
}
fn set(g: &mut ModelGraph, object: u32, k: &str, v: Atom) {
    let key = if let Some(i) = g.strings.iter().position(|s| s.is(k)) {
        i as u32
    } else {
        let i = g.strings.len() as u32;
        g.strings.push(JsString::from(k));
        i
    };
    let props = g.nodes[object as usize].properties_mut();
    if let Some(p) = props.iter_mut().find(|p| p.key == key) {
        p.value = v;
    } else {
        props.push(Property {
            key,
            value: v,
            flags: 7,
        });
    }
}
fn set_text(g: &mut ModelGraph, object: u32, k: &str, v: &str) {
    let v = text(g, v);
    set(g, object, k, v)
}
fn array(g: &mut ModelGraph, values: Vec<Atom>) -> u32 {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Array {
        length: values.len() as u32,
        properties: vec![],
    });
    for (i, v) in values.into_iter().enumerate() {
        set(g, id, &i.to_string(), v);
    }
    id
}
#[derive(Clone, Copy)]
struct Handles {
    tasks: u32,
    a: u32,
    b: u32,
    c: u32,
}
fn child(g: &ModelGraph, a: u32, k: &str) -> u32 {
    id(field(g, &Atom::Ref(a), k))
}
fn handles(g: &ModelGraph) -> Handles {
    let root = id(g.root.clone());
    let tasks = child(g, root, "tasks");
    Handles {
        tasks,
        a: child(g, tasks, "0"),
        b: child(g, tasks, "1"),
        c: child(g, tasks, "2"),
    }
}
fn date(g: &mut ModelGraph, ms: f64) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Date {
        milliseconds: ms,
        properties: vec![],
    });
    Atom::Ref(id)
}
fn date_field(g: &mut ModelGraph, id: u32, key: &str, ms: f64) {
    let value = date(g, ms);
    set(g, id, key, value)
}
fn cloned_task(g: &mut ModelGraph, original: u32, id_text: &str, label: &str) -> u32 {
    let id = g.nodes.len() as u32;
    g.nodes.push(g.nodes[original as usize].clone());
    set_text(g, id, "id", id_text);
    set_text(g, id, "task", label);
    let classes = child(g, original, "classes");
    let new_classes = g.nodes.len() as u32;
    g.nodes.push(g.nodes[classes as usize].clone());
    set(g, id, "classes", Atom::Ref(new_classes));
    id
}
fn put_index(g: &mut ModelGraph, a: u32, index: u32, v: Atom) {
    set(g, a, &index.to_string(), v);
    let Node::Array { length, .. } = &mut g.nodes[a as usize] else {
        panic!()
    };
    *length = (*length).max(index + 1);
}
fn encoded(g: &ModelGraph, a: &Atom) -> Json {
    match a {
        Atom::Undefined | Atom::Null => Json::Null,
        Atom::Bool(v) => json!(v),
        Atom::Number(v) => json!(v),
        Atom::String(i) => json!(g.strings[*i as usize].well_formed().unwrap()),
        Atom::Ref(i) => match &g.nodes[*i as usize] {
            Node::Array { length, .. } => json!((0..*length)
                .map(|n| encoded(g, &field(g, a, &n.to_string())))
                .collect::<Vec<_>>()),
            Node::Date { milliseconds, .. } => {
                if !milliseconds.is_finite() {
                    Json::Null
                } else {
                    let fields = Plain::default()
                        .calendar_fields(*milliseconds)
                        .unwrap()
                        .unwrap()
                        .civil;
                    json!(format!(
                        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                        fields.year,
                        fields.month,
                        fields.day,
                        fields.hour,
                        fields.minute,
                        fields.second,
                        fields.millisecond
                    ))
                }
            }
            Node::Map { entries, .. } => json!(entries
                .iter()
                .map(|(k, v)| vec![encoded(g, k), encoded(g, v)])
                .collect::<Vec<_>>()),
            n => Json::Object(
                n.properties()
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
        _ => panic!("fixture value"),
    }
}
fn state(g: &ModelGraph) -> Json {
    let mut out = serde_json::Map::new();
    for k in [
        "title",
        "accTitle",
        "accDescr",
        "displayMode",
        "topAxis",
        "axisFormat",
        "dateFormat",
        "tickInterval",
        "todayMarker",
        "tasks",
        "sections",
        "links",
        "includes",
        "excludes",
        "work",
        "now",
    ] {
        let v = field(g, &g.root, k);
        if !matches!(v, Atom::Undefined)
            || ["title", "accTitle", "accDescr", "tickInterval"].contains(&k)
        {
            out.insert(k.into(), encoded(g, &v));
        }
    }
    Json::Object(out)
}
fn same(a: &Atom, b: u32) -> bool {
    matches!(a,Atom::Ref(i) if *i==b)
}
fn includes(g: &ModelGraph, array: u32, object: u32) -> bool {
    let Node::Array { length, .. } = g.nodes[array as usize] else {
        panic!()
    };
    (0..length).any(|i| same(&field(g, &Atom::Ref(array), &i.to_string()), object))
}
fn original(g: &ModelGraph, h: Handles) -> Json {
    json!({"tasks":encoded(g,&Atom::Ref(h.tasks)),"a":encoded(g,&Atom::Ref(h.a)),"b":encoded(g,&Atom::Ref(h.b)),"c":encoded(g,&Atom::Ref(h.c))})
}
fn identity(g: &ModelGraph, h: Handles) -> Json {
    let tasks = child(g, id(g.root.clone()), "tasks");
    json!({"tasks":tasks==h.tasks,"a":includes(g,tasks,h.a),"b":includes(g,tasks,h.b),"c":includes(g,tasks,h.c)})
}
fn assert_json(a: &Json, b: &Json, context: &str) {
    match (a, b) {
        (Json::Number(a), Json::Number(b)) => assert_eq!(
            a.as_f64().unwrap().to_bits(),
            b.as_f64().unwrap().to_bits(),
            "{context}"
        ),
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{context}");
            for (a, b) in a.iter().zip(b) {
                assert_json(a, b, context)
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{context}"
            );
            for (k, a) in a {
                assert_json(a, &b[k], context)
            }
        }
        _ => assert_eq!(a, b, "{context}"),
    }
}
#[derive(Default)]
struct Plain {
    trace: Vec<Json>,
}
impl RenderResources for Plain {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.trace.push(json!([
            "width",
            r.text,
            format!("{:016x}", r.size.to_bits()),
            r.style
        ]));
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
struct Callback<'a> {
    doc: Rc<RefCell<PublicDocument>>,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    raw: bool,
    responsive: bool,
    fired: bool,
    plain: Plain,
    nested: Option<Json>,
}
impl Callback<'_> {
    fn trigger(&mut self, label: &str) -> Result<(), PeleError> {
        if self.fired || label != "Alpha" {
            return Ok(());
        }
        self.fired = true;
        let mut g = self.doc.borrow().snapshot()?;
        let h = handles(&g);
        let root = id(g.root.clone());
        match self.mode {
            "baseline" => {}
            "current-label" => set_text(&mut g, h.a, "task", "Changed current"),
            "next-label" => set_text(&mut g, h.b, "task", "Changed later"),
            "dates" => {
                date_field(&mut g, h.a, "startTime", 1704153600000.);
                date_field(&mut g, h.a, "endTime", 1704326400000.);
                date_field(&mut g, h.b, "endTime", 1704585600000.);
            }
            "render-end" => date_field(&mut g, h.a, "renderEndTime", 1704153600000.),
            "flags" => {
                for key in ["active", "crit", "milestone"] {
                    set(&mut g, h.a, key, Atom::Bool(true));
                }
                set(&mut g, h.b, "done", Atom::Bool(true));
            }
            "classes" => {
                let classes = child(&g, h.a, "classes");
                let n = match g.nodes[classes as usize] {
                    Node::Array { length, .. } => length,
                    _ => panic!(),
                };
                let text = text(&mut g, "clickable");
                put_index(&mut g, classes, n, text);
            }
            "links" => {
                let links = child(&g, root, "links");
                let k = text(&mut g, "a");
                let v = text(&mut g, "https://example.com/alpha");
                let Node::Map { entries, .. } = &mut g.nodes[links as usize] else {
                    panic!()
                };
                entries.push((k, v));
            }
            "tasks-replace" => {
                let t = cloned_task(&mut g, h.b, "replacement", "Replacement");
                let a = array(&mut g, vec![Atom::Ref(t)]);
                set(&mut g, root, "tasks", Atom::Ref(a));
            }
            "tasks-append" => {
                let t = cloned_task(&mut g, h.c, "appended", "Appended");
                set(&mut g, t, "order", Atom::Number(3.));
                put_index(&mut g, h.tasks, 3, Atom::Ref(t));
            }
            "section" => {
                set_text(&mut g, h.a, "section", "Changed current section");
                set_text(&mut g, h.b, "section", "Changed next section");
            }
            "axis" => {
                set_text(&mut g, root, "axisFormat", "%d");
                set_text(&mut g, root, "tickInterval", "1day");
            }
            "title-accessibility" => {
                set_text(&mut g, root, "title", "Changed title");
                set_text(&mut g, root, "accTitle", "Changed access");
                set_text(&mut g, root, "accDescr", "Changed description");
            }
            "today" => set_text(&mut g, root, "todayMarker", "off"),
            "exclusions" => {
                let d = text(&mut g, "2024-01-02");
                let a = array(&mut g, vec![d.clone()]);
                set(&mut g, root, "excludes", Atom::Ref(a));
                let set_id = g.nodes.len() as u32;
                g.nodes.push(Node::Set {
                    values: vec![d],
                    properties: vec![],
                });
                set(&mut g, root, "excludeSet", Atom::Ref(set_id));
                set(&mut g, root, "datedTokens", Atom::Bool(true));
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, root, "title", "Before failure");
                set_text(&mut g, h.b, "task", "Before failure task");
            }
            "nested" => set_text(&mut g, root, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            {
                let mut d = self.doc.borrow_mut();
                let rev = d.revision();
                d.apply(rev, g)?;
            }
            *self.graph.borrow_mut() = self.doc.borrow().snapshot()?;
        }
        if self.mode.starts_with("throw-") {
            return Err(PeleError::new(
                "Gantt callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested" {
            let job = prepare(&self.doc, self.raw, self.responsive)?;
            let done = job.run(
                &mut Read(self.graph.clone(), self.doc.clone()),
                &mut Plain::default(),
            );
            self.nested = Some(drawing(&rendered(
                self.doc
                    .borrow_mut()
                    .finish_detached_render(done)?
                    .result
                    .unwrap(),
            )));
            let mut g = self.doc.borrow().snapshot()?;
            let root = id(g.root.clone());
            set_text(&mut g, root, "title", "After nested");
            {
                let mut d = self.doc.borrow_mut();
                let rev = d.revision();
                d.apply(rev, g)?;
            }
            *self.graph.borrow_mut() = self.doc.borrow().snapshot()?;
        }
        Ok(())
    }
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let label = r.text.to_owned();
        let width = self.plain.text_width(r)?;
        self.trigger(&label)?;
        Ok(width)
    }
}
#[test]
fn gantt_live_callbacks_preserve_schedule_task_handles_dates_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-gantt-live-render.json"
    ))
    .unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 72);
    for case in fixture["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{raw}/{responsive}/{mode}");
        let doc = Rc::new(RefCell::new(
            PublicEngine::with_builtins()
                .parse(SOURCE, &Default::default())
                .unwrap(),
        ));
        if responsive {
            let mut g = doc.borrow().snapshot().unwrap();
            let root = id(g.root.clone());
            set_text(&mut g, root, "displayMode", "compact");
            set(&mut g, root, "topAxis", Atom::Bool(true));
            let mut d = doc.borrow_mut();
            let rev = d.revision();
            d.apply(rev, g).unwrap();
        }
        let graph = Rc::new(RefCell::new(doc.borrow().snapshot().unwrap()));
        let old = handles(&graph.borrow());
        let mut read = Read(graph.clone(), doc.clone());
        let job = prepare(&doc, raw, responsive).unwrap();
        let mut callback = Callback {
            doc: doc.clone(),
            graph: graph.clone(),
            mode,
            raw,
            responsive,
            fired: false,
            plain: Plain::default(),
            nested: None,
        };
        let done = job.run(&mut read, &mut callback);
        assert!(callback.fired, "{context}");
        assert_json(&json!(callback.plain.trace), &case["trace"], &context);
        assert_eq!(callback.nested, case.get("nested").cloned(), "{context}");
        drop(callback);
        let result = {
            let mut document = doc.borrow_mut();
            let completion = document.finish_detached_render(done).unwrap();
            completion.result
        };
        if case["first"]["ok"] == true {
            let draw = drawing(&rendered(result.unwrap()));
            for key in ["svg", "width", "height", "links"] {
                assert_json(&draw[key], &case["first"][key], &format!("{context}/{key}"));
            }
        } else {
            let PublicError::Pele(e) = result.err().unwrap() else {
                panic!("error kind")
            };
            assert_eq!(e.message, "Gantt callback sentinel");
        }
        assert!(!doc.borrow().has_active_detached_render());
        let now = doc.borrow().snapshot().unwrap();
        assert_json(&state(&now), &case["after"], &context);
        assert_json(&original(&now, old), &case["original"], &context);
        assert_json(&identity(&now, old), &case["identity"], &context);
        *graph.borrow_mut() = now;
        let job = prepare(&doc, raw, responsive).unwrap();
        let mut plain = Plain::default();
        let done = job.run(&mut read, &mut plain);
        let recovery = drawing(&rendered(
            doc.borrow_mut()
                .finish_detached_render(done)
                .unwrap()
                .result
                .unwrap(),
        ));
        for key in ["svg", "width", "height", "links"] {
            assert_json(
                &recovery[key],
                &case["recovery"][key],
                &format!("{context}/recovery/{key}"),
            );
        }
        assert_json(&json!(plain.trace), &case["recoveryTrace"], &context);
    }
}
