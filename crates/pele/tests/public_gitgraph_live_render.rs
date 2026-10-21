//! Public GitGraph callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE:&str="gitGraph\ncommit id:\"a\" msg:\"Alpha\" tag:\"v1\"\nbranch feature\ncommit id:\"b\" type:HIGHLIGHT\ncheckout main\ncommit id:\"c\"\nmerge feature id:\"m\"";
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
            PeleError::new("GitGraph label requires string", ErrorCode::Semantic).into()
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
fn first(g: &ModelGraph) -> (u32, u32) {
    let m = id(field(g, &g.root, "commits"));
    (m, id(lookup(g, m, "a")))
}
fn state(g: &ModelGraph) -> Json {
    fn scalar(g: &ModelGraph, a: Atom) -> Json {
        match a {
            Atom::String(i) => json!(g.strings[i as usize].well_formed().unwrap()),
            Atom::Number(n) => json!(format!("{:016x}", n.to_bits())),
            Atom::Bool(b) => json!(b),
            Atom::Null | Atom::Undefined => Json::Null,
            _ => panic!("scalar"),
        }
    }
    fn strings(g: &ModelGraph, a: Atom) -> Vec<Json> {
        let n = match field(g, &a, "length") {
            Atom::Number(n) => n as usize,
            _ => panic!(),
        };
        (0..n)
            .map(|i| scalar(g, field(g, &a, &i.to_string())))
            .collect()
    }
    let m = id(field(g, &g.root, "commits"));
    let Node::Map { entries, .. } = &g.nodes[m as usize] else {
        panic!()
    };
    let commits = entries
        .iter()
        .map(|(key, c)| {
            let mut o = serde_json::Map::new();
            o.insert("key".into(), scalar(g, key.clone()));
            for k in [
                "id",
                "message",
                "seq",
                "type",
                "customType",
                "customId",
                "branch",
            ] {
                o.insert(k.into(), scalar(g, field(g, c, k)));
            }
            for k in ["tags", "parents"] {
                o.insert(k.into(), json!(strings(g, field(g, c, k))));
            }
            Json::Object(o)
        })
        .collect::<Vec<_>>();
    json!({"title":scalar(g,field(g,&g.root,"title")),"direction":scalar(g,field(g,&g.root,"direction")),"lanes":strings(g,field(g,&g.root,"lanes")),"commits":commits})
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
        let selected = if matches!(self.mode, "current-tag" | "replace-map" | "type") {
            text_value == "v1"
        } else if matches!(self.mode, "current-id" | "later-id" | "parents") {
            text_value == "a"
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
        let (map, first) = first(&g);
        let later = id(lookup(&g, map, "b"));
        let tags = id(field(&g, &Atom::Ref(first), "tags"));
        match mode {
            "baseline" => {}
            "title" => set_text(&mut g, document, "title", "Changed title"),
            "lanes" => {
                let lanes = id(field(&g, &g.root, "lanes"));
                let values = (0..2)
                    .rev()
                    .map(|i| field(&g, &Atom::Ref(lanes), &i.to_string()))
                    .collect();
                let a = array(&mut g, values);
                set(&mut g, document, "lanes", Atom::Ref(a));
            }
            "current-tag" => {
                set_text(&mut g, tags, "0", "Changed tag");
                set_text(&mut g, tags, "1", "late-tag");
                if let Node::Array { length, .. } = &mut g.nodes[tags as usize] {
                    *length = 2;
                }
            }
            "current-id" => set_text(&mut g, first, "id", "renamed-a"),
            "later-id" => set_text(&mut g, later, "id", "renamed-b"),
            "parents" => {
                let a = array(&mut g, vec![]);
                set(&mut g, later, "parents", Atom::Ref(a));
            }
            "replace-map" => {
                let a = g.nodes.len() as u32;
                g.nodes.push(Node::Map {
                    entries: vec![],
                    properties: vec![],
                });
                set(&mut g, document, "commits", Atom::Ref(a));
            }
            "type" => {
                set(&mut g, first, "type", Atom::Number(2.));
                set(&mut g, later, "customType", Atom::Number(1.));
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, document, "title", "Before failure");
                set_text(&mut g, first, "message", "Persisted message");
            }
            "nested" => set_text(&mut g, document, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if mode != "baseline" {
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if mode.starts_with("throw-") {
            return Err(PeleError::new(
                "GitGraph callback sentinel",
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
        let text = r.text.to_owned();
        let width = self.plain.text_width(r)?;
        self.trigger(&text)?;
        Ok(width)
    }
}
#[test]
fn gitgraph_live_callbacks_keep_captured_commits_tags_late_identity_errors_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-gitgraph-live-render.json"
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
        let old = first(&graph.borrow());
        let old_lanes = id(field(&graph.borrow(), &graph.borrow().root, "lanes"));
        let old_tags = id(field(&graph.borrow(), &Atom::Ref(old.1), "tags"));
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
            assert_eq!(e.message, "GitGraph callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_eq!(state(&now), case["after"], "{context}/state");
        let current_map = id(field(&now, &now.root, "commits"));
        assert_eq!(
            json!({"map":current_map==old.0,"first":matches!(lookup(&now,current_map,"a"),Atom::Ref(i) if i==old.1),"tags":id(field(&now,&Atom::Ref(old.1),"tags"))==old_tags,"lanes":id(field(&now,&now.root,"lanes"))==old_lanes}),
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
