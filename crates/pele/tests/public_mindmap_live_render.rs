//! Public Mindmap callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE: &str = "mindmap\nroot((Root))\n  alpha(Alpha)\n    leaf[Leaf]\n  beta[Beta]";
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
            PeleError::new("Mindmap label requires string", ErrorCode::Semantic).into()
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
fn first(g: &ModelGraph) -> (u32, u32) {
    let nodes = id(field(g, &g.root, "nodes"));
    (nodes, id(field(g, &Atom::Ref(nodes), "0")))
}
fn state(g: &ModelGraph) -> Json {
    fn scalar(g: &ModelGraph, a: Atom) -> Json {
        match a {
            Atom::String(i) => json!(g.strings[i as usize].well_formed().unwrap()),
            Atom::Number(n) => json!(format!("{:016x}", n.to_bits())),
            Atom::Null | Atom::Undefined => Json::Null,
            _ => panic!("scalar"),
        }
    }
    let (nodes, _) = first(g);
    let Node::Array { length, .. } = g.nodes[nodes as usize] else {
        panic!()
    };
    let values=(0..length).map(|i|{
        let node=field(g,&Atom::Ref(nodes),&i.to_string());
        let children=id(field(g,&node,"children"));
        let Node::Array{length,..}=g.nodes[children as usize] else{panic!()};
        let child=(0..length).map(|i|scalar(g,field(g,&field(g,&Atom::Ref(children),&i.to_string()),"id"))).collect::<Vec<_>>();
        json!({"id":scalar(g,field(g,&node,"id")),"nodeId":scalar(g,field(g,&node,"nodeId")),"descr":scalar(g,field(g,&node,"descr")),"type":scalar(g,field(g,&node,"type")),"children":child,"class":scalar(g,field(g,&node,"class")),"icon":scalar(g,field(g,&node,"icon")),"section":scalar(g,field(g,&node,"section"))})
    }).collect::<Vec<_>>();
    json!({"title":scalar(g,field(g,&g.root,"title")),"nodes":values})
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
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.trace.push(json!(["icon", name]));
        Ok(Some(TrustedSvg::from_trusted_markup(
            "<path d=\"M0 0h1\"/>".into(),
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
    fn trigger(&mut self, kind: &str) -> Result<(), PeleError> {
        if self.fired || (self.mode.starts_with("icon-") != (kind == "icon")) {
            return Ok(());
        }
        self.fired = true;
        let mode = self.mode.strip_prefix("icon-").unwrap_or(self.mode);
        let mut g = self.doc.snapshot()?;
        let document = id(g.root.clone());
        let (nodes, root) = first(&g);
        match mode {
            "baseline" | "decorate" => {}
            "title" => set_text(&mut g, document, "title", "Changed title"),
            "current-descr" => set_text(&mut g, root, "descr", "Changed Root"),
            "later-descr" => {
                let later = id(field(&g, &Atom::Ref(nodes), "1"));
                set_text(&mut g, later, "descr", "Changed Alpha")
            }
            "type" => {
                let later = id(field(&g, &Atom::Ref(nodes), "1"));
                set(&mut g, root, "type", Atom::Number(2.));
                set(&mut g, later, "type", Atom::Number(3.));
            }
            "replace-nodes" => {
                let replacement = g.nodes.len() as u32;
                g.nodes.push(g.nodes[root as usize].clone());
                let empty = array(&mut g, vec![]);
                set(&mut g, replacement, "children", Atom::Ref(empty));
                set_text(&mut g, replacement, "descr", "Replacement root");
                let fresh = array(&mut g, vec![Atom::Ref(replacement)]);
                set(&mut g, document, "nodes", Atom::Ref(fresh));
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, document, "title", "Before failure");
                set_text(&mut g, root, "descr", "Persisted root");
            }
            "nested" => set_text(&mut g, document, "title", "Nested title"),
            _ => panic!("fixture mode"),
        }
        if mode != "baseline" && mode != "decorate" {
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if mode == "decorate" {
            let mut g = self.doc.snapshot()?;
            let arg = g.nodes.len() as u32;
            g.nodes.push(Node::Object { properties: vec![] });
            set_text(&mut g, arg, "class", "decorated");
            set_text(&mut g, arg, "icon", "decorated-icon");
            self.doc.apply(self.doc.revision(), g)?;
            let root = self.doc.snapshot()?.root;
            self.doc
                .invoke(
                    self.doc.revision(),
                    MethodCall {
                        method: MethodId(2),
                        receiver: root,
                        arguments: vec![Atom::Ref(arg)],
                    },
                    &mut Plain::default(),
                )?
                .result
                .map_err(|_| PeleError::new("decorate failed", ErrorCode::Semantic))?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if mode.starts_with("throw-") {
            return Err(PeleError::new(
                "Mindmap callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if mode == "nested" {
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
        let width = self.plain.text_width(r)?;
        self.trigger("width")?;
        Ok(width)
    }
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        let icon = self.plain.icon(name)?;
        self.trigger("icon")?;
        Ok(icon)
    }
}
#[test]
fn mindmap_live_callbacks_keep_captured_nodes_icons_helpers_errors_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-mindmap-live-render.json"
    ))
    .unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 48);
    for case in fixture["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{mode}/{raw}/{responsive}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = id(initial.root.clone());
        let (nodes, first_node) = first(&initial);
        let second = id(field(&initial, &Atom::Ref(nodes), "1"));
        set_text(&mut initial, root, "title", "Initial title");
        set_text(&mut initial, first_node, "icon", "root-icon");
        set_text(&mut initial, second, "icon", "alpha-icon");
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let old = first(&graph.borrow());
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
        assert!(callback.fired, "{context}/nonvacuous");
        assert_eq!(
            json!(callback.plain.trace),
            case["trace"],
            "{context}/trace"
        );
        assert_eq!(
            callback.nested,
            case.get("nested").cloned(),
            "{context}/nested"
        );
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
            assert_eq!(e.message, "Mindmap callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_eq!(state(&now), case["after"], "{context}/state");
        let current = first(&now);
        assert_eq!(
            json!({"nodes":old.0==current.0,"root":old.1==current.1}),
            case["identity"],
            "{context}/identity"
        );
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
        assert_eq!(
            json!(plain.trace),
            case["recoveryTrace"],
            "{context}/recovery trace"
        );
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

// Native resource extension, not an oracle limit: each retained label occurrence
// consumes text budget even when callback edits make every slot alias one node.
#[test]
fn mindmap_callback_aliased_labels_are_bounded_and_repairable() {
    struct Expand<'a> {
        doc: &'a mut PublicDocument,
        graph: Rc<RefCell<ModelGraph>>,
        fired: bool,
    }
    impl RenderResources for Expand<'_> {
        fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
            if !self.fired {
                self.fired = true;
                let mut g = self.doc.snapshot()?;
                let (nodes, root) = first(&g);
                set_text(&mut g, root, "descr", &"x".repeat(8192));
                for i in 1..2049 {
                    set(&mut g, nodes, &i.to_string(), Atom::Ref(root));
                }
                self.doc.apply(self.doc.revision(), g)?;
                *self.graph.borrow_mut() = self.doc.snapshot()?;
            }
            Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
        }
    }
    let mut doc = PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap();
    let mut g = doc.snapshot().unwrap();
    let document = id(g.root.clone());
    let (_, root) = first(&g);
    let empty = array(&mut g, vec![]);
    let mut rows = Vec::new();
    for i in 0..2049 {
        let n = g.nodes.len() as u32;
        g.nodes.push(g.nodes[root as usize].clone());
        set(&mut g, n, "id", Atom::Number(i as f64));
        set(&mut g, n, "children", Atom::Ref(empty));
        set_text(&mut g, n, "descr", "x");
        rows.push(Atom::Ref(n));
    }
    let nodes = array(&mut g, rows);
    set(&mut g, document, "nodes", Atom::Ref(nodes));
    doc.apply(doc.revision(), g).unwrap();
    let repair = doc.snapshot().unwrap();
    let expected = doc
        .render(&Default::default(), &mut Plain::default())
        .unwrap();
    let graph = Rc::new(RefCell::new(repair.clone()));
    let job = doc
        .prepare_detached_render(doc.revision(), request(false, false))
        .unwrap()
        .unwrap();
    let mut callback = Expand {
        doc: &mut doc,
        graph: graph.clone(),
        fired: false,
    };
    let done = job.run(&mut Read(graph.clone()), &mut callback);
    assert!(callback.fired);
    drop(callback);
    let PublicError::Pele(error) = doc
        .finish_detached_render(done)
        .unwrap()
        .result
        .err()
        .unwrap()
    else {
        panic!("budget error")
    };
    assert_eq!(error.code, ErrorCode::Limit);
    let after = doc.snapshot().unwrap();
    let (nodes, first) = first(&after);
    assert!(matches!(field(&after, &Atom::Ref(nodes), "2048"), Atom::Ref(found) if found == first));
    let Atom::String(text) = field(&after, &Atom::Ref(first), "descr") else {
        panic!()
    };
    assert_eq!(after.strings[text as usize].0.len(), 8192);
    doc.apply(doc.revision(), repair).unwrap();
    let recovered = doc
        .render(&Default::default(), &mut Plain::default())
        .unwrap();
    assert_eq!(drawing(&recovered.rendered), drawing(&expected.rendered));
}
