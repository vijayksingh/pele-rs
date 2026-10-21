//! Public Railroad normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE: &str = "railroad-beta\nentry = sequence(terminal(\"Alpha\"), nonterminal(\"Other\"), zeroOrMore(terminal(\"Beta\")));\nOther = choice(terminal(\"x\"), special(\"y\"));";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-railroad-live-render.json"
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
        max_width: responsive.then_some(180.),
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
fn nodes(g: &ModelGraph) -> (Atom, Atom, Atom, Atom, Atom, Atom) {
    let rules = field(g, &g.root, "rules");
    let rule = field(g, &rules, "0");
    let definition = field(g, &rule, "definition");
    let children = field(g, &definition, "elements");
    let first = field(g, &children, "0");
    let repetition = field(g, &children, "2");
    (rules, rule, definition, children, first, repetition)
}
fn terminal(g: &mut ModelGraph, value: &str) -> Atom {
    let kind = text(g, "terminal");
    let value = text(g, value);
    object(g, vec![("type", kind), ("value", value)])
}
fn rule(g: &mut ModelGraph, name: &str, value: &str) -> Atom {
    let name = text(g, name);
    let definition = terminal(g, value);
    object(g, vec![("name", name), ("definition", definition)])
}
fn state(g: &ModelGraph) -> Json {
    json!({"title":encoded(g,&field(g,&g.root,"title")),"accTitle":encoded(g,&field(g,&g.root,"accTitle")),"accDescr":encoded(g,&field(g,&g.root,"accDescr")),"rules":encoded(g,&field(g,&g.root,"rules"))})
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
impl RenderResources for Callback<'_> {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        let target = match self.mode {
            "repetition" => "Beta",
            "rules-append" => "entry",
            "title-accessibility" | "nested" | "baseline" => "Initial",
            _ => "Alpha",
        };
        let trigger = r.text == target;
        let width = self.plain.text_width(r)?;
        if !self.fired && trigger {
            self.fired = true;
            let mut g = self.doc.snapshot().unwrap();
            let root = g.root.clone();
            let (rules, rule, definition, children, first, repetition) = nodes(&g);
            match self.mode {
                "title-accessibility" => {
                    set_text(&mut g, &root, "title", "Changed title");
                    set_text(&mut g, &root, "accTitle", "Changed access");
                    set_text(&mut g, &root, "accDescr", "Changed description");
                }
                "rules-replace" => {
                    let v = rule_value(&mut g, "Replacement", "new");
                    let v = array(&mut g, vec![v]);
                    set_at(&mut g, &root, "rules", v);
                }
                "rule-name" => set_text(&mut g, &rule, "name", "Changed entry"),
                "definition-replace" => {
                    let v = terminal(&mut g, "Replacement definition");
                    set_at(&mut g, &rule, "definition", v);
                }
                "element-text" => {
                    set_text(&mut g, &first, "value", "After captured Alpha");
                    let second = field(&g, &children, "1");
                    set_text(&mut g, &second, "name", "Changed Other");
                }
                "children-replace" => {
                    let v = terminal(&mut g, "Replacement child");
                    let v = array(&mut g, vec![v]);
                    set_at(&mut g, &definition, "elements", v);
                }
                "children-append" => {
                    let v = terminal(&mut g, "Appended child");
                    put_index(&mut g, &children, 3, v);
                }
                "type-change" => {
                    set_text(&mut g, &definition, "type", "choice");
                    set_at(&mut g, &definition, "alternatives", children);
                }
                "repetition" => {
                    set_at(&mut g, &repetition, "min", Atom::Number(2.));
                    set_at(&mut g, &repetition, "max", Atom::Number(4.));
                }
                "rules-append" => {
                    let v = rule_value(&mut g, "Appended", "new");
                    put_index(&mut g, &rules, 2, v);
                }
                "shared-identity" => set_text(&mut g, &first, "value", "Second occurrence"),
                "throw" => {
                    set_text(&mut g, &root, "title", "Before throw");
                    set_text(&mut g, &first, "value", "Saved Alpha");
                }
                "nested" => set_text(&mut g, &root, "title", "Nested title"),
                "baseline" => {}
                _ => panic!("mode"),
            }
            if self.mode != "baseline" {
                self.doc.apply(self.doc.revision(), g).unwrap();
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
            if self.mode == "throw" {
                return Err(PeleError::new(
                    "Railroad callback sentinel",
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
                let output = self
                    .doc
                    .finish_detached_render(done)
                    .unwrap()
                    .result
                    .unwrap();
                self.nested = Some(drawing(&rendered(output)));
                let mut g = self.doc.snapshot().unwrap();
                let root = g.root.clone();
                set_text(&mut g, &root, "title", "After nested");
                self.doc.apply(self.doc.revision(), g).unwrap();
                *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
            }
        }
        Ok(width)
    }
}
fn rule_value(g: &mut ModelGraph, name: &str, value: &str) -> Atom {
    rule(g, name, value)
}
#[test]
fn railroad_live_phases_preserve_retained_grammar_aliases_folding_errors_and_recovery() {
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
        if mode == "shared-identity" {
            let (_, _, _, children, first, _) = nodes(&initial);
            put_index(&mut initial, &children, 1, first);
        }
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
        let result = doc.finish_detached_render(done).unwrap().result;
        if case["first"]["ok"] == true {
            let result = drawing(&rendered(result.unwrap()));
            for key in ["svg", "width", "height"] {
                assert_eq!(result[key], case["first"][key], "{context}/{key}");
            }
        } else {
            assert!(
                matches!(result,Err(PublicError::Pele(ref e))if e.message=="Railroad callback sentinel"),
                "{context}/original error"
            );
        }
        let g = doc.snapshot().unwrap();
        assert_json(&state(&g), &case["after"], &format!("{context}/state"));
        let (rules, rule, definition, children, first, _) = &captured;
        for (name, atom) in [
            ("originalRules", rules),
            ("originalRule", rule),
            ("originalDefinition", definition),
            ("originalChildren", children),
            ("originalFirst", first),
        ] {
            assert_json(
                &encoded(&g, atom),
                &case[name],
                &format!("{context}/{name}"),
            );
        }
        let current_rules = field(&g, &g.root, "rules");
        let current_rule = field(&g, &current_rules, "0");
        let current_definition = field(&g, &current_rule, "definition");
        let current_children = field(&g, &current_definition, "elements");
        for (name, a, b) in [
            ("rulesCurrent", rules, &current_rules),
            ("ruleCurrent", rule, &current_rule),
            ("definitionCurrent", definition, &current_definition),
            ("childrenCurrent", children, &current_children),
        ] {
            assert_eq!(
                same(a, b),
                case[name].as_bool().unwrap(),
                "{context}/{name}"
            );
        }
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

#[test]
fn railroad_live_collection_budget_preserves_callback_write_and_recovery() {
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
                let items = field(&g, &g.root, "rules");
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
    let items = field(&g, &g.root, "rules");
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
