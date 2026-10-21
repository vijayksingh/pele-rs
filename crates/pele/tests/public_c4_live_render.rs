//! Public C4 callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE: &str = "C4Context\nPerson(user,\"Customer\")\nSystem_Boundary(bank,\"Bank\") {\nSystem(app,\"Banking\")\nContainerDb(store,\"Ledger\",\"SQL\")\n}\nRel(user,app,\"Uses\",\"HTTPS\")\nRel(app,store,\"Reads\")";
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
            panic!("map")
        };
        let g = self.0.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("map")
        };
        Ok(self.own(
            entries
                .iter()
                .find(|(k, _)| matches!(k,Atom::String(i) if g.strings[*i as usize].is(key)))
                .map_or(Atom::Undefined, |(_, v)| v.clone()),
        ))
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
        self.string_units(v)?
            .ok_or_else(|| PeleError::new("C4 label requires string", ErrorCode::Semantic).into())
    }
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
    let a = id(field(g, &g.root, "shapes"));
    (a, id(field(g, &Atom::Ref(a), "0")))
}
fn child(g: &ModelGraph, key: &str, index: &str) -> u32 {
    id(field(g, &field(g, &g.root, key), index))
}
fn state(g: &ModelGraph) -> Json {
    fn scalar(g: &ModelGraph, a: Atom) -> Json {
        match a {
            Atom::String(i) => json!(g.strings[i as usize].well_formed().unwrap()),
            Atom::Number(n) => json!({"bits":format!("{:016x}",n.to_bits())}),
            Atom::Null | Atom::Undefined => Json::Null,
            _ => panic!("scalar"),
        }
    }
    fn records(g: &ModelGraph, key: &str, fields: &[&str], parent: bool) -> Vec<Json> {
        let a = field(g, &g.root, key);
        let n = match field(g, &a, "length") {
            Atom::Number(n) => n as usize,
            _ => panic!(),
        };
        (0..n)
            .map(|i| {
                let v = field(g, &a, &i.to_string());
                let mut o = serde_json::Map::new();
                for k in fields {
                    o.insert((*k).into(), scalar(g, field(g, &v, k)));
                }
                if parent {
                    o.insert(
                        "parent".into(),
                        scalar(g, field(g, &field(g, &v, "parent"), "alias")),
                    );
                }
                Json::Object(o)
            })
            .collect()
    }
    let num = |key| match field(g, &g.root, key) {
        Atom::Number(n) => format!("{:016x}", n.to_bits()),
        _ => panic!(),
    };
    json!({"title":scalar(g,field(g,&g.root,"title")),"shapeInRow":num("shapeInRow"),"boundaryInRow":num("boundaryInRow"),"shapes":records(g,"shapes",&["alias","label","descr","techn","kind","shape","bgColor","fontColor","borderColor"],true),"boundaries":records(g,"boundaries",&["alias","label","type","descr","bgColor","fontColor","borderColor"],true),"rels":records(g,"rels",&["from","to","label","techn","descr","type","lineColor","textColor","offsetX","offsetY"],false)})
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
    fn trigger(&mut self, text_value: &str) -> Result<(), PeleError> {
        let selected = if self.mode.starts_with("relation-") {
            text_value == "Uses"
        } else if matches!(self.mode, "current-shape" | "later-shape") {
            text_value == "Customer"
        } else {
            true
        };
        if self.fired || !selected {
            return Ok(());
        }
        self.fired = true;
        let mode = self.mode;
        let mut g = self.doc.snapshot()?;
        let document = id(g.root.clone());
        let (shapes, first) = first(&g);
        let later = child(&g, "shapes", "1");
        let boundary = child(&g, "boundaries", "1");
        let rel = child(&g, "rels", "0");
        match mode {
            "baseline" | "style" => {}
            "helper-shadow" => set(&mut g, document, "getC4Shape", Atom::Number(0.)),
            "title" => set_text(&mut g, document, "title", "Changed title"),
            "boundary-label" => {
                set_text(&mut g, boundary, "label", "Changed Bank");
                set_text(&mut g, boundary, "descr", "Late boundary description");
            }
            "current-shape" => {
                set_text(&mut g, first, "label", "Changed customer");
                set_text(&mut g, first, "descr", "Late current description");
                set_text(&mut g, first, "kind", "System");
            }
            "later-shape" => {
                set_text(&mut g, later, "label", "Changed Banking");
                set_text(&mut g, later, "techn", "Late technology");
            }
            "relation-label" => {
                set_text(&mut g, rel, "label", "Changed relation");
                set_text(&mut g, rel, "techn", "Late protocol");
                set_text(&mut g, rel, "descr", "Late relation description");
            }
            "relation-offset" => {
                set(&mut g, rel, "offsetX", Atom::Number(13.));
                set(&mut g, rel, "offsetY", Atom::Number(17.));
                set_text(&mut g, rel, "textColor", "#234567");
            }
            "replace-shapes" => {
                let values = (0..3)
                    .rev()
                    .map(|i| field(&g, &Atom::Ref(shapes), &i.to_string()))
                    .collect();
                let a = array(&mut g, values);
                set(&mut g, document, "shapes", Atom::Ref(a));
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, document, "title", "Before failure");
                set_text(&mut g, first, "label", "Persisted customer");
            }
            "nested" => set_text(&mut g, document, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if mode != "baseline" && mode != "style" {
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if mode == "style" {
            let mut g = self.doc.snapshot()?;
            let values = ["user", "#123456", "#654321", "#abcdef"]
                .into_iter()
                .map(|s| text(&mut g, s))
                .collect();
            let a = array(&mut g, values);
            let kind = text(&mut g, "UpdateElementStyle");
            let root = g.root.clone();
            self.doc.apply(self.doc.revision(), g)?;
            self.doc
                .invoke(
                    self.doc.revision(),
                    MethodCall {
                        method: MethodId(16),
                        receiver: root,
                        arguments: vec![kind, Atom::Ref(a)],
                    },
                    &mut Plain::default(),
                )?
                .result
                .map_err(|_| PeleError::new("style failed", ErrorCode::Semantic))?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if mode.starts_with("throw-") {
            return Err(PeleError::new("C4 callback sentinel", ErrorCode::Semantic));
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
        let text = r.text.to_owned();
        let width = self.plain.text_width(r)?;
        self.trigger(&text)?;
        Ok(width)
    }
}
#[test]
fn c4_live_callbacks_keep_boundary_shape_relation_phases_helpers_errors_and_recovery() {
    let fixture: Json =
        serde_json::from_str(include_str!("../../../fixtures/public-c4-live-render.json")).unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 48);
    for case in fixture["rows"]
        .as_array()
        .unwrap()
        .iter()
        .chain(std::iter::once(&fixture["helperReplacement"]))
    {
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
        let old = first(&graph.borrow());
        let old_boundary = child(&graph.borrow(), "boundaries", "1");
        let old_rel = child(&graph.borrow(), "rels", "0");
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
            let error = completion.result.err().unwrap();
            if mode == "helper-shadow" {
                let PublicError::JavaScript {
                    name: JavaScriptErrorKind::TypeError,
                    message,
                } = error
                else {
                    panic!("helper error")
                };
                assert_eq!(
                    message,
                    JsString::from(case["first"]["error"]["message"].as_str().unwrap())
                );
            } else {
                let PublicError::Pele(e) = error else {
                    panic!("error kind")
                };
                assert_eq!(e.message, "C4 callback sentinel");
            }
        }
        assert!(!doc.has_active_detached_render());
        if mode == "helper-shadow" {
            let mut g = doc.snapshot().unwrap();
            assert!(matches!(field(&g, &g.root, "getC4Shape"), Atom::Number(n) if n == 0.));
            let root = id(g.root.clone());
            let keys = g.strings.clone();
            g.nodes[root as usize]
                .properties_mut()
                .retain(|p| !keys[p.key as usize].is("getC4Shape"));
            doc.apply(doc.revision(), g).unwrap();
        }
        let now = doc.snapshot().unwrap();
        assert_eq!(state(&now), case["after"], "{context}/state");
        let current = first(&now);
        assert_eq!(
            json!({"shapes":old.0==current.0,"first":old.1==current.1,"boundary":child(&now,"boundaries","1")==old_boundary,"relation":child(&now,"rels","0")==old_rel,"lookup":matches!({let root=read.root();let map=read.get(&root,"shapeIndex").unwrap();read.map_get_text(&map,"user").unwrap()},Val::Atom(Atom::Ref(found)) if found==old.1)}),
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
                            max_width: responsive.then_some(300.),
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
