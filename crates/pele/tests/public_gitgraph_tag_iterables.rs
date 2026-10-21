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
    Iterator(Rc<RefCell<(Val, usize, bool)>>),
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
    fn iterator_begin(&mut self, value: &Val) -> Result<Val, PublicError> {
        Ok(Val::Iterator(Rc::new(RefCell::new((
            value.clone(),
            0,
            false,
        )))))
    }
    fn iterator_next(&mut self, value: &Val) -> Result<Option<Val>, PublicError> {
        let Val::Iterator(cursor) = value else {
            panic!("iterator")
        };
        let mut cursor = cursor.borrow_mut();
        if cursor.2 {
            return Ok(None);
        }
        let index = cursor.1;
        let (value, step) = match &cursor.0 {
            Val::Text(text) => {
                if let Some(&first) = text.0.get(index) {
                    let step = if (0xd800..=0xdbff).contains(&first)
                        && text
                            .0
                            .get(index + 1)
                            .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
                    {
                        2
                    } else {
                        1
                    };
                    (
                        Some(Val::Text(JsString(text.0[index..index + step].to_vec()))),
                        step,
                    )
                } else {
                    (None, 0)
                }
            }
            Val::Atom(Atom::Ref(id)) => {
                let graph = self.0.borrow();
                match &graph.nodes[*id as usize] {
                    Node::Set { values, .. } => {
                        (values.get(index).cloned().map(|value| self.own(value)), 1)
                    }
                    Node::Array { length, .. } if index < *length as usize => (
                        Some(self.own(field(&graph, &Atom::Ref(*id), &index.to_string()))),
                        1,
                    ),
                    Node::Array { .. } => (None, 0),
                    _ => panic!("ordinary iterable"),
                }
            }
            _ => panic!("ordinary iterable"),
        };
        if value.is_some() {
            cursor.1 += step;
        } else {
            cursor.2 = true;
        }
        Ok(value)
    }
    fn iterator_close(&mut self, value: &Val) -> Result<(), PublicError> {
        let Val::Iterator(cursor) = value else {
            panic!("iterator")
        };
        cursor.borrow_mut().2 = true;
        Ok(())
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
fn set_node(g: &mut ModelGraph, strings: &[&str]) -> Atom {
    let values = strings.iter().map(|value| text(g, value)).collect();
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Set {
        values,
        properties: vec![],
    });
    Atom::Ref(id)
}
fn append_set(g: &mut ModelGraph, node: u32, value: &str) {
    let value = text(g, value);
    let Node::Set { values, .. } = &mut g.nodes[node as usize] else {
        panic!()
    };
    values.push(value);
}
fn same(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Ref(a), Atom::Ref(b)) => a == b,
        (Atom::String(a), Atom::String(b)) => g.strings[*a as usize].0 == g.strings[*b as usize].0,
        _ => false,
    }
}
fn iterated_strings(g: &ModelGraph, value: Atom) -> Vec<Json> {
    if let Atom::String(id) = value {
        return g.strings[id as usize]
            .well_formed()
            .unwrap()
            .chars()
            .map(|c| json!(c.to_string()))
            .collect();
    }
    let node = id(value.clone());
    let values = match &g.nodes[node as usize] {
        Node::Set { values, .. } => values.clone(),
        Node::Array { length, .. } => (0..*length)
            .map(|i| field(g, &value, &i.to_string()))
            .collect(),
        _ => panic!("iterable"),
    };
    values
        .into_iter()
        .map(|value| {
            let Atom::String(id) = value else {
                panic!("string")
            };
            json!(g.strings[id as usize].well_formed().unwrap())
        })
        .collect()
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
                o.insert(k.into(), json!(iterated_strings(g, field(g, c, k))));
            }
            Json::Object(o)
        })
        .collect::<Vec<_>>();
    json!({"title":scalar(g,field(g,&g.root,"title")),"direction":scalar(g,field(g,&g.root,"direction")),"lanes":iterated_strings(g,field(g,&g.root,"lanes")),"commits":commits})
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
        let selected = match self.mode {
            "initial-empty" => true,
            "initial-string" | "string-replace" => text_value == "X",
            "initial-astral" => text_value == "😀",
            _ => text_value == "v1",
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
        let tags = field(&g, &Atom::Ref(first), "tags");
        match mode {
            "initial-set" | "initial-string" | "initial-astral" | "initial-empty" => {}
            "later-set" => {
                let value = set_node(&mut g, &["Later", "Tail"]);
                set(&mut g, later, "tags", value);
            }
            "later-string" => set_text(&mut g, later, "tags", "😀Z"),
            "set-append" => append_set(&mut g, id(tags.clone()), "late-tag"),
            "set-delete-future" => {
                let remove = g.strings.iter().position(|value| value.is("v2")).unwrap() as u32;
                let Node::Set { values, .. } = &mut g.nodes[id(tags.clone()) as usize] else {
                    panic!()
                };
                values.retain(|value| !matches!(value,Atom::String(i) if *i==remove));
            }
            "set-replace" => {
                let value = set_node(&mut g, &["Replacement"]);
                set(&mut g, first, "tags", value);
            }
            "string-replace" => set_text(&mut g, first, "tags", "Replacement"),
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, document, "title", "Before failure");
                append_set(&mut g, id(tags.clone()), "Persisted");
            }
            "nested" => {
                append_set(&mut g, id(tags.clone()), "Nested tag");
                set_text(&mut g, document, "title", "Nested title");
            }
            _ => panic!("mode"),
        }
        if !mode.starts_with("initial-") {
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
fn gitgraph_tag_iterables_retain_set_and_string_receivers_with_callbacks_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-gitgraph-tag-iterables.json"
    ))
    .unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 52);
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
        let (_, initial_first) = first(&initial);
        let tags = match mode {
            "initial-string" | "string-replace" => text(&mut initial, "XY"),
            "initial-astral" => text(&mut initial, "😀Z"),
            "initial-empty" => set_node(&mut initial, &[]),
            mode if mode.starts_with("later-") => {
                field(&initial, &Atom::Ref(initial_first), "tags")
            }
            _ => set_node(&mut initial, &["v1", "v2", "v3"]),
        };
        set(&mut initial, initial_first, "tags", tags);
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let old = first(&graph.borrow());
        let old_lanes = id(field(&graph.borrow(), &graph.borrow().root, "lanes"));
        let old_tags = field(&graph.borrow(), &Atom::Ref(old.1), "tags");
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
            json!({"map":current_map==old.0,"first":matches!(lookup(&now,current_map,"a"),Atom::Ref(i) if i==old.1),"tags":same(&now,&field(&now,&Atom::Ref(old.1),"tags"),&old_tags),"lanes":id(field(&now,&now.root,"lanes"))==old_lanes}),
            case["identity"],
            "{context}/identity"
        );
        assert_eq!(
            json!(iterated_strings(&now, old_tags.clone())),
            case["originalTags"],
            "{context}/retained tags"
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
