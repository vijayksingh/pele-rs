//! Public Ishikawa callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE: &str = "ishikawa\nRoot\n  Category A\n    Cause A\n  Category B\n    Cause B";
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
            PeleError::new("Ishikawa label requires string", ErrorCode::Semantic).into()
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
fn node(g: &mut ModelGraph, label: &str, children: Vec<Atom>) -> u32 {
    let children = array(g, children);
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    set_text(g, id, "text", label);
    set(g, id, "children", Atom::Ref(children));
    id
}
fn first(g: &ModelGraph) -> (u32, u32, u32) {
    let root = id(field(g, &g.root, "root"));
    let children = id(field(g, &Atom::Ref(root), "children"));
    let category = id(field(g, &Atom::Ref(children), "0"));
    (root, children, category)
}
fn state(g: &ModelGraph) -> Json {
    fn tree(g: &ModelGraph, root: Atom) -> Json {
        let text = field(g, &root, "text");
        let Atom::String(text) = text else {
            panic!("text")
        };
        let children = id(field(g, &root, "children"));
        let Node::Array { length, .. } = g.nodes[children as usize] else {
            panic!()
        };
        let children = (0..length)
            .map(|i| tree(g, field(g, &Atom::Ref(children), &i.to_string())))
            .collect::<Vec<_>>();
        json!({"text":g.strings[text as usize].well_formed().unwrap(),"children":children})
    }
    let Atom::String(title) = field(g, &g.root, "title") else {
        panic!("title")
    };
    json!({"title":g.strings[title as usize].well_formed().unwrap(),"root":tree(g,field(g,&g.root,"root"))})
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
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let category_label = r.text == "Category A";
        let width = self.plain.text_width(r)?;
        if !self.fired
            && (!matches!(
                self.mode,
                "current-text"
                    | "later-text"
                    | "append-child"
                    | "replace-child"
                    | "replace-children"
            ) || category_label)
        {
            self.fired = true;
            let mut g = self.doc.snapshot()?;
            let document = id(g.root.clone());
            let (_root, children, category) = first(&g);
            match self.mode {
                "title" => set_text(&mut g, document, "title", "Changed title"),
                "current-text" => set_text(&mut g, category, "text", "Changed A"),
                "later-text" => {
                    let second = id(field(&g, &Atom::Ref(children), "1"));
                    set_text(&mut g, second, "text", "Changed B");
                }
                "append-child" => {
                    let n = node(&mut g, "Added cause", vec![]);
                    let children = id(field(&g, &Atom::Ref(category), "children"));
                    let Node::Array { length, .. } = &mut g.nodes[children as usize] else {
                        panic!()
                    };
                    let index = *length;
                    *length += 1;
                    set(&mut g, children, &index.to_string(), Atom::Ref(n));
                }
                "replace-child" => {
                    let n = node(&mut g, "Replacement category", vec![]);
                    set(&mut g, children, "1", Atom::Ref(n));
                }
                "replace-children" => {
                    let n = node(&mut g, "Replacement cause", vec![]);
                    let array = array(&mut g, vec![Atom::Ref(n)]);
                    set(&mut g, category, "children", Atom::Ref(array));
                }
                "replace-root" => {
                    let n = node(&mut g, "Fresh category", vec![]);
                    let root = node(&mut g, "Replacement root", vec![Atom::Ref(n)]);
                    set(&mut g, document, "root", Atom::Ref(root));
                }
                "throw-object" | "throw-undefined" => {
                    set_text(&mut g, document, "title", "Before failure");
                    set_text(&mut g, category, "text", "Persisted category");
                }
                "nested" => set_text(&mut g, document, "title", "Nested title"),
                "baseline" | "helper" => {}
                _ => panic!("mode"),
            }
            if self.mode != "baseline" && self.mode != "helper" {
                self.doc.apply(self.doc.revision(), g)?;
                *self.graph.borrow_mut() = self.doc.snapshot()?;
            }
            if self.mode == "helper" {
                let mut g = self.doc.snapshot()?;
                let name = text(&mut g, "Helper cause");
                self.doc.apply(self.doc.revision(), g)?;
                let root = self.doc.snapshot()?.root;
                let out = self.doc.invoke(
                    self.doc.revision(),
                    MethodCall {
                        method: MethodId(0),
                        receiver: root,
                        arguments: vec![Atom::Number(4.), name],
                    },
                    &mut Plain::default(),
                )?;
                out.result
                    .map_err(|_| PeleError::new("Helper failed", ErrorCode::Semantic))?;
                *self.graph.borrow_mut() = self.doc.snapshot()?;
            }
            if self.mode.starts_with("throw-") {
                return Err(PeleError::new(
                    "Ishikawa callback sentinel",
                    ErrorCode::Semantic,
                ));
            }
            if self.mode == "nested" {
                let job = self
                    .doc
                    .prepare_detached_render(self.doc.revision(), request(true, false))?
                    .unwrap();
                let done = job.run(&mut Read(self.graph.clone()), &mut Plain::default());
                let result = self.doc.finish_detached_render(done)?.result.unwrap();
                self.nested = Some(drawing(&rendered(result)));
                let mut g = self.doc.snapshot()?;
                let root = id(g.root.clone());
                set_text(&mut g, root, "title", "After nested");
                self.doc.apply(self.doc.revision(), g)?;
                *self.graph.borrow_mut() = self.doc.snapshot()?;
            }
        }
        Ok(width)
    }
}
#[test]
fn ishikawa_live_callbacks_keep_phase_reads_aliases_nested_errors_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-ishikawa-live-render.json"
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
        set_text(&mut initial, root, "title", "Initial title");
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let (old_rows, _, old_first) = first(&graph.borrow());
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut read = Read(graph.clone());
        let mut callback = Callback {
            doc: &mut doc,
            graph: graph.clone(),
            mode,
            fired: false,
            plain: Plain::default(),
            nested: None,
        };
        let done = job.run(&mut read, &mut callback);
        assert_eq!(
            json!(callback.plain.trace),
            case["trace"],
            "{context}/trace"
        );
        assert_eq!(json!(callback.fired), case["fired"], "{context}/activation");
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
            assert_eq!(e.message, "Ishikawa callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_eq!(state(&now), case["after"], "{context}/state");
        let (new_rows, _, new_first) = first(&now);
        assert_eq!(
            json!({"root":new_rows==old_rows,"category":new_first==old_first}),
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
        let completion = doc.finish_detached_render(done).unwrap();
        let recovery = drawing(&rendered(completion.result.unwrap()));
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
