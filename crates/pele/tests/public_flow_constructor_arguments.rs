//! Captured Flow/Swimlane options, ordered live helpers, cancellation and reentry.
use pele::compat::{public::*, *};
use serde_json::{json, Value};
use std::collections::HashMap;
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
            Atom::HostReference(_) => json!(["options"]),
            _ => panic!("fixture atom kind"),
        }
    }
    let mut ids = HashMap::new();
    let mut nodes = Vec::new();
    let root = atom(g, &g.root, &mut ids, &mut nodes);
    let result = atom(g, result, &mut ids, &mut nodes);
    {
        let _ = result;
        json!({"root":root,"nodes":nodes})
    }
}

struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn original() -> PublicError {
    PublicError::HostException(HostExceptionToken(7))
}
fn field<'a>(g: &'a ModelGraph, key: &str) -> &'a Atom {
    let Atom::Ref(root) = g.root else {
        panic!("root")
    };
    &g.nodes[root as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .unwrap()
        .value
}
fn text(g: &ModelGraph, a: &Atom) -> String {
    let Atom::String(id) = a else { panic!("text") };
    g.strings[*id as usize].well_formed().unwrap().to_owned()
}
fn call(
    doc: &mut PublicDocument,
    method: u32,
    start: &str,
    end: Option<&str>,
    styled: bool,
) -> MethodCall {
    let mut g = doc.snapshot().unwrap();
    let mut args = vec![];
    for value in std::iter::once(start).chain(end) {
        let id = g.strings.len() as u32;
        g.strings.push(JsString::from(value));
        args.push(Atom::String(id));
    }
    if styled {
        let id = g.nodes.len() as u32;
        g.nodes.push(Node::Array {
            length: 0,
            properties: vec![],
        });
        args.extend([Atom::Undefined, Atom::Undefined, Atom::Ref(id)]);
    }
    let receiver = g.root.clone();
    doc.apply(doc.revision(), g).unwrap();
    MethodCall {
        receiver,
        method: MethodId(method),
        arguments: args,
    }
}
fn subcall(doc: &mut PublicDocument) -> MethodCall {
    let mut g = doc.snapshot().unwrap();
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Array {
        length: 0,
        properties: vec![],
    });
    let receiver = g.root.clone();
    doc.apply(doc.revision(), g).unwrap();
    MethodCall {
        receiver,
        method: MethodId(15),
        arguments: vec![Atom::Undefined, Atom::Ref(id), Atom::Undefined],
    }
}
fn nested_vertex(doc: &mut PublicDocument) {
    let c = call(doc, 1, "B", None, false);
    assert!(doc
        .invoke(doc.revision(), c, &mut Host)
        .unwrap()
        .result
        .is_ok());
}
fn drive(
    doc: &mut PublicDocument,
    c: MethodCall,
    id: &str,
    trace: &mut Vec<Value>,
    entered: &mut bool,
) -> Result<Atom, PublicError> {
    let mut job = doc
        .prepare_detached_invocation(doc.revision(), c)
        .unwrap()
        .unwrap();
    let mut reply = None;
    loop {
        let before = doc.revision();
        let step = doc
            .resume_detached_invocation(&mut job, reply.take())
            .unwrap();
        assert!(doc.revision() > before);
        let value = match step {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Resource(operation) => {
                reply = Some(operation.run(&mut Host));
                continue;
            }
            DetachedInvocationStep::Host(PublicHostOperation::Get { receiver, key }) => {
                if key == "options" {
                    assert!(matches!(receiver, Atom::Ref(_)));
                    trace.push(json!(["get", "model", "options"]));
                    Ok(PublicHostValue::Opaque(10))
                } else {
                    assert!(matches!(receiver, Atom::HostReference(10)));
                    trace.push(json!(["get", "options", key]));
                    if id == format!("{}-get-throw", if key == "warn" { "warn" } else { "max" }) {
                        Err(original())
                    } else if key == "warn" {
                        Ok(if id == "warn-null" {
                            PublicHostValue::Null
                        } else {
                            PublicHostValue::Opaque(20)
                        })
                    } else if key == "inheritDir" {
                        if id == "sub-inherit-throw" {
                            Err(original())
                        } else {
                            if id == "sub-root-direction" {
                                let c = call(doc, 8, "TB", None, false);
                                assert!(doc
                                    .invoke(doc.revision(), c, &mut Host)
                                    .unwrap()
                                    .result
                                    .is_ok());
                            }
                            Ok(if id == "sub-bigint-zero" {
                                PublicHostValue::Opaque(40)
                            } else {
                                PublicHostValue::Bool(id != "sub-false")
                            })
                        }
                    } else if key == "direction" {
                        if id == "sub-direction-throw" {
                            Err(original())
                        } else {
                            Ok(PublicHostValue::String(JsString::from("RL")))
                        }
                    } else {
                        assert_eq!(key, "maxEdges");
                        if id == "max-reentry" && !*entered {
                            *entered = true;
                            let c = call(doc, 2, "C", Some("D"), false);
                            assert!(drive(doc, c, id, trace, entered).is_ok());
                        }
                        Ok(match id {
                            "max-zero" => PublicHostValue::Number(0.),
                            "max-one" | "max-reentry" => PublicHostValue::Number(1.),
                            s if s.starts_with("max-coerce") || s == "max-string-throw" => {
                                PublicHostValue::Opaque(30)
                            }
                            _ => PublicHostValue::Undefined,
                        })
                    }
                }
            }
            DetachedInvocationStep::Host(PublicHostOperation::Call {
                function,
                receiver,
                arguments,
                ..
            }) => {
                assert!(matches!(function, Atom::HostReference(20)));
                assert!(matches!(receiver, Atom::HostReference(10)));
                let message = text(&doc.snapshot().unwrap(), &arguments[0]);
                trace.push(json!(["call", "warn", true, message]));
                if id == "warn-reentry" || id == "warn-call-throw" {
                    nested_vertex(doc);
                }
                if id == "warn-call-throw" {
                    Err(original())
                } else {
                    Ok(PublicHostValue::Undefined)
                }
            }
            DetachedInvocationStep::Host(PublicHostOperation::GreaterEqual { left, right }) => {
                let Atom::Number(left) = left else {
                    panic!("left")
                };
                let n = match right {
                    Atom::Number(n) => n,
                    Atom::HostReference(30) => {
                        trace.push(json!(["valueOf"]));
                        if id == "max-coerce-throw" {
                            reply = Some(Err(original()));
                            continue;
                        }
                        if id == "max-coerce-reentry" {
                            nested_vertex(doc);
                        }
                        0.
                    }
                    _ => panic!("right"),
                };
                Ok(PublicHostValue::Bool(left >= n))
            }
            DetachedInvocationStep::Host(PublicHostOperation::ToString { value }) => {
                if matches!(value, Atom::HostReference(30)) {
                    trace.push(json!(["toString"]));
                    if id == "max-string-throw" {
                        reply = Some(Err(original()));
                        continue;
                    }
                }
                let value = match value {
                    Atom::Number(1.) => "1",
                    _ => "0",
                };
                Ok(PublicHostValue::String(JsString::from(value)))
            }
            DetachedInvocationStep::Host(PublicHostOperation::Truthy { value }) => {
                Ok(PublicHostValue::Bool(matches!(value, Atom::Bool(true))))
            }
            _ => panic!("unexpected operation"),
        };
        reply = Some(value);
    }
    let before = doc.revision();
    let out = doc.finish_detached_invocation(&mut job).unwrap();
    assert_eq!(out.revision, before);
    out.result
}
fn error(e: Result<Atom, PublicError>) -> Value {
    match e {
        Ok(_) => Value::Null,
        Err(PublicError::HostException(HostExceptionToken(7))) => json!({"original":true}),
        Err(PublicError::Pele(e)) => json!({"name":"PeleError","message":e.message,"code":e.code}),
        Err(other) => panic!("unexpected {other:?}"),
    }
}
#[test]
fn flow_and_swimlane_retained_options_live_helpers_match_pinned_oracle() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/public-flow-arguments.json")).unwrap();
    let engine = PublicEngine::with_builtins();
    for row in fixture["cases"].as_array().unwrap() {
        let family = row["family"].as_str().unwrap();
        let id = row["id"].as_str().unwrap();
        let factory = engine
            .capture_constructor(
                &DiagramId::from(if family == "flowchart" {
                    DiagramType::Flowchart
                } else {
                    DiagramType::Swimlane
                }),
                PrototypeId(1),
            )
            .unwrap()
            .unwrap();
        assert!(!factory.ignores_arguments());
        assert_eq!(factory.live_fields()[0].key, "options");
        let mut doc = factory
            .construct_with(PublicHostValue::Opaque(if id == "warn-replacement" {
                9
            } else {
                10
            }))
            .unwrap();
        assert!(doc.is_constructor_created());
        assert_eq!(
            graph(&doc.snapshot().unwrap(), &Atom::Undefined),
            row["initial"],
            "{family}/{id} initial"
        );
        let c = if id.starts_with("warn") {
            call(&mut doc, 1, "A", None, true)
        } else if id.starts_with("sub") {
            subcall(&mut doc)
        } else {
            call(&mut doc, 2, "A", Some("B"), false)
        };
        let mut trace = vec![];
        let result = drive(&mut doc, c, id, &mut trace, &mut false);
        assert_eq!(
            trace,
            row["trace"].as_array().unwrap().clone(),
            "{family}/{id} trace"
        );
        assert_eq!(error(result), row["error"], "{family}/{id} error");
        assert_eq!(
            graph(&doc.snapshot().unwrap(), &Atom::Undefined),
            row["state"],
            "{family}/{id} state"
        );
    }
}
#[test]
fn captured_options_scalars_cancellation_and_foreign_jobs_preserve_current_revision() {
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramId::from(DiagramType::Flowchart), PrototypeId(1))
        .unwrap()
        .unwrap();
    drop(engine);
    for value in [
        PublicHostValue::Null,
        PublicHostValue::Bool(false),
        PublicHostValue::Number(-0.),
        PublicHostValue::String(JsString(vec![0xd800])),
    ] {
        let doc = factory.construct_with(value.clone()).unwrap();
        let g = doc.snapshot().unwrap();
        match (value, field(&g, "options")) {
            (PublicHostValue::Null, Atom::Null)
            | (PublicHostValue::Bool(false), Atom::Bool(false)) => {}
            (PublicHostValue::Number(n), Atom::Number(m)) => assert_eq!(n.to_bits(), m.to_bits()),
            (PublicHostValue::String(s), Atom::String(id)) => {
                assert_eq!(s.0, g.strings[*id as usize].0)
            }
            _ => panic!("option changed"),
        }
    }
    let mut a = factory.construct().unwrap();
    let mut b = factory.construct().unwrap();
    let c = call(&mut a, 2, "A", Some("B"), false);
    let mut job = a
        .prepare_detached_invocation(a.revision(), c)
        .unwrap()
        .unwrap();
    let before = b.revision();
    assert!(b.resume_detached_invocation(&mut job, None).is_err());
    assert_eq!(b.revision(), before);
    let before = a.revision();
    assert!(matches!(
        a.resume_detached_invocation(&mut job, None).unwrap(),
        DetachedInvocationStep::Host(_)
    ));
    assert!(a.revision() > before);
    let accepted = a.revision();
    drop(job);
    assert_eq!(a.revision(), accepted);
    let c = call(&mut a, 1, "B", None, false);
    assert!(a.invoke(a.revision(), c, &mut Host).unwrap().result.is_ok());
}
