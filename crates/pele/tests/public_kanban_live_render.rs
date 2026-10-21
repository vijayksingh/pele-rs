//! Kanban callback phases through actual public normal/raw jobs.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-kanban-live-render.json"
    ))
    .unwrap()
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
    json!({"svg":v.svg,"width":format!("{:016x}",v.width.to_bits()),"height":format!("{:016x}",v.height.to_bits()),"links":v.links})
}
fn field(graph: &ModelGraph, receiver: &Atom, key: &str) -> Atom {
    if let Atom::Ref(id) = receiver {
        if key == "length" {
            if let Node::Array { length, .. } = &graph.nodes[*id as usize] {
                return Atom::Number(f64::from(*length));
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
    Atom(Atom),
    String(JsString),
}
struct Read {
    graph: Rc<RefCell<ModelGraph>>,
    reads: Rc<RefCell<Vec<Json>>>,
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
    fn root(&self) -> Value {
        self.own(self.graph.borrow().root.clone())
    }
    fn get(&mut self, receiver: &Value, key: &str) -> Result<Value, PublicError> {
        self.reads.borrow_mut().push(json!(key));
        Ok(match receiver {
            Value::Atom(atom) => self.own(field(&self.graph.borrow(), atom, key)),
            _ => Value::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, receiver: &Value, index: usize) -> Result<Value, PublicError> {
        self.get(receiver, &index.to_string())
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
        self.require_string(
            value,
            LiveStringUse {
                method: "includes",
                expression_receiver: "e",
            },
        )
    }
    fn require_string(&mut self, value: &Value, _: LiveStringUse) -> Result<JsString, PublicError> {
        self.string_units(value)?
            .ok_or_else(|| PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString::from("Kanban live read requires string"),
            })
    }
}

fn id(v: &Atom) -> u32 {
    if let Atom::Ref(id) = v {
        *id
    } else {
        panic!("reference")
    }
}
fn set(g: &mut ModelGraph, node: u32, key: &str, value: Atom) {
    let keyid = if let Some(id) = g.strings.iter().position(|s| s.is(key)) {
        id as u32
    } else {
        let id = g.strings.len() as u32;
        g.strings.push(JsString::from(key));
        id
    };
    // String IDs are history entries; equal property names may use different IDs.
    if let Some(index) = g.nodes[node as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key))
    {
        g.nodes[node as usize].properties_mut()[index].value = value;
    } else {
        g.nodes[node as usize].properties_mut().push(Property {
            key: keyid,
            value,
            flags: 7,
        });
    }
}
fn append(g: &mut ModelGraph, node: u32, value: Atom) {
    let Node::Array { length, .. } = g.nodes[node as usize] else {
        panic!("array")
    };
    set(g, node, &length.to_string(), value);
    if let Node::Array { length, .. } = &mut g.nodes[node as usize] {
        *length += 1;
    }
}
fn text(g: &mut ModelGraph, value: &str) -> Atom {
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(value));
    Atom::String(id)
}
fn set_text(g: &mut ModelGraph, node: u32, key: &str, value: &str) {
    let v = text(g, value);
    set(g, node, key, v);
}
fn input(g: &mut ModelGraph, v: &Json) -> Atom {
    match v {
        Json::Null => Atom::Null,
        Json::Bool(v) => Atom::Bool(*v),
        Json::Number(v) => Atom::Number(v.as_f64().unwrap()),
        Json::String(v) => text(g, v),
        Json::Array(values) => {
            let mut properties = vec![];
            for (i, v) in values.iter().enumerate() {
                let Atom::String(key) = text(g, &i.to_string()) else {
                    panic!()
                };
                properties.push(Property {
                    key,
                    value: input(g, v),
                    flags: 7,
                });
            }
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Array {
                length: values.len() as u32,
                properties,
            });
            Atom::Ref(id)
        }
        Json::Object(values) => {
            let mut properties = vec![];
            for (key, v) in values {
                let Atom::String(key) = text(g, key) else {
                    panic!()
                };
                properties.push(Property {
                    key,
                    value: input(g, v),
                    flags: 7,
                });
            }
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Object { properties });
            Atom::Ref(id)
        }
    }
}
fn output(g: &ModelGraph, v: &Atom) -> Json {
    match v {
        Atom::Undefined | Atom::Null => Json::Null,
        Atom::Bool(v) => json!(v),
        Atom::Number(v) => json!(v),
        Atom::String(id) => json!(g.strings[*id as usize].well_formed().unwrap()),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { length, .. } => (0..*length)
                .map(|i| output(g, &field(g, v, &i.to_string())))
                .collect(),
            node => node
                .properties()
                .iter()
                .map(|p| {
                    (
                        g.strings[p.key as usize].well_formed().unwrap(),
                        output(g, &p.value),
                    )
                })
                .collect(),
        },
        _ => panic!("fixture atom"),
    }
}
fn state(g: &ModelGraph) -> Json {
    ["title", "sections"]
        .into_iter()
        .map(|k| (k.to_string(), output(g, &field(g, &g.root, k))))
        .collect()
}
fn assert_json(a: &Json, b: &Json) {
    match (a, b) {
        (Json::Number(a), Json::Number(b)) => {
            assert_eq!(a.as_f64().unwrap().to_bits(), b.as_f64().unwrap().to_bits())
        }
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                assert_json(a, b);
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
            for (k, a) in a {
                assert_json(a, &b[k]);
            }
        }
        _ => assert_eq!(a, b),
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
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.trace.push(json!(["icon", name]));
        Ok(Some(TrustedSvg::from_trusted_markup(
            "<path d=\"M0 0h4v4z\"/>".into(),
        )))
    }
}

