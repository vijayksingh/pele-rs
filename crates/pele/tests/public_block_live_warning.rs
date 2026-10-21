//! Block warning callbacks through public construction and retained invocation.
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
        "../../../fixtures/public-block-live-warning.json"
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
fn set(g: &mut ModelGraph, receiver: &Atom, key: &str, value: Atom) {
    let Atom::Ref(id) = receiver else {
        panic!("receiver")
    };
    if let Some(at) = g.nodes[*id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key))
    {
        g.nodes[*id as usize].properties_mut()[at].value = value;
    } else {
        let Atom::String(key) = string(g, key) else {
            unreachable!()
        };
        g.nodes[*id as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        });
    }
}
fn at(g: &ModelGraph, receiver: &Atom, key: &str) -> Atom {
    let Atom::Ref(i) = receiver else {
        panic!("record")
    };
    g.nodes[*i as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(Atom::Undefined, |p| p.value.clone())
}
fn list(g: &ModelGraph, a: &Atom) -> Vec<Atom> {
    let Atom::Ref(id) = a else { panic!("list") };
    let Node::Array { length, .. } = g.nodes[*id as usize] else {
        panic!("array")
    };
    (0..length).map(|i| at(g, a, &i.to_string())).collect()
}
fn same_ref(a: &Atom, b: &Atom) -> bool {
    matches!((a,b),(Atom::Ref(a),Atom::Ref(b))if a==b)
}
fn state(g: &ModelGraph, statements: &Atom) -> Value {
    let root = field(g, "root");
    let blocks = field(g, "blocks");
    let Atom::Ref(id) = field(g, "byId") else {
        panic!("map")
    };
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        panic!("map")
    };
    let Atom::Ref(pending) = field(g, "pending") else {
        panic!("set")
    };
    let Node::Set { values, .. } = &g.nodes[pending as usize] else {
        panic!("set")
    };
    let aliases: Vec<Value> = list(g, statements)
        .into_iter()
        .filter_map(|n| {
            let id = at(g, &n, "id");
            entries
                .iter()
                .find(|(k, _)| val(g, k) == val(g, &id))
                .map(|(_, v)| json!([val(g, &id), same_ref(v, &n)]))
        })
        .collect();
    json!({"root":val(g,&root),"blocks":val(g,&blocks),"edges":val(g,&field(g,"edges")),"byId":entries.iter().map(|(k,v)|json!([val(g,k),val(g,v)])).collect::<Vec<_>>(),"pending":values.iter().map(|a|val(g,a)).collect::<Vec<_>>(),"ids":val(g,&field(g,"ids")),"colors":val(g,&field(g,"colors")),"spaces":val(g,&field(g,"spaces")),"statements":val(g,statements),"blocksAliasRoot":same_ref(&blocks,&at(g,&root,"children")),"byIdAlias":aliases})
}
fn construct() -> PublicDocument {
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramType::Block.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    drop(engine);
    let mut job = factory
        .begin_construct_with(PublicHostValue::Opaque(40))
        .unwrap();
    match job.resume(None).unwrap() {
        DetachedInvocationStep::Host(PublicHostOperation::Get {
            receiver: Atom::HostReference(40),
            key: "warn",
        }) => {}
        _ => panic!("constructor warn getter"),
    }
    assert!(matches!(
        job.resume(Some(Ok(PublicHostValue::Opaque(41)))).unwrap(),
        DetachedInvocationStep::Complete
    ));
    job.finish().unwrap().unwrap()
}
fn statements() -> Value {
    json!([{"type":"column-setting","columns":2},{"id":"A","type":"square","widthInColumns":4},{"id":"B","type":"square","widthInColumns":3},{"id":"G","type":"composite","widthInColumns":3,"children":[{"type":"column-setting","columns":1},{"id":"C","type":"square","widthInColumns":2}]}])
}
fn invoke(doc: &mut PublicDocument, method: u32, args: Vec<Atom>) -> Result<Atom, PublicError> {
    let receiver = doc.snapshot().unwrap().root;
    doc.invoke(
        doc.revision(),
        MethodCall {
            method: MethodId(method),
            receiver,
            arguments: args,
        },
        &mut Host,
    )
    .unwrap()
    .result
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
    let out = doc
        .render_raw_operation(
            doc.revision(),
            &Object::default(),
            &RenderSettings::default(),
            &mut host,
        )
        .unwrap();
    let r = out.result.unwrap();
    json!({"ok":true,"svg":r.svg,"width":format!("{:016x}",r.width.to_bits()),"height":format!("{:016x}",r.height.to_bits()),"trace":host.trace})
}
#[test]
fn retained_block_warnings_preserve_partial_hierarchy_mutation_reentry_and_recovery() {
    for expected in fixture()["cases"].as_array().unwrap() {
        let mode = expected["mode"].as_str().unwrap();
        let mut doc = construct();
        let mut graph = doc.snapshot().unwrap();
        let statements = input(&mut graph, &statements());
        if mode == "noncallable" {
            let receiver = graph.root.clone();
            set(&mut graph, &receiver, "warn", Atom::Number(42.));
        }
        doc.apply(doc.revision(), graph).unwrap();
        let receiver = doc.snapshot().unwrap().root;
        let mut job = doc
            .prepare_detached_invocation(
                doc.revision(),
                MethodCall {
                    method: MethodId(10),
                    receiver,
                    arguments: vec![statements.clone()],
                },
            )
            .unwrap()
            .expect("warning continuation");
        let mut reply = None;
        let mut trace = vec![];
        let mut calls = 0;
        let mut gets = 0;
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
            let mut graph = doc.snapshot().unwrap();
            reply = Some(match op {
                PublicHostOperation::Get {
                    receiver,
                    key: "warn",
                } => {
                    assert!(same_ref(&receiver, &graph.root));
                    if mode.starts_with("getter-") {
                        gets += 1;
                        trace.push(json!(["get", gets, state(&graph, &statements)]));
                        if mode == "getter-throw" {
                            Err(PublicError::HostException(HostExceptionToken(77)))
                        } else {
                            if gets == 1 {
                                let a = at(&graph, &statements, "1");
                                set(&mut graph, &a, "widthInColumns", Atom::Number(9.));
                                doc.apply(doc.revision(), graph).unwrap();
                            }
                            Ok(PublicHostValue::Opaque(41))
                        }
                    } else if mode == "noncallable" {
                        Ok(PublicHostValue::Number(42.))
                    } else {
                        Ok(PublicHostValue::Opaque(41))
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
                    if matches!(function, Atom::Number(42.)) {
                        Err(PublicError::JavaScript {
                            name: JavaScriptErrorKind::TypeError,
                            message: JsString::from(not_callable_error),
                        })
                    } else {
                        assert!(matches!(function, Atom::HostReference(41)));
                        calls += 1;
                        trace.push(json!([
                            "warn",
                            same_ref(&receiver, &graph.root),
                            val(&graph, &arguments[0]),
                            state(&graph, &statements)
                        ]));
                        if mode == "throw" {
                            Err(PublicError::HostException(HostExceptionToken(77)))
                        } else {
                            if mode == "mutate" && calls == 1 {
                                let a = at(&graph, &statements, "1");
                                let kind = string(&mut graph, "column-setting");
                                set(&mut graph, &a, "type", kind);
                                set(&mut graph, &a, "columns", Atom::Number(4.));
                                doc.apply(doc.revision(), graph).unwrap();
                            } else if mode == "mutate" && calls == 2 {
                                let b = at(&graph, &statements, "2");
                                let label = string(&mut graph, "Changed");
                                set(&mut graph, &b, "label", label);
                                let name = string(&mut graph, "Pending");
                                doc.apply(doc.revision(), graph).unwrap();
                                invoke(&mut doc, 9, vec![name]).unwrap();
                            } else if mode == "nested" && calls == 1 {
                                let nested = input(
                                    &mut graph,
                                    &json!([{"id":"Nested","type":"square","widthInColumns":1}]),
                                );
                                doc.apply(doc.revision(), graph).unwrap();
                                invoke(&mut doc, 10, vec![nested]).unwrap();
                            }
                            Ok(PublicHostValue::Undefined)
                        }
                    }
                }
                _ => panic!("unexpected warning host operation"),
            });
        }
        let out = doc.finish_detached_invocation(&mut job).unwrap();
        let actual = match out.result {
            Ok(v) => json!({"ok":true,"undefined":matches!(v,Atom::Undefined)}),
            Err(PublicError::HostException(HostExceptionToken(77))) => {
                json!({"ok":false,"original":true})
            }
            Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message,
            }) => {
                json!({"ok":false,"original":false,"name":"TypeError","message":message.well_formed().unwrap()})
            }
            other => panic!("unexpected hierarchy outcome {mode}: {other:?}"),
        };
        assert_json(&actual, &expected["result"]);
        assert_json(&json!(trace), &expected["trace"]);
        assert_json(
            &state(&doc.snapshot().unwrap(), &statements),
            &expected["after"],
        );
        if expected["result"]["ok"] == true {
            assert_json(&raw(&mut doc), &expected["firstRaw"]);
        }
        let mut graph = doc.snapshot().unwrap();
        let receiver = graph.root.clone();
        set(&mut graph, &receiver, "warn", Atom::Undefined);
        doc.apply(doc.revision(), graph).unwrap();
        assert!(matches!(
            invoke(&mut doc, 10, vec![statements.clone()]).unwrap(),
            Atom::Undefined
        ));
        assert_json(
            &state(&doc.snapshot().unwrap(), &statements),
            &expected["recovered"],
        );
        assert_json(&raw(&mut doc), &expected["recoveryRaw"]);
    }
}
