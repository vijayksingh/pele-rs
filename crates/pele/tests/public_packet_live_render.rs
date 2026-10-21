//! Public Packet callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE:&str="packet\ntitle Packet title\n0-7: \"Alpha\"\n8-15: \"Beta\"\n16-31: \"Gamma\"\n32-63: \"Delta\"";
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
            PeleError::new("Packet label requires string", ErrorCode::Semantic).into()
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
fn block(g: &mut ModelGraph, start: f64, end: f64, bits: f64, label: &str) -> u32 {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    for (k, n) in [("start", start), ("end", end), ("bits", bits)] {
        set(g, id, k, Atom::Number(n));
    }
    set_text(g, id, "label", label);
    id
}
fn first(g: &ModelGraph) -> (u32, u32, u32) {
    let rows = id(field(g, &g.root, "rows"));
    let row = id(field(g, &Atom::Ref(rows), "0"));
    let b = id(field(g, &Atom::Ref(row), "0"));
    (rows, row, b)
}
fn state(g: &ModelGraph) -> Json {
    let value = |a: Atom| match a {
        Atom::String(i) => json!(g.strings[i as usize].well_formed().unwrap()),
        Atom::Number(n) => json!(format!("{:016x}", n.to_bits())),
        _ => Json::Null,
    };
    let rows = id(field(g, &g.root, "rows"));
    let Node::Array { length, .. } = g.nodes[rows as usize] else {
        panic!()
    };
    let mut out = vec![];
    for i in 0..length {
        let row = id(field(g, &Atom::Ref(rows), &i.to_string()));
        let Node::Array { length, .. } = g.nodes[row as usize] else {
            panic!()
        };
        let mut line = vec![];
        for j in 0..length {
            let b = field(g, &Atom::Ref(row), &j.to_string());
            line.push(json!({"start":value(field(g,&b,"start")),"end":value(field(g,&b,"end")),"bits":value(field(g,&b,"bits")),"label":value(field(g,&b,"label"))}));
        }
        out.push(line);
    }
    json!({"title":value(field(g,&g.root,"title")),"accTitle":value(field(g,&g.root,"accTitle")),"accDescr":value(field(g,&g.root,"accDescr")),"bitsPerRow":value(field(g,&g.root,"bitsPerRow")),"rows":out})
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
        let alpha = r.text == "Alpha";
        let width = self.plain.text_width(r)?;
        if !self.fired
            && (!matches!(
                self.mode,
                "current-label" | "current-geometry" | "append-block"
            ) || alpha)
        {
            self.fired = true;
            let mut g = self.doc.snapshot()?;
            let root = id(g.root.clone());
            let (_, row, first) = first(&g);
            match self.mode {
                "title" => set_text(&mut g, root, "title", "Changed title"),
                "current-label" => set_text(&mut g, first, "label", "Changed Alpha"),
                "later-label" => {
                    let second = id(field(&g, &Atom::Ref(row), "1"));
                    set_text(&mut g, second, "label", "Changed Beta");
                }
                "current-geometry" => {
                    set(&mut g, first, "start", Atom::Number(1.));
                    set(&mut g, first, "end", Atom::Number(6.));
                }
                "append-block" => {
                    let b = block(&mut g, 64., 71., 8., "Added");
                    let Node::Array { length, .. } = &mut g.nodes[row as usize] else {
                        panic!()
                    };
                    let index = *length;
                    *length += 1;
                    set(&mut g, row, &index.to_string(), Atom::Ref(b));
                }
                "replace-row" => {
                    let b = block(&mut g, 0., 31., 32., "Replacement row");
                    let new = array(&mut g, vec![Atom::Ref(b)]);
                    let rows = id(field(&g, &g.root, "rows"));
                    set(&mut g, rows, "0", Atom::Ref(new));
                }
                "replace-rows" => {
                    let b = block(&mut g, 0., 15., 16., "Replacement rows");
                    let row = array(&mut g, vec![Atom::Ref(b)]);
                    let rows = array(&mut g, vec![Atom::Ref(row)]);
                    set(&mut g, root, "rows", Atom::Ref(rows));
                }
                "accessibility" => {
                    set_text(&mut g, root, "accTitle", "Changed access");
                    set_text(&mut g, root, "accDescr", "Changed description");
                }
                "throw-object" | "throw-undefined" => {
                    set_text(&mut g, root, "title", "Before failure");
                    set_text(&mut g, first, "label", "Persisted label");
                }
                "nested" => set_text(&mut g, root, "title", "Nested title"),
                "baseline" => {}
                _ => panic!("mode"),
            }
            if self.mode != "baseline" {
                self.doc.apply(self.doc.revision(), g)?;
                *self.graph.borrow_mut() = self.doc.snapshot()?;
            }
            if self.mode.starts_with("throw-") {
                return Err(PeleError::new(
                    "Packet callback sentinel",
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
fn packet_live_callbacks_keep_phase_reads_aliases_nested_errors_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-packet-live-render.json"
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
            assert_eq!(e.message, "Packet callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_eq!(state(&now), case["after"], "{context}/state");
        let (new_rows, _, new_first) = first(&now);
        assert_eq!(
            json!({"rows":new_rows==old_rows,"first":new_first==old_first}),
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
