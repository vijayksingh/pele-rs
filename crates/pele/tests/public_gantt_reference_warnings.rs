//! Gantt reference warnings: lazy native message, callback effects and task aliases.
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
            Atom::HostReference(10) => json!(["host", "options"]),
            Atom::BuiltinFunction(id) => {
                assert_eq!(
                    *id, 0,
                    "Only outline getType is an own function in this fixture"
                );
                json!(["function", 2])
            }
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

struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn field<'a>(g: &'a ModelGraph, name: &str) -> &'a Atom {
    let Atom::Ref(root) = g.root else {
        panic!("root")
    };
    &g.nodes[root as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(name))
        .unwrap()
        .value
}
fn add_section(doc: &mut PublicDocument, section: &str) {
    let mut graph = doc.snapshot().unwrap();
    let text = graph.strings.len() as u32;
    graph.strings.push(JsString::from(section));
    doc.apply(doc.revision(), graph).unwrap();
    let result = doc
        .invoke(
            doc.revision(),
            MethodCall {
                receiver: doc.snapshot().unwrap().root,
                method: MethodId(14),
                arguments: vec![Atom::String(text)],
            },
            &mut Host,
        )
        .unwrap();
    assert!(result.result.is_ok());
}

fn text(doc: &mut PublicDocument, s: JsString) -> Atom {
    let mut g = doc.snapshot().unwrap();
    let id = g.strings.len() as u32;
    g.strings.push(s);
    doc.apply(doc.revision(), g).unwrap();
    Atom::String(id)
}
fn add_task(doc: &mut PublicDocument, name: &str, data: &str) {
    let a = text(doc, JsString::from(name));
    let b = text(doc, JsString::from(data));
    doc.invoke(
        doc.revision(),
        MethodCall {
            receiver: doc.snapshot().unwrap().root,
            method: MethodId(15),
            arguments: vec![a, b],
        },
        &mut Host,
    )
    .unwrap()
    .result
    .unwrap();
}
fn original() -> PublicError {
    PublicError::HostException(HostExceptionToken(77))
}
fn run(
    doc: &mut PublicDocument,
    id: &str,
    list: &str,
    recovery: bool,
) -> (Atom, Value, Vec<Value>) {
    let keyword = if recovery {
        text(doc, JsString::from("after"))
    } else {
        Atom::HostReference(30)
    };
    let list = text(doc, JsString::from(list));
    let receiver = doc.snapshot().unwrap().root;
    let mut job = doc
        .prepare_detached_invocation(
            doc.revision(),
            MethodCall {
                receiver,
                method: MethodId(27),
                arguments: vec![keyword, list],
            },
        )
        .unwrap()
        .expect("owned references");
    let mut trace = vec![];
    let mut reads = 0;
    let mut reply = None;
    loop {
        let op = match doc
            .resume_detached_invocation(&mut job, reply.take())
            .unwrap()
        {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Resource(operation) => {
                reply = Some(operation.run(&mut Host));
                continue;
            }
            DetachedInvocationStep::Host(op) => op,
        };
        reply = Some(match op {
            PublicHostOperation::Get { receiver, key } => {
                if key == "options" {
                    assert!(matches!(receiver, Atom::Ref(_)));
                    Ok(PublicHostValue::Opaque(10))
                } else {
                    assert_eq!(key, "warn");
                    assert!(matches!(receiver, Atom::HostReference(10)));
                    if recovery {
                        Ok(PublicHostValue::Undefined)
                    } else {
                        reads += 1;
                        trace.push(json!(["get", reads]));
                        if (id == "first-get-throw" && reads == 1)
                            || (id == "second-get-throw" && reads == 2)
                        {
                            Err(original())
                        } else if id == "warn-false" {
                            Ok(PublicHostValue::Bool(false))
                        } else if id == "second-not-callable" && reads == 2 {
                            Ok(PublicHostValue::Number(0.))
                        } else {
                            Ok(PublicHostValue::Opaque(20))
                        }
                    }
                }
            }
            PublicHostOperation::Truthy { value } => Ok(PublicHostValue::Bool(matches!(
                value,
                Atom::HostReference(_)
            ))),
            PublicHostOperation::ToString { value } => {
                assert!(!recovery);
                assert!(matches!(value, Atom::HostReference(30)));
                trace.push(json!(["keyword"]));
                if id == "keyword-throw" {
                    Err(original())
                } else {
                    if id == "keyword-reentry" {
                        add_section(doc, "Nested");
                    }
                    let mut value: Vec<_> = "after".encode_utf16().collect();
                    value.push(0xd800);
                    Ok(PublicHostValue::String(JsString(value)))
                }
            }
            PublicHostOperation::Call {
                function,
                receiver,
                arguments,
                result,
                not_callable_error,
            } => {
                assert_eq!(result, PublicHostResult::Discard);
                assert_eq!(not_callable_error, "this.options.warn is not a function");
                assert!(matches!(receiver, Atom::HostReference(10)));
                assert_eq!(arguments.len(), 1);
                if matches!(function, Atom::Number(0.)) {
                    Err(PublicError::JavaScript {
                        name: JavaScriptErrorKind::TypeError,
                        message: JsString::from(not_callable_error),
                    })
                } else {
                    assert!(matches!(function, Atom::HostReference(20)));
                    let g = doc.snapshot().unwrap();
                    let Atom::String(message_id) = arguments[0] else {
                        panic!("message")
                    };
                    trace.push(json!(["call", true, g.strings[message_id as usize].0]));
                    if id == "warn-reentry" {
                        add_task(doc, "Resolved", ":missing,2024-01-01,1d");
                        let mut g = doc.snapshot().unwrap();
                        let Atom::Ref(tasks) = field(&g, "tasks") else {
                            panic!("tasks")
                        };
                        let Atom::Ref(first) = g.nodes[*tasks as usize].properties()[0].value
                        else {
                            panic!("task")
                        };
                        let value = g.strings.len() as u32;
                        g.strings.push(JsString::from("Changed"));
                        let strings = &g.strings;
                        g.nodes[first as usize]
                            .properties_mut()
                            .iter_mut()
                            .find(|p| strings[p.key as usize].is("task"))
                            .unwrap()
                            .value = Atom::String(value);
                        doc.apply(doc.revision(), g).unwrap();
                    }
                    if id == "warn-throw" {
                        Err(original())
                    } else {
                        Ok(PublicHostValue::Undefined)
                    }
                }
            }
            _ => panic!("unexpected references operation"),
        });
    }
    let outcome = doc.finish_detached_invocation(&mut job).unwrap();
    match outcome.result {
        Ok(value) => (value, Value::Null, trace),
        Err(PublicError::HostException(HostExceptionToken(77))) => {
            (Atom::Undefined, json!("original"), trace)
        }
        Err(PublicError::JavaScript { name, message }) => {
            assert_eq!(name, JavaScriptErrorKind::TypeError);
            (
                Atom::Undefined,
                json!({"name":"TypeError","message":message.well_formed().unwrap()}),
                trace,
            )
        }
        Err(e) => panic!("unexpected invocation error {e:?}"),
    }
}
fn returned_alias(g: &ModelGraph, result: &Atom, first: u32) -> Value {
    if let Atom::Ref(id) = result {
        let value = &g.nodes[*id as usize].properties()[0].value;
        json!(matches!(value,Atom::Ref(id)if *id==first))
    } else {
        Value::Null
    }
}
#[test]
fn gantt_reference_warnings_preserve_lazy_message_order_aliases_partial_effects_and_recovery() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/public-gantt-reference-warnings.json"
    ))
    .unwrap();
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramType::Gantt.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    drop(engine);
    for row in fixture["cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let list = row["list"].as_str().unwrap();
        let mut doc = factory
            .construct_with(ConstructorArgument::Opaque(10))
            .unwrap();
        add_task(&mut doc, "Alpha", ":a,2024-01-01,1d");
        add_task(&mut doc, "Beta", ":b,2024-01-02,1d");
        let g = doc.snapshot().unwrap();
        let Atom::Ref(tasks) = field(&g, "tasks") else {
            panic!("tasks")
        };
        let Atom::Ref(first) = g.nodes[*tasks as usize].properties()[0].value else {
            panic!("task")
        };
        if matches!(id, "shadowed-known" | "shadowed-warn") {
            let mut g = doc.snapshot().unwrap();
            let Atom::Ref(root) = g.root else {
                panic!("root")
            };
            let key = g.strings.len() as u32;
            g.strings.push(JsString::from("warn"));
            g.nodes[root as usize].properties_mut().push(Property {
                key,
                flags: 7,
                value: Atom::Null,
            });
            doc.apply(doc.revision(), g).unwrap();
        }
        let (result, error, trace) = run(&mut doc, id, list, false);
        let g = doc.snapshot().unwrap();
        assert_eq!(graph(&g, &result), row["first"]["graph"], "{id} graph");
        assert_eq!(error, row["first"]["error"], "{id} error");
        assert_eq!(
            trace,
            row["first"]["trace"].as_array().unwrap().clone(),
            "{id} trace"
        );
        assert_eq!(
            returned_alias(&g, &result, first),
            row["first"]["returnedAlias"],
            "{id} alias"
        );
        if matches!(id, "shadowed-known" | "shadowed-warn") {
            let mut g = doc.snapshot().unwrap();
            let Atom::Ref(root) = g.root else {
                panic!("root")
            };
            let strings = &g.strings;
            g.nodes[root as usize]
                .properties_mut()
                .retain(|p| !strings[p.key as usize].is("warn"));
            doc.apply(doc.revision(), g).unwrap();
        }
        let (result, error, _) = run(&mut doc, id, list, true);
        assert_eq!(error, Value::Null);
        let g = doc.snapshot().unwrap();
        assert_eq!(
            graph(&g, &result),
            row["recovery"]["graph"],
            "{id} recovery"
        );
        assert_eq!(
            returned_alias(&g, &result, first),
            row["recovery"]["returnedAlias"]
        );
    }
}
