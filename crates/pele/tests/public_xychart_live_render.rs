//! Public XYChart callbacks compare actual pinned Pele phase results and recovery.
use pele::compat::{public::*, *};
use serde_json::{json, Value as Json};
use std::{cell::RefCell, rc::Rc, sync::Arc};
const SOURCE:&str="xychart\ntitle \"Initial title\"\nx-axis Month [Jan, Feb, Mar]\ny-axis Revenue -10 --> 20\nline Forecast [5 \"PointA\", 10 \"PointB\", 15 \"PointC\"]\nbar Sales [-5, 10, 15]";
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
            panic!("Map")
        };
        let g = self.0.borrow();
        let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
            panic!("Map")
        };
        Ok(entries
            .iter()
            .find_map(|(k, v)| match k {
                Atom::String(i) if g.strings[*i as usize].is(key) => Some(self.own(v.clone())),
                _ => None,
            })
            .unwrap_or(Val::Atom(Atom::Undefined)))
    }
    fn array_join(&mut self, v: &Val, separator: &str) -> Result<JsString, PublicError> {
        let Val::Atom(Atom::Ref(id)) = v else {
            panic!("Array")
        };
        let g = self.0.borrow();
        let Node::Array { length, .. } = &g.nodes[*id as usize] else {
            panic!("Array")
        };
        let values = (0..*length)
            .map(|i| match field(&g, &Atom::Ref(*id), &i.to_string()) {
                Atom::String(i) => g.strings[i as usize].well_formed().unwrap(),
                Atom::Undefined | Atom::Null => String::new(),
                _ => panic!("fixture string"),
            })
            .collect::<Vec<_>>();
        Ok(JsString::from(values.join(separator).as_str()))
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
            PeleError::new("XYChart label requires string", ErrorCode::Semantic).into()
        })
    }
}
fn config() -> Object {
    serde_json::from_value(json!({"xyChart":{"showDataLabel":true}})).unwrap()
}
fn request(raw: bool, responsive: bool) -> DetachedRenderRequest {
    let settings = RenderSettings {
        max_width: responsive.then_some(320.),
        ..Default::default()
    };
    if raw {
        DetachedRenderRequest::Raw {
            config: config(),
            settings,
        }
    } else {
        DetachedRenderRequest::Normal {
            options: RenderOptions {
                parse: ParseOptions {
                    preparation: PrepareOptions {
                        config: config(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                render: settings,
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
#[derive(Clone, Copy)]
struct Handles {
    x: u32,
    categories: u32,
    y: u32,
    plots: u32,
    line: u32,
    bar: u32,
    data: u32,
    labels: u32,
}
fn child(g: &ModelGraph, a: u32, k: &str) -> u32 {
    id(field(g, &Atom::Ref(a), k))
}
fn handles(g: &ModelGraph) -> Handles {
    let root = id(g.root.clone());
    let x = child(g, root, "xAxis");
    let categories = child(g, x, "categories");
    let y = child(g, root, "yAxis");
    let plots = child(g, root, "plots");
    let line = child(g, plots, "0");
    let bar = child(g, plots, "1");
    let data = child(g, line, "data");
    let labels = child(g, line, "pointLabels");
    Handles {
        x,
        categories,
        y,
        plots,
        line,
        bar,
        data,
        labels,
    }
}
fn object(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> u32 {
    let i = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    for (k, v) in fields {
        set(g, i, k, v);
    }
    i
}
fn new_plot(g: &mut ModelGraph, title: &str, values: &[f64], labels: &[&str]) -> u32 {
    let mut points = vec![];
    for (i, v) in values.iter().enumerate() {
        let key = text(g, &((b'a' + i as u8) as char).to_string());
        let a = array(g, vec![key, Atom::Number(*v)]);
        points.push(Atom::Ref(a));
    }
    let data = array(g, points);
    let labels = labels.iter().map(|s| text(g, s)).collect();
    let labels = array(g, labels);
    let title = text(g, title);
    let kind = text(g, "line");
    object(
        g,
        vec![
            ("type", kind),
            ("title", title),
            ("data", Atom::Ref(data)),
            ("pointLabels", Atom::Ref(labels)),
        ],
    )
}
fn put_index(g: &mut ModelGraph, a: u32, index: u32, v: Atom) {
    set(g, a, &index.to_string(), v);
    let Node::Array { length, .. } = &mut g.nodes[a as usize] else {
        panic!()
    };
    *length = (*length).max(index + 1);
}
fn encoded(g: &ModelGraph, a: &Atom) -> Json {
    match a {
        Atom::Undefined | Atom::Null => Json::Null,
        Atom::Bool(v) => json!(v),
        Atom::Number(v) => json!(v),
        Atom::String(i) => json!(g.strings[*i as usize].well_formed().unwrap()),
        Atom::Ref(i) => match &g.nodes[*i as usize] {
            Node::Array { length, .. } => json!((0..*length)
                .map(|n| encoded(g, &field(g, a, &n.to_string())))
                .collect::<Vec<_>>()),
            Node::Map { entries, .. } => json!(entries
                .iter()
                .map(|(k, v)| vec![encoded(g, k), encoded(g, v)])
                .collect::<Vec<_>>()),
            n => Json::Object(
                n.properties()
                    .iter()
                    .filter(|p| !matches!(p.value, Atom::Undefined))
                    .map(|p| {
                        (
                            g.strings[p.key as usize].well_formed().unwrap(),
                            encoded(g, &p.value),
                        )
                    })
                    .collect(),
            ),
        },
        _ => panic!("fixture value"),
    }
}
fn state(g: &ModelGraph) -> Json {
    let mut out = serde_json::Map::new();
    for k in [
        "title",
        "accTitle",
        "accDescr",
        "orientation",
        "xAxis",
        "yAxis",
        "hasSetYAxis",
        "plots",
    ] {
        out.insert(k.into(), encoded(g, &field(g, &g.root, k)));
    }
    Json::Object(out)
}
fn same(a: &Atom, b: u32) -> bool {
    matches!(a,Atom::Ref(i) if *i==b)
}
fn includes(g: &ModelGraph, array: u32, object: u32) -> bool {
    let Node::Array { length, .. } = g.nodes[array as usize] else {
        panic!()
    };
    (0..length).any(|i| same(&field(g, &Atom::Ref(array), &i.to_string()), object))
}
fn original(g: &ModelGraph, h: Handles) -> Json {
    json!({"x":encoded(g,&Atom::Ref(h.x)),"categories":encoded(g,&Atom::Ref(h.categories)),"y":encoded(g,&Atom::Ref(h.y)),"plots":encoded(g,&Atom::Ref(h.plots)),"line":encoded(g,&Atom::Ref(h.line)),"bar":encoded(g,&Atom::Ref(h.bar)),"data":encoded(g,&Atom::Ref(h.data)),"labels":encoded(g,&Atom::Ref(h.labels))})
}
fn identity(g: &ModelGraph, h: Handles) -> Json {
    let root = id(g.root.clone());
    let x = child(g, root, "xAxis");
    let plots = child(g, root, "plots");
    json!({"x":x==h.x,"categories":same(&field(g,&Atom::Ref(x),"categories"),h.categories),"y":child(g,root,"yAxis")==h.y,"plots":plots==h.plots,"line":includes(g,plots,h.line),"bar":includes(g,plots,h.bar),"data":child(g,h.line,"data")==h.data,"labels":child(g,h.line,"pointLabels")==h.labels})
}
fn assert_json(a: &Json, b: &Json, context: &str) {
    match (a, b) {
        (Json::Number(a), Json::Number(b)) => assert_eq!(
            a.as_f64().unwrap().to_bits(),
            b.as_f64().unwrap().to_bits(),
            "{context}"
        ),
        (Json::Array(a), Json::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{context}");
            for (a, b) in a.iter().zip(b) {
                assert_json(a, b, context)
            }
        }
        (Json::Object(a), Json::Object(b)) => {
            assert_eq!(
                a.keys().collect::<Vec<_>>(),
                b.keys().collect::<Vec<_>>(),
                "{context}"
            );
            for (k, a) in a {
                assert_json(a, &b[k], context)
            }
        }
        _ => assert_eq!(a, b, "{context}"),
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
}
struct Callback<'a> {
    doc: &'a mut PublicDocument,
    graph: Rc<RefCell<ModelGraph>>,
    mode: &'a str,
    raw: bool,
    fired: bool,
    hits: usize,
    plain: Plain,
    nested: Option<Json>,
}
impl Callback<'_> {
    fn trigger(&mut self, label: &str) -> Result<(), PeleError> {
        let target = match self.mode {
            "early-title" => "0",
            "title-accessibility" => "Initial title",
            "category-current" | "categories-append" => "Jan",
            "point-labels" | "data-value" | "data-replace" | "late-point" => "PointA",
            _ => "Forecast",
        };
        if self.fired || label != target {
            return Ok(());
        }
        self.hits += 1;
        if self.mode == "late-point" && self.hits != 2 {
            return Ok(());
        }
        self.fired = true;
        let mut g = self.doc.snapshot()?;
        let h = handles(&g);
        let root = id(g.root.clone());
        match self.mode {
            "baseline" => {}
            "early-title" => set_text(&mut g, root, "title", "Early title"),
            "legend-current" => {
                set_text(&mut g, h.line, "title", "Changed current");
                set_text(&mut g, h.line, "type", "bar");
            }
            "legend-next" => set_text(&mut g, h.bar, "title", "Changed later"),
            "plots-replace" => {
                let p = new_plot(&mut g, "Replacement", &[3., 7.], &["R1", "R2"]);
                let a = array(&mut g, vec![Atom::Ref(p)]);
                set(&mut g, root, "plots", Atom::Ref(a));
            }
            "plots-append" => {
                let p = new_plot(&mut g, "Appended", &[2., 4.], &["A1", "A2"]);
                put_index(&mut g, h.plots, 2, Atom::Ref(p));
            }
            "axis-replace" => {
                let kind = text(&mut g, "linear");
                let title = text(&mut g, "Replacement axis");
                let x = object(
                    &mut g,
                    vec![
                        ("type", kind.clone()),
                        ("title", title),
                        ("min", Atom::Number(0.)),
                        ("max", Atom::Number(2.)),
                    ],
                );
                let title = text(&mut g, "Replacement value");
                let y = object(
                    &mut g,
                    vec![
                        ("type", kind),
                        ("title", title),
                        ("min", Atom::Number(0.)),
                        ("max", Atom::Number(30.)),
                    ],
                );
                set(&mut g, root, "xAxis", Atom::Ref(x));
                set(&mut g, root, "yAxis", Atom::Ref(y));
            }
            "category-current" => {
                set_text(&mut g, h.categories, "0", "Changed current");
                set_text(&mut g, h.categories, "1", "Changed later");
            }
            "categories-append" => {
                let v = text(&mut g, "Apr");
                put_index(&mut g, h.categories, 3, v);
            }
            "bounds" => {
                set(&mut g, h.y, "min", Atom::Number(-20.));
                set(&mut g, h.y, "max", Atom::Number(40.));
                set(&mut g, root, "hasSetYAxis", Atom::Bool(true));
            }
            "point-labels" => {
                set_text(&mut g, h.labels, "0", "Changed point");
                set_text(&mut g, h.labels, "1", "Changed later point");
            }
            "data-value" => {
                let point = child(&g, h.data, "1");
                set(&mut g, point, "1", Atom::Number(18.));
                let a = child(&g, h.bar, "data");
                let point = child(&g, a, "0");
                set(&mut g, point, "1", Atom::Number(-8.));
            }
            "data-replace" => {
                let p = new_plot(&mut g, "", &[1., 3., 7.], &[]);
                let d = child(&g, p, "data");
                set(&mut g, h.line, "data", Atom::Ref(d));
            }
            "late-point" => {
                let point = child(&g, h.data, "1");
                set(&mut g, point, "1", Atom::Number(19.));
                set_text(&mut g, h.labels, "1", "Paint changed");
            }
            "title-accessibility" => {
                set_text(&mut g, root, "title", "Changed title");
                set_text(&mut g, root, "accTitle", "Changed access");
                set_text(&mut g, root, "accDescr", "Changed description");
            }
            "throw-object" | "throw-undefined" => {
                set_text(&mut g, root, "title", "Before failure");
                let point = child(&g, h.data, "1");
                set(&mut g, point, "1", Atom::Number(17.));
            }
            "nested" => set_text(&mut g, root, "title", "Nested title"),
            _ => panic!("mode"),
        }
        if self.mode != "baseline" {
            self.doc.apply(self.doc.revision(), g)?;
            *self.graph.borrow_mut() = self.doc.snapshot()?;
        }
        if self.mode.starts_with("throw-") {
            return Err(PeleError::new(
                "XYChart callback sentinel",
                ErrorCode::Semantic,
            ));
        }
        if self.mode == "nested" {
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
        let label = r.text.to_owned();
        let width = self.plain.text_width(r)?;
        self.trigger(&label)?;
        Ok(width)
    }
}
#[test]
fn xychart_live_callbacks_preserve_captured_axes_series_labels_and_recovery() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-xychart-live-render.json"
    ))
    .unwrap();
    assert_eq!(fixture["rows"].as_array().unwrap().len(), 72);
    for case in fixture["rows"].as_array().unwrap() {
        let raw = case["raw"].as_bool().unwrap();
        let responsive = case["responsive"].as_bool().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let context = format!("{raw}/{responsive}/{mode}");
        let mut doc = PublicEngine::with_builtins()
            .parse(SOURCE, &Default::default())
            .unwrap();
        let mut initial = doc.snapshot().unwrap();
        let root = id(initial.root.clone());
        if responsive {
            set_text(&mut initial, root, "orientation", "horizontal");
        }
        doc.apply(doc.revision(), initial).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let old = handles(&graph.borrow());
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
            hits: 0,
            plain: Plain::default(),
            nested: None,
        };
        let done = job.run(&mut read, &mut callback);
        assert!(callback.fired, "{context}");
        assert_json(&json!(callback.plain.trace), &case["trace"], &context);
        assert_eq!(callback.nested, case.get("nested").cloned(), "{context}");
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
            assert_eq!(e.message, "XYChart callback sentinel");
        }
        assert!(!doc.has_active_detached_render());
        let now = doc.snapshot().unwrap();
        assert_json(&state(&now), &case["after"], &context);
        assert_json(&original(&now, old), &case["original"], &context);
        assert_json(&identity(&now, old), &case["identity"], &context);
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
        assert_json(&json!(plain.trace), &case["recoveryTrace"], &context);
        if !raw {
            let normal = doc
                .render(
                    &RenderOptions {
                        parse: ParseOptions {
                            preparation: PrepareOptions {
                                config: config(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        render: RenderSettings {
                            max_width: responsive.then_some(320.),
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

#[test]
fn nan_category_stride_stops_at_unordered_comparison_and_preserves_public_output_limits() {
    let fixture: Json = serde_json::from_str(include_str!(
        "../../../fixtures/public-xychart-nan-category.json"
    ))
    .unwrap();
    assert_eq!(
        fixture["oracleSHA256"],
        "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d"
    );
    assert_eq!(fixture["source"], SOURCE);
    let engine = PublicEngine::with_builtins();
    for row in fixture["rows"].as_array().unwrap() {
        let raw = row["raw"].as_bool().unwrap();
        let mut doc = engine.parse(SOURCE, &ParseOptions::default()).unwrap();
        let graph = Rc::new(RefCell::new(doc.snapshot().unwrap()));
        let mut read = Read(graph);
        let mut req = request(raw, false);
        match &mut req {
            DetachedRenderRequest::Normal { options, .. } => {
                options.render.padding = Some(f64::NAN)
            }
            DetachedRenderRequest::Raw { settings, .. } => settings.padding = Some(f64::NAN),
        }
        let mut host = Plain::default();
        let job = doc
            .prepare_detached_render(doc.revision(), req)
            .unwrap()
            .unwrap();
        let done = job.run(&mut read, &mut host);
        let actual = drawing(&rendered(
            doc.finish_detached_render(done).unwrap().result.unwrap(),
        ));
        for key in ["svg", "width", "height"] {
            assert_eq!(actual[key], row[key], "raw={raw}/{key}");
        }
        assert_eq!(
            json!(host.trace),
            row["trace"],
            "raw={raw}/callback schedule"
        );
        assert!(!doc.has_active_detached_render());
        if !raw {
            let mut options = RenderOptions::default();
            options.parse.preparation.config = config();
            options.render.padding = Some(f64::NAN);
            let mut host = Plain::default();
            let typed = doc.render(&options, &mut host).unwrap();
            assert_eq!(drawing(&typed.rendered), actual);
            assert_eq!(json!(host.trace), row["trace"]);
            let req = DetachedRenderRequest::Normal {
                options: options.clone(),
                limit: Some(OutputLimit::Number(1.)),
                enforce_native_bytes: true,
            };
            let job = doc
                .prepare_detached_render(doc.revision(), req)
                .unwrap()
                .unwrap();
            let done = job.run(&mut read, &mut Plain::default());
            let error = doc
                .finish_detached_render(done)
                .unwrap()
                .result
                .err()
                .expect("output budget still enforced");
            let PublicError::Pele(error) = error else {
                panic!("limit error kind")
            };
            assert_eq!(error.code, ErrorCode::Limit);
            assert!(!doc.has_active_detached_render());
            let mut host = Plain::default();
            assert_eq!(
                drawing(&doc.render(&options, &mut host).unwrap().rendered),
                actual
            );
            assert_eq!(json!(host.trace), row["trace"]);
        }
    }
}
