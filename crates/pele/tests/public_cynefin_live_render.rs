//! Public Cynefin normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE: &str = "cynefin-beta\ncomplex\n \"Alpha\"\n \"Beta\"\ncomplicated\n \"Gamma\"\nclear\n \"Delta\"\nchaotic\n \"Epsilon\"\nconfusion\n \"One\"\n \"Two\"\n \"Three\"\n \"Four\"\ncomplex --> clear : \"Move one\"\nclear --> chaotic : \"Move two\"";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-cynefin-live-render.json"
    ))
    .unwrap()
}
// JSON has one Number type; serde_json also remembers integer/f64 spelling.
// Compare the represented IEEE values, never round or stringify them. Numeric
// bit strings in the oracle remain exact strings (including NaN payloads).
fn assert_json(actual: &Json, expected: &Json, context: &str) {
    match (actual, expected) {
        (Json::Number(a), Json::Number(b)) => assert_eq!(
            a.as_f64().unwrap().to_bits(),
            b.as_f64().unwrap().to_bits(),
            "{context}: numeric bits differ"
        ),
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{context}: array length");
            for (index, (a, b)) in a.iter().zip(b).enumerate() {
                assert_json(a, b, &format!("{context}/{index}"));
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{context}: object fields"
            );
            for (key, a) in a {
                assert_json(a, &b[key], &format!("{context}/{key}"));
            }
        }
        _ => assert_eq!(actual, expected, "{context}"),
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
fn rendered(value: DetachedRenderValue) -> Rendered {
    match value {
        DetachedRenderValue::Normal(v) => v.rendered,
        DetachedRenderValue::Raw(v) => v,
    }
}
fn drawing(value: &Rendered) -> Json {
    json!({"svg":value.svg,"width":format!("{:016x}",value.width.to_bits()),"height":format!("{:016x}",value.height.to_bits())})
}
fn field(graph: &ModelGraph, receiver: &Atom, key: &str) -> Atom {
    if let Atom::Ref(id) = receiver {
        if key == "length" {
            if let Node::Array { length, .. } = &graph.nodes[*id as usize] {
                return Atom::Number(*length as f64);
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
    Pair(std::sync::Arc<[Value; 2]>),
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
            Value::Pair(values) => key
                .parse::<usize>()
                .ok()
                .and_then(|i| values.get(i))
                .cloned()
                .unwrap_or(Value::Atom(Atom::Undefined)),
            _ => Value::Atom(Atom::Undefined),
        })
    }
    fn index(&mut self, receiver: &Value, index: usize) -> Result<Value, PublicError> {
        self.get(receiver, &index.to_string())
    }
    fn has_index(&mut self, receiver: &Value, index: usize) -> Result<bool, PublicError> {
        let Value::Atom(Atom::Ref(id)) = receiver else {
            return Ok(false);
        };
        let g = self.graph.borrow();
        Ok(g.nodes[*id as usize]
            .properties()
            .iter()
            .any(|p| g.strings[p.key as usize].is(&index.to_string())))
    }
    fn snapshot_iterable(&mut self, value: &Value, max: usize) -> Result<Vec<Value>, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("Map")
        };
        let g = self.graph.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("Map")
        };
        assert!(entries.len() <= max);
        Ok(entries
            .iter()
            .map(|(k, v)| {
                Value::Pair(std::sync::Arc::new([
                    self.own(k.clone()),
                    self.own(v.clone()),
                ]))
            })
            .collect())
    }
    fn map_get_text(&mut self, value: &Value, key: &str) -> Result<Value, PublicError> {
        self.map_get(value, &Value::String(JsString::from(key)))
    }
    fn map_get(&mut self, value: &Value, key: &Value) -> Result<Value, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("Map")
        };
        let g = self.graph.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("Map")
        };
        Ok(entries
            .iter()
            .find_map(|(k, v)| match (k, key) {
                (Atom::String(i), Value::String(s)) if g.strings[*i as usize] == *s => {
                    Some(self.own(v.clone()))
                }
                _ => None,
            })
            .unwrap_or(Value::Atom(Atom::Undefined)))
    }
    fn array_join(&mut self, value: &Value, separator: &str) -> Result<JsString, PublicError> {
        let Value::Atom(Atom::Ref(id)) = value else {
            panic!("array")
        };
        let g = self.graph.borrow();
        let Node::Array { length, .. } = g.nodes[*id as usize] else {
            panic!("array")
        };
        let values = (0..length)
            .map(|i| match field(&g, &Atom::Ref(*id), &i.to_string()) {
                Atom::String(id) => g.strings[id as usize].well_formed().unwrap(),
                Atom::Undefined | Atom::Null => String::new(),
                _ => panic!("fixture join"),
            })
            .collect::<Vec<_>>();
        Ok(JsString::from(values.join(separator).as_str()))
    }
    fn object_identity(&mut self, value: &Value) -> Result<Option<u64>, PublicError> {
        Ok(if let Value::Atom(Atom::Ref(id)) = value {
            Some(*id as u64)
        } else {
            None
        })
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
        match value {
            Value::String(text) => Ok(text.clone()),
            Value::Atom(Atom::Number(n)) => Ok(JsString::from(n.to_string().as_str())),
            _ => panic!("fixture conversion"),
        }
    }
    fn require_string(&mut self, value: &Value, _: LiveStringUse) -> Result<JsString, PublicError> {
        self.string_units(value)?
            .ok_or_else(|| PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString::from("Cynefin live read requires string"),
            })
    }
}

