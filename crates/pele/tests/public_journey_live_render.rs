//! Public Journey callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE: &str = "journey\ntitle Journey title\nsection First\nAlpha: 5: Ada, Bob\nBeta with an exceptionally long task label: 2: Bob\nsection Last\nGamma: 3: Ada";
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
            PeleError::new("Journey label requires string", ErrorCode::Semantic).into()
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
fn task_node(g: &mut ModelGraph, label: &str, score: f64, people: &[&str], section: &str) -> u32 {
    let values = people.iter().map(|s| text(g, s)).collect();
    let people = array(g, values);
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    set_text(g, id, "task", label);
    set(g, id, "score", Atom::Number(score));
    set(g, id, "people", Atom::Ref(people));
    set_text(g, id, "section", section);
    id
}
fn first(g: &ModelGraph) -> (u32, u32) {
    let tasks = id(field(g, &g.root, "tasks"));
    let task = id(field(g, &Atom::Ref(tasks), "0"));
    (tasks, task)
}
fn state(g: &ModelGraph) -> Json {
    fn scalar(g: &ModelGraph, a: Atom) -> Json {
        match a {
            Atom::String(i) => json!(g.strings[i as usize].well_formed().unwrap()),
            Atom::Number(n) => json!(format!("{:016x}", n.to_bits())),
            _ => Json::Null,
        }
    }
    let tasks = id(field(g, &g.root, "tasks"));
    let Node::Array { length, .. } = g.nodes[tasks as usize] else {
        panic!()
    };
    let tasks=(0..length).map(|i|{let task=field(g,&Atom::Ref(tasks),&i.to_string());let people=field(g,&task,"people");let Node::Array{length,..}=g.nodes[id(people.clone()) as usize]else{panic!()};let people=(0..length).map(|i|scalar(g,field(g,&people,&i.to_string()))).collect::<Vec<_>>();json!({"task":scalar(g,field(g,&task,"task")),"score":scalar(g,field(g,&task,"score")),"people":people,"section":scalar(g,field(g,&task,"section"))})}).collect::<Vec<_>>();
    json!({"title":scalar(g,field(g,&g.root,"title")),"accTitle":scalar(g,field(g,&g.root,"accTitle")),"accDescr":scalar(g,field(g,&g.root,"accDescr")),"tasks":tasks})
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
        let width = self.plain.text_width(r)?;
        if !self.fired {
            self.fired = true;
            let mut g = self.doc.snapshot()?;
            let document = id(g.root.clone());
            let (tasks, task) = first(&g);
            match self.mode {
                "title" => set_text(&mut g, document, "title", "Changed title"),
                "current-task" => set_text(&mut g, task, "task", "Changed Alpha"),
                "later-task" => {
                    let second = id(field(&g, &Atom::Ref(tasks), "1"));
                    set_text(&mut g, second, "task", "Changed Beta")
                }
                "score" => {
                    set(&mut g, task, "score", Atom::Number(1.));
                    let second = id(field(&g, &Atom::Ref(tasks), "1"));
                    set(&mut g, second, "score", Atom::Number(f64::NAN));
                }
                "people" => {
                    let a = text(&mut g, "New actor");
                    let b = text(&mut g, "Ada");
                    let people = array(&mut g, vec![a, b]);
                    set(&mut g, task, "people", Atom::Ref(people));
                }
                "section" => set_text(&mut g, task, "section", "Changed section"),
                "replace-tasks" => {
                    let task = task_node(
                        &mut g,
                        "Replacement task",
                        4.,
                        &["Fresh actor"],
                        "Fresh section",
                    );
                    let tasks = array(&mut g, vec![Atom::Ref(task)]);
                    set(&mut g, document, "tasks", Atom::Ref(tasks));
                }
                "accessibility" => {
                    set_text(&mut g, document, "accTitle", "Changed access");
                    set_text(&mut g, document, "accDescr", "Changed description")
                }
                "throw-object" | "throw-undefined" => {
                    set_text(&mut g, document, "title", "Before failure");
                    set_text(&mut g, task, "task", "Persisted task")
                }
                "nested" => set_text(&mut g, document, "title", "Nested title"),
                "baseline" => {}
                _ => panic!("mode"),
            }
            if self.mode != "baseline" {
                self.doc.apply(self.doc.revision(), g)?;
                *self.graph.borrow_mut() = self.doc.snapshot()?;
            }
            if self.mode.starts_with("throw-") {
                return Err(PeleError::new(
                    "Journey callback sentinel",
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
fn journey_live_callbacks_keep_phase_reads_aliases_nested_errors_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-journey-live-render.json"
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
        let (old_rows, old_first) = first(&graph.borrow());
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
            assert_eq!(e.message, "Journey callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_eq!(state(&now), case["after"], "{context}/state");
        let (new_rows, new_first) = first(&now);
        assert_eq!(
            json!({"tasks":new_rows==old_rows,"first":new_first==old_first}),
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

// Native resource extension, not a claim that the JavaScript oracle has this
// limit: callbacks cannot multiply one admitted catalog string into unbounded
// retained actor/people text. Partial writes remain observable and repairable.
#[test]
fn journey_callback_text_budget_retains_partial_writes_and_recovers() {
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
                let (_, task) = first(&g);
                let name = text(&mut g, &"x".repeat(8192));
                let people = array(&mut g, vec![name; 2049]);
                set(&mut g, task, "people", Atom::Ref(people));
                self.doc.apply(self.doc.revision(), g)?;
                *self.graph.borrow_mut() = self.doc.snapshot()?;
            }
            Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
        }
    }
    let mut doc = PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap();
    let initial = doc.snapshot().unwrap();
    let (_, task) = first(&initial);
    let original = field(&initial, &Atom::Ref(task), "people");
    let graph = Rc::new(RefCell::new(initial));
    let mut read = Read(graph.clone());
    let job = doc
        .prepare_detached_render(doc.revision(), request(false, false))
        .unwrap()
        .unwrap();
    let mut host = Expand {
        doc: &mut doc,
        graph: graph.clone(),
        fired: false,
    };
    let done = job.run(&mut read, &mut host);
    assert!(host.fired);
    drop(host);
    let completion = doc.finish_detached_render(done).unwrap();
    let PublicError::Pele(error) = completion.result.err().unwrap() else {
        panic!("native limit")
    };
    assert_eq!(error.code, ErrorCode::Limit);
    assert!(!doc.has_active_detached_render());
    let mut after = doc.snapshot().unwrap();
    let people = id(field(&after, &Atom::Ref(task), "people"));
    assert!(matches!(
        after.nodes[people as usize],
        Node::Array { length: 2049, .. }
    ));
    set(&mut after, task, "people", original);
    doc.apply(doc.revision(), after).unwrap();
    *graph.borrow_mut() = doc.snapshot().unwrap();
    let job = doc
        .prepare_detached_render(doc.revision(), request(false, false))
        .unwrap()
        .unwrap();
    let done = job.run(&mut read, &mut Plain::default());
    let draw = drawing(&rendered(
        doc.finish_detached_render(done).unwrap().result.unwrap(),
    ));
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-journey-live-render.json"
    ))
    .unwrap();
    let expected = fixture["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["raw"] == false && r["responsive"] == false && r["mode"] == "baseline")
        .unwrap();
    for key in ["svg", "width", "height"] {
        assert_eq!(draw[key], expected["first"][key]);
    }
}
