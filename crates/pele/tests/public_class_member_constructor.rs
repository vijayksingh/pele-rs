//! Public ClassMember construction against the actual pinned Pele constructor.
use pele::compat::{public::*, *};
use serde_json::{json, Value};
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-class-member-constructor.json"
    ))
    .unwrap()
}
struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn value(g: &ModelGraph, atom: &Atom) -> Value {
    match atom {
        Atom::Undefined => json!({"undefined":true}),
        Atom::Null => Value::Null,
        Atom::Bool(v) => json!(v),
        Atom::Number(v) => json!(v),
        Atom::String(id) => json!(g.strings[*id as usize].well_formed().unwrap()),
        Atom::Ref(id) => g.nodes[*id as usize]
            .properties()
            .iter()
            .map(|p| {
                (
                    g.strings[p.key as usize].well_formed().unwrap().to_owned(),
                    value(g, &p.value),
                )
            })
            .collect::<serde_json::Map<_, _>>()
            .into(),
        _ => panic!("unexpected fixture atom"),
    }
}
fn fields(g: &ModelGraph) -> Value {
    let Atom::Ref(id) = g.root else {
        panic!("member root")
    };
    g.nodes[id as usize]
        .properties()
        .iter()
        .map(|p| {
            json!([
                g.strings[p.key as usize].well_formed().unwrap(),
                value(g, &p.value),
                p.flags
            ])
        })
        .collect()
}
fn host_value(g: &ModelGraph, a: Atom) -> PublicHostValue {
    match a {
        Atom::Undefined => PublicHostValue::Undefined,
        Atom::Null => PublicHostValue::Null,
        Atom::Bool(v) => PublicHostValue::Bool(v),
        Atom::Number(v) => PublicHostValue::Number(v),
        Atom::String(id) => PublicHostValue::String(g.strings[id as usize].clone()),
        Atom::HostReference(id) | Atom::BuiltinFunction(id) => PublicHostValue::Opaque(id),
        _ => panic!("unexpected constructor reply"),
    }
}
fn set_id(job: &mut PublicConstructionJob, value: &str) {
    let mut graph = job.snapshot().unwrap();
    let key = graph.strings.len() as u32;
    graph.strings.push(JsString::from(value));
    let Atom::Ref(root) = graph.root else {
        panic!("member root")
    };
    let position = graph.nodes[root as usize]
        .properties()
        .iter()
        .position(|p| graph.strings[p.key as usize].is("id"))
        .unwrap();
    graph.nodes[root as usize].properties_mut()[position].value = Atom::String(key);
    job.apply(job.revision().unwrap(), graph).unwrap();
}
fn drive(job: &mut PublicConstructionJob, mode: Option<&str>) -> Vec<Value> {
    let mut trace = Vec::new();
    let mut reply = None;
    loop {
        let step = job.resume(reply.take()).unwrap();
        let operation = match step {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Resource(operation) => {
                reply = Some(operation.run(&mut Host));
                continue;
            }
            DetachedInvocationStep::Host(op) => op,
        };
        let graph = job.snapshot().unwrap();
        reply = Some(match operation {
            PublicHostOperation::Get { receiver, key } => {
                let Atom::Ref(id) = receiver else {
                    panic!("object read")
                };
                let a = graph.nodes[id as usize]
                    .properties()
                    .iter()
                    .find(|p| graph.strings[p.key as usize].is(key))
                    .map(|p| p.value.clone());
                Ok(if let Some(a) = a {
                    host_value(&graph, a)
                } else {
                    PublicHostValue::Opaque(match key {
                        "parseMethod" => 24,
                        "parseAttribute" => 25,
                        _ => panic!("unknown method"),
                    })
                })
            }
            PublicHostOperation::Call {
                function,
                receiver,
                arguments,
                ..
            } => {
                let Atom::HostReference(method) = function else {
                    panic!("host callable")
                };
                if let Some(mode) = mode {
                    let name = match method {
                        24 => "parseMethod",
                        25 => "parseAttribute",
                        _ => panic!("unknown callback"),
                    };
                    trace.push(json!([name, value(&graph, &arguments[0]), fields(&graph)]));
                    if (method == 24 && mode == "method-throw")
                        || (method == 25 && mode == "attribute-throw")
                    {
                        Err(PublicError::HostException(HostExceptionToken(77)))
                    } else {
                        set_id(
                            job,
                            if method == 24 {
                                " + custom "
                            } else {
                                " attribute "
                            },
                        );
                        Ok(if method == 24 {
                            PublicHostValue::Bool(mode == "method-true")
                        } else {
                            PublicHostValue::Undefined
                        })
                    }
                } else {
                    let out = job
                        .invoke(
                            job.revision().unwrap(),
                            MethodCall {
                                method: MethodId(method),
                                receiver,
                                arguments,
                            },
                            &mut Host,
                        )
                        .unwrap();
                    out.result.map(|a| host_value(&out.graph, a))
                }
            }
            PublicHostOperation::Truthy { value } => Ok(PublicHostValue::Bool(match value {
                Atom::Bool(v) => v,
                Atom::Undefined | Atom::Null => false,
                _ => panic!("fixture boolean"),
            })),
            _ => panic!("unexpected host primitive"),
        });
    }
    trace
}
fn factory() -> CapturedPublicConstructor {
    let engine = PublicEngine::with_builtins();
    engine
        .capture_constructor(&DiagramType::Class.into(), PrototypeId(2))
        .unwrap()
        .unwrap()
}
#[test]
fn declared_class_members_use_native_helpers_and_match_pinned_constructor() {
    let factory = factory();
    assert_eq!(factory.argument_count(), 2);
    assert_eq!(factory.arity(), 2);
    assert!(!factory.ignores_arguments());
    assert!(
        factory.construct().is_err(),
        "required body cannot silently initialize without input"
    );
    for case in fixture()["cases"].as_array().unwrap() {
        let mut job = factory
            .begin_construct_with_arguments([
                PublicHostValue::String(JsString::from(case["input"].as_str().unwrap())),
                PublicHostValue::String(JsString::from(case["kind"].as_str().unwrap())),
            ])
            .unwrap();
        drive(&mut job, None);
        let mut doc = job.finish().unwrap().unwrap();
        assert_eq!(doc.revision(), 0);
        assert!(doc.is_constructor_created());
        assert_eq!(fields(&doc.snapshot().unwrap()), case["fields"], "{case}");
        for (method, key) in [(26, "display"), (27, "classifier")] {
            let receiver = doc.snapshot().unwrap().root;
            let out = doc
                .invoke(
                    doc.revision(),
                    MethodCall {
                        method: MethodId(method),
                        receiver,
                        arguments: vec![],
                    },
                    &mut Host,
                )
                .unwrap();
            assert_eq!(value(&out.graph, &out.result.unwrap()), case[key]);
        }
    }
}
#[test]
fn virtual_member_parser_calls_keep_short_circuit_partial_receiver_and_original_throw() {
    let factory = factory();
    for case in fixture()["virtual"].as_array().unwrap() {
        let mode = case["mode"].as_str().unwrap();
        let mut job = factory
            .begin_construct_with_arguments([
                PublicHostValue::String(JsString::from("+input()")),
                PublicHostValue::String(JsString::from(case["kind"].as_str().unwrap())),
            ])
            .unwrap();
        assert_eq!(json!(drive(&mut job, Some(mode))), case["trace"]);
        let out = job.finish_retaining_receiver().unwrap();
        assert_eq!(
            fields(&out.document.snapshot().unwrap()),
            case["fields"],
            "{mode}"
        );
        assert!(out.document.is_constructor_created());
        if case["ok"] == true {
            out.result.unwrap();
            assert_eq!(out.document.revision(), 0);
        } else {
            assert_eq!(case["original"], true);
            assert!(matches!(
                out.result,
                Err(PublicError::HostException(HostExceptionToken(77)))
            ));
        }
    }
}