fn text(g: &mut ModelGraph, s: &str) -> Atom {
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(s));
    Atom::String(id)
}
fn set_at(g: &mut ModelGraph, receiver: &Atom, key: &str, value: Atom) {
    let Atom::Ref(id) = receiver else {
        panic!("record")
    };
    if let Some(p) = g.nodes[*id as usize]
        .properties_mut()
        .iter_mut()
        .find(|p| g.strings[p.key as usize].is(key))
    {
        p.value = value;
        return;
    }
    let Atom::String(key) = text(g, key) else {
        unreachable!()
    };
    g.nodes[*id as usize].properties_mut().push(Property {
        key,
        value,
        flags: 7,
    });
}
fn set_text(g: &mut ModelGraph, receiver: &Atom, key: &str, value: &str) {
    let v = text(g, value);
    set_at(g, receiver, key, v)
}
fn object(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    let receiver = Atom::Ref(id);
    for (k, v) in fields {
        set_at(g, &receiver, k, v);
    }
    receiver
}
fn put_index(g: &mut ModelGraph, array: &Atom, index: usize, value: Atom) {
    set_at(g, array, &index.to_string(), value);
    let Atom::Ref(id) = array else { panic!() };
    if let Node::Array { length, .. } = &mut g.nodes[*id as usize] {
        *length = (*length).max(index as u32 + 1);
    }
}
fn encoded(g: &ModelGraph, a: &Atom) -> Json {
    match a {
        Atom::Undefined => Json::Null,
        Atom::Null => Json::Null,
        Atom::Bool(b) => json!(b),
        Atom::Number(n) => json!(n),
        Atom::String(id) => json!(g.strings[*id as usize].well_formed().unwrap()),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { length, .. } => Json::Array(
                (0..*length)
                    .map(|i| encoded(g, &field(g, a, &i.to_string())))
                    .collect(),
            ),
            Node::Map { entries, .. } => {
                json!(entries
                    .iter()
                    .map(|(k, v)| vec![encoded(g, k), encoded(g, v)])
                    .collect::<Vec<_>>())
            }
            Node::Set { values, .. } => {
                json!({"set": values.iter().map(|v|encoded(g,v)).collect::<Vec<_>>()})
            }
            node => Json::Object(
                node.properties()
                    .iter()
                    .map(|p| {
                        (
                            g.strings[p.key as usize].well_formed().unwrap(),
                            encoded(g, &p.value),
                        )
                    })
                    .collect(),
            ),
        },
        _ => panic!("fixture"),
    }
}

