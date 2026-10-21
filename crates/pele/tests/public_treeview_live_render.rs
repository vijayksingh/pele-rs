//! Public TreeView normal/raw resource phases against the pinned live Pele oracle.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc};
const SOURCE: &str = "treeView-beta\ntitle Initial title\nfolder/ icon(folder)\n    Alpha.txt ## Alpha description\n    Beta.txt icon(file) ## Beta description\nTail.txt";
fn fixture() -> Json {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-treeview-live-render.json"
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
                message: JsString::from("TreeView live read requires string"),
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
        Atom::Undefined => json!({"undefined":true}),
        Atom::Null => Json::Null,
        Atom::Bool(b) => json!(b),
        Atom::Number(n) => json!({"number":format!("{:016x}",n.to_bits())}),
        Atom::String(id) => json!(g.strings[*id as usize].well_formed().unwrap()),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { length, .. } => Json::Array(
                (0..*length)
                    .map(|i| encoded(g, &field(g, a, &i.to_string())))
                    .collect(),
            ),
            Node::Map { entries, .. } => {
                json!({"map":entries.iter().map(|(k,v)|vec![encoded(g,k),encoded(g,v)]).collect::<Vec<_>>()})
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
fn fresh(g: &mut ModelGraph, name: &str) -> Atom {
    let name = text(g, name);
    let kind = text(g, "file");
    let children = array(g, vec![]);
    object(
        g,
        vec![
            ("id", Atom::Number(9.)),
            ("level", Atom::Number(4.)),
            ("name", name),
            ("nodeType", kind),
            ("children", children),
        ],
    )
}
fn nodes(g: &ModelGraph) -> (Atom, Atom, Atom, Atom) {
    let root = field(g, &g.root, "root");
    let list = field(g, &root, "children");
    let folder = field(g, &list, "0");
    let children = field(g, &folder, "children");
    let alpha = field(g, &children, "0");
    let beta = field(g, &children, "1");
    (root, folder, alpha, beta)
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
            "<path d=\"M0,0H2\"/>".into(),
        )))
    }
}
struct Callback<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    responsive: bool,
    fired: bool,
    plain: Plain,
    nested: Option<Json>,
}
impl Callback<'_> {
    fn mutate(&mut self) -> Result<(), PeleError> {
        self.fired = true;
        let mut g = self.doc.snapshot().unwrap();
        let wrapper = g.root.clone();
        let (root, folder, alpha, beta) = nodes(&g);
        match self.mode {
            "baseline" => {}
            "title" => set_text(&mut g, &wrapper, "title", "Changed title"),
            "current" => {
                set_text(&mut g, &alpha, "name", "Changed current");
                set_text(&mut g, &alpha, "nodeType", "directory");
                set_text(&mut g, &alpha, "cssClass", "highlight red");
            }
            "later" => {
                set_text(&mut g, &beta, "name", "Changed later");
                set_text(&mut g, &beta, "icon", "none");
            }
            "ids" => {
                let id = field(&g, &beta, "id");
                set_at(&mut g, &alpha, "id", id.clone());
                set_at(&mut g, &folder, "id", id);
            }
            "children-append" => {
                let list = field(&g, &folder, "children");
                let child = fresh(&mut g, "Appended");
                put_index(&mut g, &list, 2, child);
            }
            "children-replace" => {
                let child = fresh(&mut g, "Replacement");
                let list = array(&mut g, vec![child]);
                set_at(&mut g, &folder, "children", list);
            }
            "root-replace" => {
                let child = fresh(&mut g, "New root child");
                let list = array(&mut g, vec![child]);
                let Atom::Ref(id) = root else { panic!() };
                let node = g.nodes[id as usize].clone();
                let new = g.nodes.len() as u32;
                g.nodes.push(node);
                let new = Atom::Ref(new);
                set_at(&mut g, &new, "children", list);
                set_at(&mut g, &wrapper, "root", new);
            }
            "description" => {
                set_text(&mut g, &alpha, "description", "Changed description");
                set_text(&mut g, &beta, "description", "Later description");
            }
            "icon-current" => {
                set_text(&mut g, &folder, "name", "Icon current");
                set_text(&mut g, &folder, "nodeType", "file");
                set_text(&mut g, &folder, "cssClass", "highlight blue");
                set_text(&mut g, &folder, "description", "Icon description");
            }
            "icon-later" => {
                set_text(&mut g, &beta, "name", "Icon later");
                set_text(&mut g, &beta, "nodeType", "directory");
                set_text(&mut g, &beta, "description", "Icon later description");
            }
            "accessibility" => {
                set_text(&mut g, &wrapper, "accTitle", "Changed access");
                set_text(&mut g, &wrapper, "accDescr", "Changed description");
            }
            "throw-object" | "throw-undefined" | "icon-throw" => {
                set_text(&mut g, &wrapper, "title", "Persisted title");
                set_text(&mut g, &beta, "name", "Persisted later");
            }
            "nested-render" => set_text(&mut g, &wrapper, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            self.doc.apply(self.doc.revision(), g).unwrap();
            *self.graph.borrow_mut() = self.doc.snapshot().unwrap();
        }
        if self.mode.starts_with("throw-") || self.mode == "icon-throw" {
            return Err(PeleError::new(
                "TreeView callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested-render" {
            let job = self
                .doc
                .prepare_detached_render(self.doc.revision(), request(true, self.responsive))
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
            "children-append" | "children-replace" => "folder",
            "current" | "later" | "ids" | "root-replace" | "description" => "Alpha.txt",
            _ => "Initial title",
        };
        let hit = r.text == target;
        let result = self.plain.text_width(r)?;
        if !self.fired && !self.mode.starts_with("icon-") && hit {
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
fn treeview_live_phases_preserve_node_identity_children_icons_errors_and_recovery() {
    for case in fixture()["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{mode}/{raw}/{responsive}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let original_root = nodes(&graph.borrow()).0;
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
            responsive,
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
        assert_json(
            &encoded(&g, &g.root),
            &case["after"],
            &format!("{context}/state"),
        );
        assert_json(
            &encoded(&g, &original_root),
            &case["capturedRoot"],
            &format!("{context}/captured"),
        );
        let current = field(&g, &g.root, "root");
        assert_eq!(
            matches!((&current,&original_root),(Atom::Ref(a),Atom::Ref(b))if a==b),
            case["rootStillCurrent"].as_bool().unwrap(),
            "{context}"
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

// Native bounded allocation extension: repeated references cannot expand one
// admitted source string into unbounded retained row Strings. This tests native
// failure/partial-write recovery; it does not attribute this limit to Pele JS.
#[test]
fn treeview_repeated_node_text_budget_preserves_callback_writes_and_recovery() {
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
                let root = field(&g, &g.root, "root");
                let shared = fresh(&mut g, &"x".repeat(8192));
                let children = array(&mut g, vec![shared; 2049]);
                set_at(&mut g, &root, "children", children);
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
    let root = nodes(&initial).0;
    let original = field(&initial, &root, "children");
    let graph = Rc::new(RefCell::new(initial));
    let mut read = Read {
        graph: graph.clone(),
        reads: Rc::new(RefCell::new(vec![])),
    };
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
    let Atom::Ref(children) = field(&after, &root, "children") else {
        panic!("children")
    };
    assert!(matches!(
        after.nodes[children as usize],
        Node::Array { length: 2049, .. }
    ));
    set_at(&mut after, &root, "children", original);
    doc.apply(doc.revision(), after).unwrap();
    *graph.borrow_mut() = doc.snapshot().unwrap();
    let job = doc
        .prepare_detached_render(doc.revision(), request(false, false))
        .unwrap()
        .unwrap();
    let done = job.run(&mut read, &mut Plain::default());
    let result = drawing(&rendered(
        doc.finish_detached_render(done).unwrap().result.unwrap(),
    ));
    let fixture = fixture();
    let baseline = fixture["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["raw"] == false && r["responsive"] == false && r["mode"] == "baseline")
        .unwrap();
    assert_json(&result, &baseline["recovery"], "repaired rows");
}