struct Callback<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    fired: bool,
    plain: Plain,
    nested: Option<Json>,
    icons: usize,
    raw: bool,
}
impl Callback<'_> {
    fn mutate(&mut self) -> Result<(), PeleError> {
        self.fired = true;
        let mut g = self.doc.snapshot().unwrap();
        let root = id(&g.root);
        let sections = id(&field(&g, &g.root, "sections"));
        let section = id(&field(&g, &Atom::Ref(sections), "0"));
        let items = id(&field(&g, &Atom::Ref(section), "items"));
        let first = id(&field(&g, &Atom::Ref(items), "0"));
        match self.mode {
            "header-edit" => {
                set_text(&mut g, section, "label", "Changed header");
                set_text(&mut g, section, "id", "changed-header");
            }
            "item-edit" => {
                let second = id(&field(&g, &Atom::Ref(items), "1"));
                set_text(&mut g, first, "label", "Changed first");
                set_text(&mut g, second, "label", "Changed second");
            }
            "item-replace" => {
                let v = input(
                    &mut g,
                    &json!({"id":"replacement","level":1,"label":"Replacement"}),
                );
                set(&mut g, items, "1", v);
            }
            "items-replace" => {
                let v = input(
                    &mut g,
                    &json!([{"id":"replacement","level":1,"label":"Replacement"}]),
                );
                set(&mut g, section, "items", v);
            }
            "sections-replace" => {
                let v = input(
                    &mut g,
                    &json!([{"id":"new-root","level":0,"label":"New root","items":[{"id":"new","level":1,"label":"New"}]}]),
                );
                set(&mut g, root, "sections", v);
            }
            "item-append" => {
                let v = input(&mut g, &json!({"id":"append","level":1,"label":"Appended"}));
                append(&mut g, items, v);
            }
            "section-append" => {
                let v = input(
                    &mut g,
                    &json!({"id":"append-section","level":0,"label":"Appended section","items":[{"id":"append","level":1,"label":"Appended"}]}),
                );
                append(&mut g, sections, v);
            }
            "section-replace" => {
                let v = input(
                    &mut g,
                    &json!({"id":"replacement-section","level":0,"label":"Replacement section","items":[{"id":"replacement","level":1,"label":"Replacement"}]}),
                );
                set(&mut g, sections, "1", v);
            }
            "late-fields" => {
                for (k, v) in [
                    ("id", "late-id"),
                    ("priority", "high"),
                    ("cssClasses", "late-card"),
                    ("ticket", "Changed ticket"),
                    ("assigned", "Grace"),
                ] {
                    set_text(&mut g, first, k, v)
                }
                set_text(&mut g, section, "cssClasses", "late-column");
                set_text(&mut g, root, "title", "Late title");
            }
            "throw" => {
                set_text(&mut g, root, "title", "Before throw");
                set_text(&mut g, first, "label", "Saved label");
            }
            "nested" => set_text(&mut g, root, "title", "Nested title"),
            "icon-label" => {
                set_text(&mut g, first, "label", "After icon");
                set_text(&mut g, first, "ticket", "After icon ticket");
            }
            "icon-paint" => {
                set_text(&mut g, first, "id", "paint-id");
                set_text(&mut g, first, "cssClasses", "paint-card");
                set_text(&mut g, first, "ticket", "Paint ticket");
                let v = input(&mut g, &json!([]));
                set(&mut g, section, "items", v);
            }
            "icon-throw" => set_text(&mut g, first, "label", "Saved icon label"),
            "baseline" => {}
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            self.doc.apply(self.doc.revision(), g).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        if matches!(self.mode, "throw" | "icon-throw") {
            return Err(PeleError::new(
                "Kanban callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested" {
            let job = self
                .doc
                .prepare_detached_render(self.doc.revision(), request(self.raw, false))
                .unwrap()
                .unwrap();
            let mut reader = Read {
                graph: self.graph.clone(),
                reads: Rc::new(RefCell::new(vec![])),
            };
            let done = job.run(&mut reader, &mut Plain::default());
            let result = self
                .doc
                .finish_detached_render(done)
                .unwrap()
                .result
                .unwrap();
            self.nested = Some(drawing(&rendered(result)));
            let mut g = self.doc.snapshot().unwrap();
            set_text(&mut g, root, "title", "After nested");
            self.doc.apply(self.doc.revision(), g).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        Ok(())
    }
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let trigger = match self.mode {
            "item-append" => r.text == "First task",
            "late-fields" => r.text == "Initial",
            _ => true,
        };
        let width = self.plain.text_width(r)?;
        if !self.fired && !self.mode.starts_with("icon-") && trigger {
            self.mutate()?
        }
        Ok(width)
    }
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        let value = self.plain.icon(name)?;
        self.icons += 1;
        if !self.fired
            && self.mode.starts_with("icon-")
            && ((self.mode == "icon-paint" && self.icons == 3)
                || (self.mode != "icon-paint" && name == "item"))
        {
            self.mutate()?
        }
        Ok(value)
    }
}
#[test]
fn kanban_live_phases_preserve_captured_arrays_late_fields_errors_and_recovery() {
    let fixture = fixture();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 54);
    assert_eq!(
        fixture["oracleSHA256"],
        "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d"
    );
    for case in fixture["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let mut doc = PublicEngine::with_builtins()
            .parse(fixture["source"].as_str().unwrap(), &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = id(&initial.root);
        let sections = field(&initial, &initial.root, "sections");
        let section = field(&initial, &sections, "0");
        let items = field(&initial, &section, "items");
        let first = field(&initial, &items, "0");
        set_text(&mut initial, root, "title", "Initial");
        set_text(&mut initial, id(&section), "icon", "header");
        for (k, v) in [("icon", "item"), ("ticket", "T1"), ("assigned", "Ada")] {
            set_text(&mut initial, id(&first), k, v)
        }
        doc.apply(doc.revision(), initial).unwrap();
        let initial = doc.snapshot().unwrap();
        let graph = Rc::new(RefCell::new(initial));
        let mut reader = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut host = Callback {
            doc: &mut doc,
            graph: graph.clone(),
            mode,
            fired: false,
            plain: Plain::default(),
            nested: None,
            icons: 0,
            raw,
        };
        let done = job.run(&mut reader, &mut host);
        assert_eq!(host.fired, case["fired"].as_bool().unwrap());
        assert_json(&json!(host.plain.trace), &case["trace"]);
        assert_eq!(host.nested, case.get("nested").cloned());
        drop(host);
        let prior = doc.revision();
        let outcome = doc.finish_detached_render(done).unwrap();
        assert_eq!(outcome.revision, prior + 1);
        if case["first"]["ok"] == true {
            let result = drawing(&rendered(outcome.result.unwrap()));
            for k in ["svg", "width", "height", "links"] {
                assert_eq!(result[k], case["first"][k], "{mode}/{raw}/{responsive}/{k}");
            }
        } else {
            let PublicError::Pele(e) = outcome.result.err().unwrap() else {
                panic!("error kind")
            };
            assert_eq!(e.message, "Kanban callback sentinel");
        }
        let after = doc.snapshot().unwrap();
        assert_json(&state(&after), &case["after"]);
        assert_json(&output(&after, &sections), &case["originalSections"]);
        assert_json(&output(&after, &items), &case["originalItems"]);
        assert_json(&output(&after, &first), &case["originalItem"]);
        let current_sections = field(&after, &after.root, "sections");
        let current_section = field(&after, &current_sections, "0");
        let current_items = field(&after, &current_section, "items");
        let current_first = field(&after, &current_items, "0");
        assert_eq!(
            matches!((&sections, &current_sections), (Atom::Ref(a), Atom::Ref(b)) if a == b),
            case["sectionsCurrent"].as_bool().unwrap()
        );
        assert_eq!(
            matches!((&items, &current_items), (Atom::Ref(a), Atom::Ref(b)) if a == b),
            case["itemsCurrent"].as_bool().unwrap()
        );
        assert_eq!(
            matches!((&first, &current_first), (Atom::Ref(a), Atom::Ref(b)) if a == b),
            case["itemCurrent"].as_bool().unwrap()
        );
        *graph.borrow_mut() = after;
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut host = Plain::default();
        let done = job.run(&mut reader, &mut host);
        let result = doc.finish_detached_render(done).unwrap().result.unwrap();
        assert_eq!(drawing(&rendered(result)), case["recovery"]);
        assert_json(&json!(host.trace), &case["recoveryTrace"]);
        if !raw {
            let mut options = RenderOptions::default();
            options.render.max_width = responsive.then_some(300.);
            assert_eq!(
                drawing(
                    &doc.render(&options, &mut Plain::default())
                        .unwrap()
                        .rendered
                ),
                case["recovery"]
            );
        }
    }
}
