//! Pinned ClassDb defaults, retained helper aliases and raw rendering.
//! All native calls use captured capabilities; parsing never initializes a model.
use pele::compat::{public::*, *};
use serde_json::{json, Value};
use std::collections::HashMap;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-class-constructor.json"
    ))
    .unwrap()
}
fn graph(g: &ModelGraph, result: &Atom) -> Value {
    fn atom(
        g: &ModelGraph,
        a: &Atom,
        ids: &mut HashMap<u32, usize>,
        nodes: &mut Vec<Value>,
    ) -> Value {
        match a {
            Atom::Undefined => json!(["undefined"]),
            Atom::Null => json!(["null"]),
            Atom::Bool(v) => json!(["bool", v]),
            Atom::Number(v) => json!(["number", format!("{:016x}", v.to_bits())]),
            Atom::String(id) => json!(["string", g.strings[*id as usize].well_formed().unwrap()]),
            Atom::Ref(id) => {
                if let Some(i) = ids.get(id) {
                    return json!(["ref", i]);
                }
                let i = nodes.len();
                ids.insert(*id, i);
                nodes.push(Value::Null);
                let n = &g.nodes[*id as usize];
                let kind = match n {
                    Node::Array { .. } => "array",
                    Node::Map { .. } => "map",
                    Node::Set { .. } => "set",
                    Node::Instance { .. } => "instance",
                    Node::Object { .. } => "object",
                    _ => panic!("fixture node kind"),
                };
                let mut out = json!({"kind":kind,"properties":[]});
                if let Node::Map { entries, .. } = n {
                    out["entries"] = entries
                        .iter()
                        .map(|(k, v)| json!([atom(g, k, ids, nodes), atom(g, v, ids, nodes)]))
                        .collect();
                }
                if let Node::Set { values, .. } = n {
                    out["values"] = values.iter().map(|v| atom(g, v, ids, nodes)).collect();
                }
                let mut properties: Vec<Value> = n
                    .properties()
                    .iter()
                    .map(|p| {
                        json!([
                            g.strings[p.key as usize].well_formed().unwrap(),
                            atom(g, &p.value, ids, nodes),
                            p.flags
                        ])
                    })
                    .collect();
                if let Node::Array { length, .. } = n {
                    if !n
                        .properties()
                        .iter()
                        .any(|p| g.strings[p.key as usize].is("length"))
                    {
                        properties.push(json!([
                            "length",
                            ["number", format!("{:016x}", (*length as f64).to_bits())],
                            2
                        ]));
                    }
                }
                out["properties"] = properties.into();
                nodes[i] = out;
                json!(["ref", i])
            }
            _ => panic!("fixture atom kind"),
        }
    }
    let mut ids = HashMap::new();
    let mut nodes = Vec::new();
    let root = atom(g, &g.root, &mut ids, &mut nodes);
    let result = atom(g, result, &mut ids, &mut nodes);
    json!({"root":root,"nodes":nodes,"result":result})
}
#[derive(Default)]
struct Width {
    trace: Vec<Value>,
}
impl RenderResources for Width {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.trace
            .push(json!([r.text, format!("{:016x}", r.size.to_bits())]));
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn raw(doc: &mut PublicDocument) -> Value {
    let mut host = Width::default();
    let settings = RenderSettings {
        now: Some(1704067200000.0),
        ..Default::default()
    };
    let outcome = doc
        .render_raw_operation(doc.revision(), &Object::default(), &settings, &mut host)
        .unwrap();
    match outcome.result {
        Ok(r) => {
            json!({"ok":true,"svg":r.svg,"width":format!("{:016x}",r.width.to_bits()),"height":format!("{:016x}",r.height.to_bits()),"trace":host.trace})
        }
        Err(PublicError::Pele(e)) => json!({"ok":false,"error":e,"trace":host.trace}),
        Err(e) => panic!("unexpected raw error {e:?}"),
    }
}
fn string(g: &mut ModelGraph, s: &str) -> u32 {
    if let Some(i) = g.strings.iter().position(|v| v.is(s)) {
        return i as u32;
    }
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(s));
    id
}
// Ordered fixture argument objects become ordinary public graph values. Calling
// apply first is the same supported public ingress used by host helper adapters.
fn argument(g: &mut ModelGraph, value: &Value) -> Atom {
    match value {
        Value::Null => Atom::Null,
        Value::Bool(v) => Atom::Bool(*v),
        Value::Number(v) => Atom::Number(v.as_f64().unwrap()),
        Value::String(v) => Atom::String(string(g, v)),
        Value::Object(v) if v.contains_key("array") => {
            let array = v["array"].as_array().unwrap();
            let properties = array
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let key = string(g, &i.to_string());
                    let value = argument(g, v);
                    Property {
                        key,
                        value,
                        flags: 7,
                    }
                })
                .collect();
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Array {
                length: array.len() as u32,
                properties,
            });
            Atom::Ref(id)
        }
        Value::Object(v) if v.contains_key("object") => {
            let properties = v["object"]
                .as_array()
                .unwrap()
                .iter()
                .map(|pair| {
                    let key = string(g, pair[0].as_str().unwrap());
                    let value = argument(g, &pair[1]);
                    Property {
                        key,
                        value,
                        flags: 7,
                    }
                })
                .collect();
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Object { properties });
            Atom::Ref(id)
        }
        _ => panic!("invalid argument fixture"),
    }
}
fn reference(a: &Atom) -> u32 {
    let Atom::Ref(id) = a else {
        panic!("nonvacuous alias must be object")
    };
    *id
}
fn selected(g: &ModelGraph, result: &Atom, action: &Value) -> Atom {
    let path = action
        .get("rememberPath")
        .or_else(|| action.get("samePath"));
    let Some(path) = path else {
        return result.clone();
    };
    let mut a = g.root.clone();
    for key in path.as_array().unwrap() {
        a = g.nodes[reference(&a) as usize]
            .properties()
            .iter()
            .find(|p| g.strings[p.key as usize].is(key.as_str().unwrap()))
            .unwrap()
            .value
            .clone();
    }
    a
}

