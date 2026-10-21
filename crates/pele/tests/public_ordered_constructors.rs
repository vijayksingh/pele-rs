//! Native ordered constructors/defaults/helpers/raw drawings against actual Pele.
use pele::compat::{public::*, *};
use serde_json::{json, Value};
use std::collections::HashMap;
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-ordered-constructors.json"
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
            Atom::BuiltinFunction(_) | Atom::HostReference(43) => json!(["function", 1]),
            Atom::HostReference(42) => json!(["host", "options"]),
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
        Value::Object(v) if v.contains_key("undefined") => Atom::Undefined,
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

fn native_value(g: &ModelGraph, value: &Atom) -> PublicHostValue {
    match value {
        Atom::Undefined => PublicHostValue::Undefined,
        Atom::Null => PublicHostValue::Null,
        Atom::Bool(v) => PublicHostValue::Bool(*v),
        Atom::Number(v) => PublicHostValue::Number(*v),
        Atom::String(id) => PublicHostValue::String(g.strings[*id as usize].clone()),
        _ => panic!("unexpected default value"),
    }
}
fn drive(
    factory: &CapturedPublicConstructor,
    kind: &str,
) -> (Result<PublicDocument, PublicError>, Vec<String>) {
    let input = if kind == "undefined" {
        ConstructorArgument::Undefined
    } else {
        ConstructorArgument::Opaque(42)
    };
    let mut job = factory.begin_construct_with(input).unwrap();
    assert!(job.finish().is_err(), "body cannot finish before its gets");
    let mut reads = vec![];
    let mut reply = None;
    loop {
        match job.resume(reply.take()).unwrap() {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Host(PublicHostOperation::Get { receiver, key }) => {
                let value = match receiver {
                    Atom::Ref(id) => {
                        let graph = job.snapshot().unwrap();
                        let value = graph.nodes[id as usize]
                            .properties()
                            .iter()
                            .find(|p| graph.strings[p.key as usize].is(key))
                            .map(|p| native_value(&graph, &p.value))
                            .unwrap_or(PublicHostValue::Undefined);
                        Ok(value)
                    }
                    Atom::HostReference(42) => {
                        reads.push(key.to_string());
                        if kind == "firstThrow"
                            || kind == "undefinedThrow"
                            || (kind == "secondThrow" && reads.len() == 2)
                        {
                            Err(PublicError::HostException(HostExceptionToken(1)))
                        } else if kind == "values" {
                            Ok(match key {
                                "wrap" => PublicHostValue::Bool(true),
                                "sequenceWrap" => PublicHostValue::Bool(false),
                                "lineOffset" => PublicHostValue::Number(9.0),
                                "warn" => PublicHostValue::Opaque(43),
                                _ => panic!("constructor unexpected key"),
                            })
                        } else {
                            Ok(PublicHostValue::Undefined)
                        }
                    }
                    _ => panic!("constructor unexpected receiver"),
                };
                reply = Some(value);
            }
            _ => panic!("constructor requested non-get operation"),
        }
    }
    let result = job.finish().unwrap();
    assert!(job.finish().is_err());
    (result, reads)
}
fn invoke(doc: &mut PublicDocument, action: &Value) -> Value {
    let name = action["name"].as_str().unwrap();
    let method = doc
        .schema_view()
        .model
        .methods
        .iter()
        .find(|m| m.name == name)
        .unwrap()
        .id;
    let mut g = doc.snapshot().unwrap();
    let args = action["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| argument(&mut g, v))
        .collect();
    let root = g.root.clone();
    doc.apply(doc.revision(), g).unwrap();
    let out = doc
        .invoke(
            doc.revision(),
            MethodCall {
                receiver: root,
                method,
                arguments: args,
            },
            &mut Width::default(),
        )
        .unwrap();
    graph(&out.graph, &out.result.unwrap())
}
fn family_feature(selected: &str) {
    let fixture = fixture();
    let engine = PublicEngine::with_builtins();
    let rows: Vec<_> = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["family"] == selected)
        .collect();
    assert_eq!(rows.len(), 1);
    let factories: Vec<_> = rows
        .iter()
        .map(|r| {
            let id: DiagramId = serde_json::from_value(r["family"].clone()).unwrap();
            engine
                .capture_constructor(&id, PrototypeId(1))
                .unwrap()
                .unwrap()
        })
        .collect();
    drop(engine);
    for (row, factory) in rows.iter().zip(factories) {
        let family = row["family"].as_str().unwrap();
        assert_eq!(factory.argument_count(), 1);
        assert_eq!(
            factory.argument_transfers(),
            &[ConstructorArgumentTransfer::Retain]
        );
        assert_eq!(factory.arity(), 0);
        for case in row["constructorCases"].as_array().unwrap() {
            let kind = case["kind"].as_str().unwrap();
            let (result, reads) = drive(&factory, kind);
            assert_eq!(json!(reads), case["reads"], "{family}/{kind}");
            if case["thrown"] == true {
                assert!(
                    matches!(
                        result,
                        Err(PublicError::HostException(HostExceptionToken(1)))
                    ),
                    "{family}/{kind}"
                );
            } else {
                let doc = result.unwrap();
                assert_eq!(doc.revision(), 0);
                assert!(doc.is_constructor_created());
                assert_eq!(
                    graph(&doc.snapshot().unwrap(), &Atom::Undefined),
                    case["graph"],
                    "{family}/{kind}"
                );
            }
        }
        let mut doc = factory.construct().unwrap();
        assert_eq!(
            graph(&doc.snapshot().unwrap(), &Atom::Undefined),
            row["initial"]["graph"],
            "{family} default"
        );
        assert_eq!(raw(&mut doc), row["initial"]["render"], "{family} emptyraw");
        for action in row["actions"].as_array().unwrap() {
            assert_eq!(
                invoke(&mut doc, action),
                action["graph"],
                "{family}/{}",
                action["name"]
            );
        }
        assert_eq!(
            graph(&doc.snapshot().unwrap(), &Atom::Undefined),
            row["populated"]["graph"],
            "{family} populated"
        );
        assert_eq!(
            raw(&mut doc),
            row["populated"]["render"],
            "{family} cachedraw"
        );
        let mut fresh = drive(&factory, "undefined").0.unwrap();
        for action in row["actions"].as_array().unwrap() {
            invoke(&mut fresh, action);
        }
        assert_eq!(
            graph(&fresh.snapshot().unwrap(), &Atom::Undefined),
            row["freshPopulated"]["graph"],
            "{family} freshpopulated"
        );
        assert_eq!(
            raw(&mut fresh),
            row["freshPopulated"]["render"],
            "{family} freshraw"
        );
    }
}

#[test]
fn sequence_constructor_preserves_ordered_gets_helpers_and_raw_render() {
    family_feature("sequence")
}
#[test]
fn block_constructor_preserves_ordered_gets_helpers_and_raw_render() {
    family_feature("block")
}
#[test]
fn agentflow_constructor_preserves_ordered_gets_helpers_and_cached_raw_render() {
    family_feature("agentflow")
}