fn array(g: &mut ModelGraph, values: Vec<Atom>) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Array {
        length: 0,
        properties: vec![],
    });
    let a = Atom::Ref(id);
    for (i, v) in values.into_iter().enumerate() {
        put_index(g, &a, i, v);
    }
    a
}
fn domain(g: &ModelGraph, name: &str) -> Atom {
    let Atom::Ref(map) = field(g, &g.root, "domains") else {
        panic!()
    };
    let Node::Map { entries, .. } = &g.nodes[map as usize] else {
        panic!()
    };
    entries
        .iter()
        .find_map(|(key, value)| {
            matches!(key,Atom::String(id) if g.strings[*id as usize].is(name))
                .then_some(value.clone())
        })
        .unwrap_or(Atom::Undefined)
}
fn nodes(g: &ModelGraph) -> (Atom, Atom, Atom, Atom, Atom) {
    let domains = field(g, &g.root, "domains");
    let items = field(g, &domain(g, "complex"), "items");
    let first = field(g, &items, "0");
    let transitions = field(g, &g.root, "transitions");
    let transition = field(g, &transitions, "0");
    (domains, items, first, transitions, transition)
}
fn item(g: &mut ModelGraph, label: &str) -> Atom {
    let label = text(g, label);
    object(g, vec![("label", label)])
}
fn transition(g: &mut ModelGraph, from: &str, to: &str, label: &str) -> Atom {
    let from = text(g, from);
    let to = text(g, to);
    let label = text(g, label);
    object(g, vec![("from", from), ("to", to), ("label", label)])
}
fn state(g: &ModelGraph) -> Json {
    json!({"title":encoded(g,&field(g,&g.root,"title")),"accTitle":encoded(g,&field(g,&g.root,"accTitle")),"accDescr":encoded(g,&field(g,&g.root,"accDescr")),"domains":encoded(g,&field(g,&g.root,"domains")),"transitions":encoded(g,&field(g,&g.root,"transitions"))})
}
fn same(a: &Atom, b: &Atom) -> bool {
    matches!((a,b),(Atom::Ref(x),Atom::Ref(y))if x==y)
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
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.trace.push(json!(["icon", name]));
        Ok(Some(TrustedSvg::from_trusted_markup(
            "<path d=\"M0,0H2\"/>".into(),
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
    fn mutate(&mut self) -> Result<(), PeleError> {
        self.fired = true;
        let mut g = self.doc.snapshot().unwrap();
        let wrapper = g.root.clone();
        let (_, items, first, transitions, first_transition) = nodes(&g);
        match self.mode {
            "baseline" => {}
            "title-accessibility" => {
                set_text(&mut g, &wrapper, "title", "Changed title");
                set_text(&mut g, &wrapper, "accTitle", "Changed access");
                set_text(&mut g, &wrapper, "accDescr", "Changed description");
            }
            "domain-replace" => {
                let label = item(&mut g, "Replacement");
                let items = array(&mut g, vec![label]);
                let name = text(&mut g, "complex");
                let record = object(&mut g, vec![("name", name.clone()), ("items", items)]);
                let id = g.nodes.len() as u32;
                g.nodes.push(Node::Map {
                    entries: vec![(name, record)],
                    properties: vec![],
                });
                set_at(&mut g, &wrapper, "domains", Atom::Ref(id));
            }
            "next-domain" => {
                let list = field(&g, &domain(&g, "complicated"), "items");
                let item = field(&g, &list, "0");
                set_text(&mut g, &item, "label", "Changed Gamma");
            }
            "items-replace" => {
                let record = item(&mut g, "Replacement items");
                let list = array(&mut g, vec![record]);
                let domain = domain(&g, "complex");
                set_at(&mut g, &domain, "items", list);
            }
            "captured-label" => {
                set_text(&mut g, &first, "label", "After captured Alpha");
                let second = field(&g, &items, "1");
                set_text(&mut g, &second, "label", "Changed Beta");
            }
            "append" => {
                let record = item(&mut g, "Appended");
                put_index(&mut g, &items, 2, record);
            }
            "confusion-shrink" => {
                let list = field(&g, &domain(&g, "confusion"), "items");
                let Atom::Ref(id) = list else { panic!() };
                g.nodes[id as usize]
                    .properties_mut()
                    .retain(|p| !g.strings[p.key as usize].is("3"));
                if let Node::Array { length, .. } = &mut g.nodes[id as usize] {
                    *length = 3;
                }
            }
            "transitions-replace" => {
                let row = transition(&mut g, "complicated", "chaotic", "Replacement transition");
                let list = array(&mut g, vec![row]);
                set_at(&mut g, &wrapper, "transitions", list);
            }
            "transition-append" => {
                let row = transition(&mut g, "complex", "chaotic", "Appended transition");
                put_index(&mut g, &transitions, 2, row);
            }
            "transition-fields" => {
                set_text(&mut g, &first_transition, "from", "chaotic");
                set_text(&mut g, &first_transition, "to", "complicated");
                set_text(&mut g, &first_transition, "label", "Changed after capture");
            }
            "throw" => {
                set_text(&mut g, &wrapper, "title", "Before throw");
                set_text(&mut g, &first, "label", "Saved Alpha");
            }
            "nested" => set_text(&mut g, &wrapper, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            self.doc.apply(self.doc.revision(), g).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        if self.mode == "throw" {
            return Err(PeleError::new(
                "Cynefin callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested" {
            let job = self
                .doc
                .prepare_detached_render(self.doc.revision(), request(self.raw, false))
                .unwrap()
                .unwrap();
            let mut read = Read {
                graph: self.graph.clone(),
                reads: Rc::new(RefCell::new(vec![])),
            };
            let done = job.run(&mut read, &mut Plain::default());
            let outcome = self.doc.finish_detached_render(done).unwrap();
            self.nested = Some(drawing(&rendered(outcome.result.unwrap())));
            let mut g = self.doc.snapshot().unwrap();
            let wrapper = g.root.clone();
            set_text(&mut g, &wrapper, "title", "After nested");
            self.doc.apply(self.doc.revision(), g).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        Ok(())
    }
}
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let target = match self.mode {
            "transition-fields" => "Move one",
            "items-replace" | "captured-label" | "append" => "Alpha",
            "confusion-shrink" => "One",
            _ => "Complex",
        };
        let hit = r.text == target;
        let result = self.plain.text_width(r)?;
        if !self.fired && hit {
            self.mutate()?;
        }
        Ok(result)
    }
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        let result = self.plain.icon(name)?;
        if !self.fired && self.mode.starts_with("icon-") {
            self.mutate()?;
        }
        Ok(result)
    }
}
#[test]
fn cynefin_live_phases_preserve_domains_transitions_original_errors_and_recovery() {
    for case in fixture()["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{mode}/{raw}/{responsive}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = initial.root.clone();
        set_text(&mut initial, &root, "title", "Initial");
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let captured = nodes(&graph.borrow());
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut read = Read {
            graph: graph.clone(),
            reads: Rc::new(RefCell::new(vec![])),
        };
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
        assert_json(
            &json!(callback.plain.trace),
            &case["trace"],
            &format!("{context}/trace"),
        );
        assert_eq!(
            callback.fired,
            case["fired"].as_bool().unwrap(),
            "{context}/fired"
        );
        let nested = callback.nested.take();
        drop(callback);
        let outcome = doc.finish_detached_render(done).unwrap();
        if case["first"]["ok"] == true {
            let result = drawing(&rendered(outcome.result.unwrap()));
            for key in ["svg", "width", "height"] {
                assert_eq!(result[key], case["first"][key], "{context}/{key}");
            }
        } else {
            assert!(outcome.result.is_err(), "{context}");
        }
        let g = doc.snapshot().unwrap();
        assert_json(&state(&g), &case["after"], &format!("{context}/state"));
        let (domains, items, first, transitions, transition) = &captured;
        for (name, atom) in [
            ("originalItems", items),
            ("originalFirst", first),
            ("originalTransitions", transitions),
            ("originalTransition", transition),
        ] {
            assert_json(
                &encoded(&g, atom),
                &case[name],
                &format!("{context}/{name}"),
            );
        }
        assert_eq!(
            same(domains, &field(&g, &g.root, "domains")),
            case["domainsCurrent"].as_bool().unwrap(),
            "{context}/domains identity"
        );
        assert_eq!(
            same(items, &field(&g, &domain(&g, "complex"), "items")),
            case["itemsCurrent"].as_bool().unwrap(),
            "{context}/items identity"
        );
        assert_eq!(
            same(transitions, &field(&g, &g.root, "transitions")),
            case["transitionsCurrent"].as_bool().unwrap(),
            "{context}/transitions identity"
        );
        if let Some(n) = nested {
            assert_json(&n, &case["nested"], &format!("{context}/nested"));
        }
        let job = doc
            .prepare_detached_render(doc.revision(), request(raw, responsive))
            .unwrap()
            .unwrap();
        let mut plain = Plain::default();
        let done = job.run(&mut read, &mut plain);
        let result = rendered(doc.finish_detached_render(done).unwrap().result.unwrap());
        assert_json(
            &drawing(&result),
            &case["recovery"],
            &format!("{context}/recovery"),
        );
        assert_json(
            &json!(plain.trace),
            &case["recoveryTrace"],
            &format!("{context}/recoveryTrace"),
        );
    }
}

// Native resource extension: an admitted sparse Array length can grow inside a
// callback. The next live collection read must reject before materializing it,
// retain the accepted write, and permit ordinary public repair.
#[test]
fn cynefin_live_collection_budget_preserves_callback_write_and_recovery() {
    struct Grow<'a> {
        doc: &'a mut PublicDocument,
        graph: Rc<RefCell<ModelGraph>>,
        fired: bool,
    }
    impl RenderResources for Grow<'_> {
        fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
            if !self.fired {
                self.fired = true;
                let mut g = self.doc.snapshot()?;
                let items = field(&g, &domain(&g, "complex"), "items");
                let Atom::Ref(id) = items else { panic!() };
                if let Node::Array { length, .. } = &mut g.nodes[id as usize] {
                    *length = 1_000_001;
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
    let baseline = doc.snapshot().unwrap();
    let graph = Rc::new(RefCell::new(baseline.clone()));
    let mut read = Read {
        graph: graph.clone(),
        reads: Rc::new(RefCell::new(vec![])),
    };
    let job = doc
        .prepare_detached_render(doc.revision(), request(false, false))
        .unwrap()
        .unwrap();
    let mut plain = Plain::default();
    let done = job.run(&mut read, &mut plain);
    let expected = rendered(doc.finish_detached_render(done).unwrap().result.unwrap());
    let job = doc
        .prepare_detached_render(doc.revision(), request(false, false))
        .unwrap()
        .unwrap();
    let mut grow = Grow {
        doc: &mut doc,
        graph: graph.clone(),
        fired: false,
    };
    let done = job.run(&mut read, &mut grow);
    assert!(grow.fired);
    drop(grow);
    let outcome = doc.finish_detached_render(done).unwrap();
    assert!(matches!(
        outcome.result,
        Err(PublicError::Pele(PeleError {
            code: ErrorCode::Limit,
            ..
        }))
    ));
    let g = doc.snapshot().unwrap();
    let items = field(&g, &domain(&g, "complex"), "items");
    let Atom::Ref(id) = items else { panic!() };
    assert!(matches!(
        g.nodes[id as usize],
        Node::Array {
            length: 1_000_001,
            ..
        }
    ));
    doc.apply(doc.revision(), baseline).unwrap();
    *graph.borrow_mut() = doc.snapshot().unwrap();
    let job = doc
        .prepare_detached_render(doc.revision(), request(false, false))
        .unwrap()
        .unwrap();
    let done = job.run(&mut read, &mut Plain::default());
    let actual = rendered(doc.finish_detached_render(done).unwrap().result.unwrap());
    assert_eq!(actual.svg, expected.svg);
    assert_eq!(actual.width.to_bits(), expected.width.to_bits());
    assert_eq!(actual.height.to_bits(), expected.height.to_bits());
}