#[test]
fn class_constructor_defaults_helpers_and_raw_render_match_pele() {
    let expected = fixture();
    let case = &expected["cases"][0];
    assert_eq!(case["family"], "class");
    let engine = PublicEngine::with_builtins();
    assert!(engine
        .capture_constructor(&DiagramType::Class.into(), PrototypeId(u32::MAX))
        .unwrap()
        .is_none());
    let factory = engine
        .capture_constructor(&DiagramType::Class.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    drop(engine);
    assert!(factory.ignores_arguments());
    let mut doc = factory.construct().unwrap();
    let sibling = factory.construct().unwrap();
    let mut drawing_probe = factory.construct().unwrap();
    drop(factory);
    assert!(doc.is_constructor_created());
    assert_eq!(doc.revision(), 0);
    assert_eq!(
        graph(&doc.snapshot().unwrap(), &Atom::Undefined),
        case["initial"]["graph"]
    );
    assert_eq!(raw(&mut drawing_probe), case["initial"]["render"]);
    let mut identities = HashMap::new();
    for action in case["actions"].as_array().unwrap() {
        let name = action["name"].as_str().unwrap();
        let mut input = doc.snapshot().unwrap();
        let arguments = action["args"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| argument(&mut input, v))
            .collect();
        doc.apply(doc.revision(), input).unwrap();
        let method = doc
            .model_schema()
            .methods
            .iter()
            .find(|m| m.name == name)
            .unwrap()
            .id;
        let receiver = doc.snapshot().unwrap().root;
        let out = doc
            .invoke(
                doc.revision(),
                MethodCall {
                    method,
                    receiver,
                    arguments,
                },
                &mut Width::default(),
            )
            .unwrap();
        let result = out.result.unwrap();
        assert_eq!(
            graph(&out.graph, &result),
            action["graph"],
            "ClassDb.{name}"
        );
        let value = selected(&out.graph, &result, action);
        if let Some(key) = action["remember"].as_str() {
            identities.insert(key, reference(&value));
        }
        if let Some(key) = action["sameAs"].as_str() {
            assert_eq!(
                action["sameIdentity"], true,
                "oracle alias must be nonvacuous"
            );
            assert_eq!(
                reference(&value),
                identities[key],
                "ClassDb.{name} identity"
            );
        }
        if let Some(key) = action["differentFrom"].as_str() {
            assert_eq!(
                action["differentIdentity"], true,
                "oracle replacement must be nonvacuous"
            );
            assert_ne!(reference(&value), identities[key]);
        }
    }
    assert_eq!(
        graph(&doc.snapshot().unwrap(), &Atom::Undefined),
        case["populated"]["graph"]
    );
    assert_eq!(raw(&mut doc), case["populated"]["render"]);
    assert_eq!(
        graph(&sibling.snapshot().unwrap(), &Atom::Undefined),
        case["initial"]["graph"]
    );
    assert!(doc.is_constructor_created());
    assert_eq!(case["coreError"]["name"], "TypeError");
    assert_eq!(
        case["coreError"]["message"],
        "render() takes diagram text or a model that parse() returned."
    );
}
