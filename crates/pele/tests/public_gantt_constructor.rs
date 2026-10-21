//! Captured Gantt construction and live warning behavior against pinned Pele.
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
fn label(value: &Atom) -> Value {
    match value {
        Atom::Undefined => json!("undefined"),
        Atom::Null => json!("null"),
        Atom::Number(n) => json!(n),
        Atom::HostReference(11) => json!("options1"),
        Atom::HostReference(12) => json!("options2"),
        Atom::HostReference(26) => json!(42),
        other => panic!("unexpected returned value {other:?}"),
    }
}
// JSON distinguishes integer and float syntax. Compare actual IEEE values so
// 42 and 42.0 agree while signed zero and every other bit difference remain.
fn trace_numbers(value: &Value) -> Value {
    match value {
        Value::Number(number) => json!([
            "number",
            format!("{:016x}", number.as_f64().unwrap().to_bits())
        ]),
        Value::Array(values) => Value::Array(values.iter().map(trace_numbers).collect()),
        other => other.clone(),
    }
}
fn original() -> PublicError {
    PublicError::HostException(HostExceptionToken(1))
}
fn javascript(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: JsString::from(message),
    }
}
#[test]
fn captured_gantt_retains_options_and_warning_evaluation_errors_and_reentry() {
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/public-gantt-constructor.json"
    ))
    .unwrap();
    assert_eq!(
        expected["oracleSHA256"],
        "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d"
    );
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramId::from(DiagramType::Gantt), PrototypeId(1))
        .unwrap()
        .unwrap();
    assert!(!factory.ignores_arguments());
    drop(engine);
    let mut fresh = factory.construct().unwrap();
    assert!(fresh.is_constructor_created());
    assert_eq!(
        graph(&fresh.snapshot().unwrap(), &Atom::Undefined),
        expected["initial"]
    );
    let drawing = fresh
        .render_raw_operation(
            fresh.revision(),
            &Object::default(),
            &RenderSettings {
                now: Some(1704067200000.0),
                ..Default::default()
            },
            &mut Host,
        )
        .unwrap()
        .result
        .unwrap();
    assert_eq!(drawing.svg, expected["drawing"]["svg"].as_str().unwrap());
    assert_eq!(
        drawing.width,
        expected["drawing"]["width"].as_f64().unwrap()
    );
    assert_eq!(
        drawing.height,
        expected["drawing"]["height"].as_f64().unwrap()
    );
    let graph = fresh.snapshot().unwrap();
    assert!(matches!(field(&graph, "options"), Atom::Ref(_)));
    for row in expected["cases"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        let mut doc = factory
            .construct_with(ConstructorArgument::Opaque(10))
            .unwrap();
        assert!(matches!(
            field(&doc.snapshot().unwrap(), "options"),
            Atom::HostReference(10)
        ));
        let root = doc.snapshot().unwrap().root;
        let producer = match id {
            "producer-noncallable-null" => Atom::Null,
            "producer-noncallable-number" => Atom::Number(3.0),
            _ => Atom::HostReference(23),
        };
        let mut job = doc
            .prepare_detached_invocation(
                doc.revision(),
                MethodCall {
                    receiver: root.clone(),
                    method: MethodId(24),
                    arguments: vec![producer],
                },
            )
            .unwrap()
            .unwrap();
        let mut trace = vec![];
        let mut options = 0;
        let mut warns = 0;
        let mut reply = None;
        loop {
            match doc
                .resume_detached_invocation(&mut job, reply.take())
                .unwrap()
            {
                DetachedInvocationStep::Complete => break,
                DetachedInvocationStep::Resource(operation) => {
                    reply = Some(operation.run(&mut Host));
                    continue;
                }
                DetachedInvocationStep::Host(PublicHostOperation::Get { receiver, key }) => {
                    let value = if key == "options" {
                        assert!(
                            matches!((&receiver, &root), (Atom::Ref(a), Atom::Ref(b)) if a == b)
                        );
                        trace.push(json!(["get", "model", "options"]));
                        options += 1;
                        PublicHostValue::Opaque(if options == 1 { 11 } else { 12 })
                    } else {
                        assert_eq!(key, "warn");
                        warns += 1;
                        trace.push(json!(["get", label(&receiver), "warn"]));
                        if warns == 1 {
                            match id {
                                "lazy-undefined" => PublicHostValue::Undefined,
                                "lazy-null" => PublicHostValue::Null,
                                "lazy-false" => PublicHostValue::Bool(false),
                                "lazy-zero" => PublicHostValue::Number(0.0),
                                "lazy-nan" => PublicHostValue::Number(f64::NAN),
                                "lazy-empty" => PublicHostValue::String(JsString::from("")),
                                "lazy-bigint-zero" => PublicHostValue::Opaque(25),
                                _ => PublicHostValue::Opaque(21),
                            }
                        } else {
                            if id == "second-get-throw" {
                                reply = Some(Err(original()));
                                continue;
                            }
                            match id {
                                "second-noncallable-false" => PublicHostValue::Bool(false),
                                "second-noncallable-object" => PublicHostValue::Opaque(24),
                                _ => PublicHostValue::Opaque(22),
                            }
                        }
                    };
                    reply = Some(Ok(value));
                }
                DetachedInvocationStep::Host(PublicHostOperation::Call {
                    function,
                    receiver,
                    arguments,
                    not_callable_error,
                    result,
                }) => match function {
                    Atom::HostReference(23) => {
                        assert_eq!(result, PublicHostResult::Retain);
                        assert_eq!(not_callable_error, "e is not a function");
                        assert!(arguments.is_empty());
                        trace.push(json!(["call", "producer", label(&receiver)]));
                        if id == "producer-reentry" {
                            add_section(&mut doc, "nested producer");
                        }
                        reply = Some(if id.starts_with("producer-throw-") {
                            Err(original())
                        } else {
                            Ok(PublicHostValue::Opaque(26))
                        });
                    }
                    Atom::HostReference(22) => {
                        assert_eq!(result, PublicHostResult::Discard);
                        assert_eq!(not_callable_error, "this.options.warn is not a function");
                        assert_eq!(arguments.len(), 1);
                        trace.push(json!([
                            "call",
                            "warn",
                            label(&receiver),
                            label(&arguments[0])
                        ]));
                        if id == "warn-reentry" {
                            add_section(&mut doc, "nested warning");
                        }
                        reply = Some(if id == "warn-throw" {
                            Err(original())
                        } else {
                            Ok(PublicHostValue::Undefined)
                        });
                    }
                    _ => {
                        reply = Some(Err(javascript(not_callable_error)));
                    }
                },
                DetachedInvocationStep::Host(PublicHostOperation::Truthy { value }) => {
                    let truthy = match value {
                        Atom::Undefined | Atom::Null | Atom::HostReference(25) => false,
                        Atom::Bool(v) => v,
                        Atom::Number(n) => n != 0.0 && !n.is_nan(),
                        Atom::String(s) => {
                            !doc.snapshot().unwrap().strings[s as usize].0.is_empty()
                        }
                        _ => true,
                    };
                    reply = Some(Ok(PublicHostValue::Bool(truthy)));
                }
                _ => panic!("unexpected host operation for warning"),
            }
        }
        let outcome = doc.finish_detached_invocation(&mut job).unwrap();
        match &outcome.result {
            Ok(Atom::Undefined) => assert_eq!(row["thrown"], false, "{id}"),
            Err(PublicError::HostException(HostExceptionToken(1))) => assert!(
                row["error"]
                    .as_str()
                    .is_some_and(|s| s.starts_with("original-")),
                "{id}"
            ),
            Err(PublicError::JavaScript { name, message }) => {
                assert_eq!(*name, JavaScriptErrorKind::TypeError);
                assert_eq!(
                    message.well_formed().unwrap(),
                    row["error"]["message"].as_str().unwrap(),
                    "{id}"
                );
            }
            other => panic!("{id}: {other:?}"),
        }
        drop(outcome);
        assert_eq!(
            trace_numbers(&json!(trace)),
            trace_numbers(&row["trace"]),
            "{id}"
        );
        assert_eq!(json!(options), row["optionReads"], "{id}");
        assert_eq!(json!(warns), row["warnReads"], "{id}");
        let graph = doc.snapshot().unwrap();
        let Atom::Ref(sections) = field(&graph, "sections") else {
            panic!("sections")
        };
        let actual: Vec<_> = graph.nodes[*sections as usize]
            .properties()
            .iter()
            .filter_map(|p| match p.value {
                Atom::String(s) => Some(graph.strings[s as usize].well_formed().unwrap()),
                _ => None,
            })
            .collect();
        assert_eq!(json!(actual), row["sections"], "{id}");
        assert!(matches!(
            field(&fresh.snapshot().unwrap(), "options"),
            Atom::Ref(_)
        ));
    }
}
