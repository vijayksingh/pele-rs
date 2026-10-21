//! Public AgentFlow construction, live option reads, partial errors and callbacks.
use pele::compat::{public::*, *};
use serde_json::{json, Value};
// JSON text loses integer/f64 storage distinctions, not numeric values.
fn assert_json(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => {
            assert_eq!(a.as_f64().unwrap().to_bits(), b.as_f64().unwrap().to_bits())
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                assert_json(a, b);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (k, a) in a {
                assert_json(a, b.get(k).expect("same key"));
            }
        }
        _ => assert_eq!(actual, expected),
    }
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-agentflow-live-options.json"
    ))
    .unwrap()
}
struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn val(g: &ModelGraph, a: &Atom) -> Value {
    match a {
        Atom::Undefined => Value::Null,
        Atom::Null => Value::Null,
        Atom::Bool(v) => json!(v),
        Atom::Number(v) => json!(v),
        Atom::String(i) => json!(g.strings[*i as usize].well_formed().unwrap()),
        Atom::Ref(i) => match &g.nodes[*i as usize] {
            Node::Array { length, properties } => (0..*length)
                .map(|i| {
                    properties
                        .iter()
                        .find(|p| g.strings[p.key as usize].is(&i.to_string()))
                        .map(|p| val(g, &p.value))
                        .unwrap_or(Value::Null)
                })
                .collect(),
            n => n
                .properties()
                .iter()
                .filter(|p| !matches!(p.value, Atom::Undefined))
                .map(|p| {
                    (
                        g.strings[p.key as usize].well_formed().unwrap().to_owned(),
                        val(g, &p.value),
                    )
                })
                .collect::<serde_json::Map<_, _>>()
                .into(),
        },
        _ => panic!("fixture value"),
    }
}
fn field(g: &ModelGraph, key: &str) -> Atom {
    let Atom::Ref(root) = g.root else {
        panic!("root")
    };
    g.nodes[root as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(Atom::Undefined, |p| p.value.clone())
}
fn count(g: &ModelGraph, key: &str) -> u32 {
    let Atom::Ref(i) = field(g, key) else {
        panic!("array")
    };
    let Node::Array { length, .. } = g.nodes[i as usize] else {
        panic!("array")
    };
    length
}
fn string(g: &mut ModelGraph, s: &str) -> Atom {
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(s));
    Atom::String(id)
}
fn input(g: &mut ModelGraph, v: &Value) -> Atom {
    match v {
        Value::Null => Atom::Undefined,
        Value::Bool(v) => Atom::Bool(*v),
        Value::Number(n) => Atom::Number(n.as_f64().unwrap()),
        Value::String(s) => string(g, s),
        Value::Array(v) => {
            let mut properties = vec![];
            for (i, v) in v.iter().enumerate() {
                let Atom::String(key) = string(g, &i.to_string()) else {
                    unreachable!()
                };
                let value = input(g, v);
                properties.push(Property {
                    key,
                    value,
                    flags: 7,
                });
            }
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Array {
                length: v.len() as u32,
                properties,
            });
            Atom::Ref(id)
        }
        Value::Object(v) => {
            let mut properties = vec![];
            for (k, v) in v {
                let Atom::String(key) = string(g, k) else {
                    unreachable!()
                };
                let value = input(g, v);
                properties.push(Property {
                    key,
                    value,
                    flags: 7,
                });
            }
            let id = g.nodes.len() as u32;
            g.nodes.push(Node::Object { properties });
            Atom::Ref(id)
        }
    }
}
fn call(doc: &mut PublicDocument, method: u32, args: Value) -> MethodCall {
    let mut graph = doc.snapshot().unwrap();
    let arguments = args
        .as_array()
        .unwrap()
        .iter()
        .map(|v| input(&mut graph, v))
        .collect();
    let receiver = graph.root.clone();
    doc.apply(doc.revision(), graph).unwrap();
    MethodCall {
        method: MethodId(method),
        receiver,
        arguments,
    }
}
fn vertex(doc: &mut PublicDocument, id: &str) {
    let c = call(doc, 3, json!([id]));
    doc.invoke(doc.revision(), c, &mut Host)
        .unwrap()
        .result
        .unwrap();
}
fn construct(trace: &mut Vec<Value>, opaque: bool) -> PublicDocument {
    let engine = PublicEngine::with_builtins();
    let f = engine
        .capture_constructor(&DiagramType::Agentflow.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    drop(engine);
    if !opaque {
        return f.construct().unwrap();
    }
    let mut job = f
        .begin_construct_with(PublicHostValue::Opaque(900))
        .unwrap();
    let mut reply = None;
    loop {
        match job.resume(reply.take()).unwrap() {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Host(PublicHostOperation::Get { receiver, key }) => {
                let g = job.snapshot().unwrap();
                reply = Some(Ok(if matches!(receiver, Atom::HostReference(900)) {
                    assert_eq!(key, "lineOffset");
                    trace.push(json!(["get", key]));
                    PublicHostValue::Number(0.)
                } else {
                    assert_eq!(key, "options");
                    assert!(matches!(field(&g, "options"), Atom::HostReference(900)));
                    PublicHostValue::Opaque(900)
                }));
            }
            _ => panic!("constructor primitive"),
        }
    }
    job.finish().unwrap().unwrap()
}
fn run(
    doc: &mut PublicDocument,
    method: u32,
    args: Value,
    mode: &str,
    trace: &mut Vec<Value>,
) -> Result<Atom, PublicError> {
    let call = call(doc, method, args);
    let mut job = doc
        .prepare_detached_invocation(doc.revision(), call)
        .unwrap()
        .expect("live helper");
    let mut reply = None;
    loop {
        let op = match doc
            .resume_detached_invocation(&mut job, reply.take())
            .unwrap()
        {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Host(op) => op,
        };
        let g = doc.snapshot().unwrap();
        reply = Some(match op {
            PublicHostOperation::Get { receiver, key } => {
                if matches!(receiver, Atom::HostReference(900)) {
                    match key {
                        "maxEdges" => {
                            trace.push(json!(["get", key, count(&g, "edges")]));
                            if mode == "max-throw" {
                                Err(PublicError::HostException(HostExceptionToken(77)))
                            } else {
                                Ok(PublicHostValue::Number(2.))
                            }
                        }
                        "inheritDir" => {
                            trace.push(json!(["get", key, val(&g, &field(&g, "subCount"))]));
                            if mode == "inherit-throw" {
                                Err(PublicError::HostException(HostExceptionToken(77)))
                            } else {
                                Ok(PublicHostValue::Bool(true))
                            }
                        }
                        "direction" => {
                            trace.push(json!(["get", key]));
                            Ok(PublicHostValue::String(JsString::from("LR")))
                        }
                        "log" => {
                            trace.push(json!(["get", key, count(&g, "diagnostics")]));
                            Ok(PublicHostValue::Opaque(901))
                        }
                        _ => panic!("unknown option"),
                    }
                } else {
                    assert_eq!(key, "options");
                    Ok(PublicHostValue::Opaque(900))
                }
            }
            PublicHostOperation::Truthy { value } => {
                Ok(PublicHostValue::Bool(matches!(value, Atom::Bool(true))))
            }
            PublicHostOperation::GreaterEqual {
                left: Atom::Number(left),
                right: Atom::Number(right),
            } => Ok(PublicHostValue::Bool(left >= right)),
            PublicHostOperation::ToString {
                value: Atom::String(i),
            } => Ok(PublicHostValue::String(g.strings[i as usize].clone())),
            PublicHostOperation::Call {
                function: Atom::HostReference(901),
                receiver,
                arguments,
                result,
                ..
            } => {
                assert_eq!(result, PublicHostResult::Discard);
                trace.push(json!([
                    "call",
                    "log",
                    matches!(receiver, Atom::HostReference(900)),
                    val(&g, &arguments[0]),
                    val(&g, &arguments[1])
                ]));
                if mode == "log-throw" {
                    Err(PublicError::HostException(HostExceptionToken(77)))
                } else {
                    if mode == "log-reentry" {
                        vertex(doc, "Nested");
                    }
                    Ok(PublicHostValue::Undefined)
                }
            }
            _ => panic!("unexpected option operation"),
        });
    }
    doc.finish_detached_invocation(&mut job).unwrap().result
}
fn state(g: &ModelGraph) -> Value {
    let Atom::Ref(nodes) = field(g, "nodes") else {
        panic!("nodes")
    };
    let Node::Map { entries, .. } = &g.nodes[nodes as usize] else {
        panic!("map")
    };
    json!({"nodes":entries.iter().map(|(k,_)|val(g,k)).collect::<Vec<_>>(),"edges":count(g,"edges"),"subCount":val(g,&field(g,"subCount")),"subgraphs":val(g,&field(g,"subgraphs")),"diagnostics":val(g,&field(g,"diagnostics"))})
}
#[test]
fn live_agentflow_options_keep_partial_state_callbacks_and_reentry() {
    for expected in fixture()["cases"].as_array().unwrap() {
        let mode = expected["mode"].as_str().unwrap();
        let mut trace = vec![];
        let mut doc = construct(&mut trace, true);
        vertex(&mut doc, "A");
        vertex(&mut doc, "B");
        for (i, (method, args)) in [
            (
                4,
                json!(["A","B",{"type":"arrow_point","stroke":"normal","length":1}]),
            ),
            (
                17,
                json!([{"text":"G"},["A","B"],{"text":"Group","type":"text"}]),
            ),
            (33, json!(["test","message",{"nodeId":"A"}])),
        ]
        .into_iter()
        .enumerate()
        {
            let result = run(&mut doc, method, args, mode, &mut trace);
            if expected["operations"][i]["ok"] == true {
                result.unwrap();
            } else {
                assert_eq!(expected["operations"][i]["original"], true);
                assert!(matches!(
                    result,
                    Err(PublicError::HostException(HostExceptionToken(77)))
                ));
            }
        }
        assert_json(&json!(trace), &expected["trace"]);
        assert_json(&state(&doc.snapshot().unwrap()), &expected["state"]);
    }
}
fn raw(doc: &mut PublicDocument) -> Value {
    let out = doc
        .render_raw_operation(
            doc.revision(),
            &Object::default(),
            &RenderSettings::default(),
            &mut Host,
        )
        .unwrap();
    let r = out.result.unwrap();
    json!({"ok":true,"svg":r.svg,"width":format!("{:016x}",r.width.to_bits()),"height":format!("{:016x}",r.height.to_bits())})
}
fn labeled(doc: &mut PublicDocument, id: &str, label: &str) {
    let c = call(doc, 3, json!([id,{"text":label,"type":"text"},"rect"]));
    doc.invoke(doc.revision(), c, &mut Host)
        .unwrap()
        .result
        .unwrap();
}
#[test]
fn constructed_agentflow_retains_first_draw_graph_cache_like_pele() {
    let expected = &fixture()["cache"];
    assert_eq!(expected["sameAfterEdit"], true);
    assert_eq!(expected["newOwnerDiffers"], true);
    let mut doc = construct(&mut vec![], false);
    labeled(&mut doc, "A", "Alpha");
    assert_eq!(raw(&mut doc), expected["first"]);
    labeled(&mut doc, "B", "Beta");
    assert_eq!(raw(&mut doc), expected["second"]);
    let mut fresh = construct(&mut vec![], false);
    labeled(&mut fresh, "A", "Alpha");
    labeled(&mut fresh, "B", "Beta");
    assert_eq!(raw(&mut fresh), expected["freshBoth"]);
}

#[test]
fn fresh_agentflow_raw_keeps_plain_and_shadowed_helper_errors() {
    for expected in fixture()["rawGuards"].as_array().unwrap() {
        let mut doc = construct(&mut vec![], false);
        labeled(&mut doc, "A", "Alpha");
        let mut graph = doc.snapshot().unwrap();
        let Atom::Ref(root) = graph.root else {
            panic!("root")
        };
        if expected["kind"] == "plain" {
            let properties = graph.nodes[root as usize].properties().to_vec();
            graph.nodes[root as usize] = Node::Object { properties };
        } else {
            let Atom::String(key) = string(&mut graph, "vertexKind") else {
                unreachable!()
            };
            graph.nodes[root as usize].properties_mut().push(Property {
                key,
                value: Atom::Null,
                flags: 7,
            });
        }
        doc.apply(doc.revision(), graph).unwrap();
        let outcome = doc
            .render_raw_operation(
                doc.revision(),
                &Object::default(),
                &RenderSettings::default(),
                &mut Host,
            )
            .unwrap();
        let err = outcome.result.unwrap_err();
        assert_eq!(expected["result"]["ok"], false);
        let PublicError::JavaScript { name, message } = err else {
            panic!("JavaScript error")
        };
        assert!(matches!(name, JavaScriptErrorKind::TypeError));
        assert_eq!(expected["result"]["name"], "TypeError");
        assert_eq!(
            message.well_formed().unwrap(),
            expected["result"]["message"].as_str().unwrap()
        );
    }
}
