//! Public Block callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE:&str="block\ncolumns 2\nblock:g[\"Group\"]:2\ncolumns 2\na[\"Alpha\"] b((\"Beta\"))\nend\nc<[\"Arrow\"]>(right) space:2 d{{\"Delta\"}}\na -- \"Flow\" --> d\nclassDef hot fill:#f96,color:blue\nclass a hot";
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
struct Read(Rc<RefCell<ModelGraph>>);
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
            Val::Atom(a) => self.own(field(&self.0.borrow(), a, k)),
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
            PeleError::new("Block label requires string", ErrorCode::Semantic).into()
        })
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
fn rendered(v: DetachedRenderValue) -> Rendered {
    match v {
        DetachedRenderValue::Normal(v) => v.rendered,
        DetachedRenderValue::Raw(v) => v,
    }
}
fn drawing(v: &Rendered) -> Json {
    json!({"svg":v.svg,"width":format!("{:016x}",v.width.to_bits()),"height":format!("{:016x}",v.height.to_bits())})
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
fn lookup(g: &ModelGraph, map: u32, key: &str) -> Atom {
    let Node::Map { entries, .. } = &g.nodes[map as usize] else {
        panic!()
    };
    entries
        .iter()
        .find(|(k, _)| matches!(k,Atom::String(i) if g.strings[*i as usize].is(key)))
        .map_or(Atom::Undefined, |(_, v)| v.clone())
}

#[derive(Clone, Copy)]
struct Handles {
    root: u32,
    root_children: u32,
    group: u32,
    children: u32,
    a: u32,
    b: u32,
    arrow: u32,
    edges: u32,
    edge: u32,
}
fn child(g: &ModelGraph, a: u32, key: &str) -> u32 {
    id(field(g, &Atom::Ref(a), key))
}
fn named(g: &ModelGraph, a: u32, name: &str) -> u32 {
    let Node::Array { length, .. } = g.nodes[a as usize] else {
        panic!()
    };
    (0..length).map(|i|child(g,a,&i.to_string())).find(|i|matches!(field(g,&Atom::Ref(*i),"id"),Atom::String(s) if g.strings[s as usize].is(name))).unwrap()
}
fn handles(g: &ModelGraph) -> Handles {
    let root = child(g, id(g.root.clone()), "root");
    let root_children = child(g, root, "children");
    let group = named(g, root_children, "g");
    let children = child(g, group, "children");
    let a = child(g, children, "0");
    let b = child(g, children, "1");
    let arrow = named(g, root_children, "c");
    let edges = child(g, id(g.root.clone()), "edges");
    let edge = child(g, edges, "0");
    Handles {
        root,
        root_children,
        group,
        children,
        a,
        b,
        arrow,
        edges,
        edge,
    }
}
fn object(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> u32 {
    let i = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    for (k, v) in fields {
        set(g, i, k, v);
    }
    i
}
fn leaf(g: &mut ModelGraph, name: &str, label: &str) -> u32 {
    let id = text(g, name);
    let label = text(g, label);
    let kind = text(g, "square");
    object(g, vec![("id", id), ("type", kind), ("label", label)])
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
    json!({"title":encoded(g,&field(g,&g.root,"title")),"accTitle":encoded(g,&field(g,&g.root,"accTitle")),"accDescr":encoded(g,&field(g,&g.root,"accDescr")),"root":encoded(g,&field(g,&g.root,"root")),"edges":encoded(g,&field(g,&g.root,"edges")),"classes":encoded(g,&field(g,&g.root,"classes"))})
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
    json!({"root":encoded(g,&Atom::Ref(h.root)),"rootChildren":encoded(g,&Atom::Ref(h.root_children)),"group":encoded(g,&Atom::Ref(h.group)),"children":encoded(g,&Atom::Ref(h.children)),"a":encoded(g,&Atom::Ref(h.a)),"b":encoded(g,&Atom::Ref(h.b)),"arrow":encoded(g,&Atom::Ref(h.arrow)),"edges":encoded(g,&Atom::Ref(h.edges)),"edge":encoded(g,&Atom::Ref(h.edge))})
}
fn identity(g: &ModelGraph, h: Handles) -> Json {
    let root = child(g, id(g.root.clone()), "root");
    let rc = child(g, root, "children");
    let children = child(g, h.group, "children");
    let edges = child(g, id(g.root.clone()), "edges");
    json!({"root":root==h.root,"rootChildren":rc==h.root_children,"group":includes(g,rc,h.group),"children":children==h.children,"a":includes(g,children,h.a),"edges":edges==h.edges,"edge":includes(g,edges,h.edge)})
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
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    raw: bool,
    fired: bool,
    plain: Plain,
    nested: Option<Json>,
}
impl Callback<'_> {
    fn trigger(&mut self, label: &str) -> Result<(), PeleError> {
        let target = match self.mode {
            "title-accessibility" => "Initial title",
            "arrow" => "Arrow",
            "edge-label" => "Flow",
            "baseline" | "children-replace" | "grid" | "nested" => "Group",
            _ => "Alpha",
        };
        if self.fired || label != target {
            return Ok(());
        }
        self.fired = true;
        let mut g = self.doc.snapshot()?;
        let h = handles(&g);
        let root = id(g.root.clone());
        match self.mode {
            "baseline" => {}
            "title-accessibility" => {
                set_text(&mut g, root, "title", "Changed title");
                set_text(&mut g, root, "accTitle", "Changed access");
                set_text(&mut g, root, "accDescr", "Changed description");
            }
            "children-replace" => {
                let n = leaf(&mut g, "replacement", "Replacement");
                let a = array(&mut g, vec![Atom::Ref(n)]);
                set(&mut g, h.group, "children", Atom::Ref(a));
            }
            "children-append" => {
                let n = leaf(&mut g, "appended", "Appended");
                put_index(&mut g, h.children, 2, Atom::Ref(n));
            }
            "current-node" => {
                set_text(&mut g, h.a, "label", "Changed current");
                set_text(&mut g, h.a, "id", "renamed-a");
                set_text(&mut g, h.a, "type", "diamond");
            }
            "later-node" => set_text(&mut g, h.b, "label", "Changed later"),
            "grid" => {
                set(&mut g, h.group, "columns", Atom::Number(1.));
                set(&mut g, h.group, "widthInColumns", Atom::Number(1.));
            }
            "arrow" => {
                let v = text(&mut g, "down");
                let a = array(&mut g, vec![v]);
                set(&mut g, h.arrow, "directions", Atom::Ref(a));
            }
            "edge-label" => {
                set_text(&mut g, h.edge, "label", "Changed edge");
                set_text(&mut g, h.edge, "id", "Changed edge ID");
                set_text(&mut g, h.edge, "arrowTypeEnd", "circle");
            }
            "edges-replace" => {
                let a = array(&mut g, vec![]);
                set(&mut g, root, "edges", Atom::Ref(a));
            }
            "style" => {
                let classes = child(&g, root, "classes");
                let class = id(lookup(&g, classes, "hot"));
                let a = text(&mut g, "fill:#123456");
                let b = text(&mut g, "color:#654321");
                let a = array(&mut g, vec![a, b]);
                set(&mut g, class, "styles", Atom::Ref(a));
                let name = text(&mut g, "hot");
                let a = array(&mut g, vec![name]);
                set(&mut g, h.b, "classes", Atom::Ref(a));
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, root, "title", "Before failure");
                set_text(&mut g, h.a, "label", "Saved Alpha");
            }
            "nested" => set_text(&mut g, root, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if self.mode.starts_with("throw-") {
            return Err(PeleError::new(
                "Block callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested" {
            let job = self
                .doc
                .prepare_detached_render(self.doc.revision(), request(self.raw, false))?
                .unwrap();
            let done = job.run(&mut Read(self.graph.clone()), &mut Plain::default());
            self.nested = Some(drawing(&rendered(
                self.doc.finish_detached_render(done)?.result.unwrap(),
            )));
            let mut g = self.doc.snapshot()?;
            let root = id(g.root.clone());
            set_text(&mut g, root, "title", "After nested");
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
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
fn block_live_callbacks_preserve_grid_operands_children_edges_styles_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-block-live-render.json"
    ))
    .unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 56);
    for case in fixture["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{raw}/{responsive}/{mode}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = id(initial.root.clone());
        set_text(&mut initial, root, "title", "Initial title");
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let old = handles(&graph.borrow());
        let mut read = Read(graph.clone());
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
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
        assert_eq!(callback.nested, case.get("nested").cloned(), "{context}");
        drop(callback);
        let completion = doc.finish_detached_render(done).unwrap();
        if case["first"]["ok"] == true {
            let draw = drawing(&rendered(completion.result.unwrap()));
            for key in ["svg", "width", "height"] {
                assert_eq!(draw[key], case["first"][key], "{context}/{key}");
            }
        } else {
            let PublicError::Pele(e) = completion.result.err().unwrap() else {
                panic!("error kind")
            };
            assert_eq!(e.message, "Block callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_json(&state(&now), &case["after"], &context);
        assert_json(&original(&now, old), &case["original"], &context);
        assert_json(&identity(&now, old), &case["identity"], &context);
        *graph.borrow_mut() = now;
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut plain = Plain::default();
        let done = job.run(&mut read, &mut plain);
        let recovery = drawing(&rendered(
            doc.finish_detached_render(done).unwrap().result.unwrap(),
        ));
        for key in ["svg", "width", "height"] {
            assert_eq!(
                recovery[key], case["recovery"][key],
                "{context}/recovery/{key}"
            );
        }
        assert_json(&json!(plain.trace), &case["recoveryTrace"], &context);
        if !raw {
            let normal = doc
                .render(
                    &RenderOptions {
                        render: RenderSettings {
                            max_width: responsive.then_some(220.),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    &mut Plain::default(),
                )
                .unwrap();
            assert_eq!(
                drawing(&normal.rendered),
                recovery,
                "{context}/typed geometry"
            );
        }
    }
}
