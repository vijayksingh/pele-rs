//! Public Gantt start-date scheduling, warning callbacks and owned calendar requests.
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
                    Node::Date { .. } => "date",
                    _ => panic!("fixture node kind"),
                };
                let mut out = json!({"kind":kind,"properties":[]});
                if let Node::Date { milliseconds, .. } = n {
                    out["milliseconds"] = json!(format!("{:016x}", milliseconds.to_bits()));
                }
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
fn own(g: &ModelGraph, object: u32, name: &str) -> Atom {
    g.nodes[object as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(name))
        .map(|p| p.value.clone())
        .unwrap_or(Atom::Undefined)
}
fn set(g: &mut ModelGraph, object: u32, name: &str, value: Atom) {
    if let Some(p) = g.nodes[object as usize]
        .properties_mut()
        .iter_mut()
        .find(|p| g.strings[p.key as usize].is(name))
    {
        p.value = value;
    } else {
        let key = g.strings.len() as u32;
        g.strings.push(JsString::from(name));
        g.nodes[object as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        });
    }
}
fn date(g: &mut ModelGraph, ms: f64) -> Atom {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Date {
        milliseconds: ms,
        properties: vec![],
    });
    Atom::Ref(id)
}
fn task_ids(doc: &PublicDocument) -> [u32; 2] {
    let g = doc.snapshot().unwrap();
    let Atom::Ref(tasks) = field(&g, "tasks") else {
        panic!("tasks")
    };
    [0, 1].map(|i| {
        let Atom::Ref(id) = own(&g, *tasks, &i.to_string()) else {
            panic!("task")
        };
        id
    })
}
fn change_date(doc: &mut PublicDocument, ms: f64) {
    let [a, _] = task_ids(doc);
    let mut g = doc.snapshot().unwrap();
    let value = date(&mut g, ms);
    set(&mut g, a, "endTime", value);
    doc.apply(doc.revision(), g).unwrap();
}
fn error(e: PublicError) -> Value {
    match e {
        PublicError::HostException(HostExceptionToken(77)) => json!("original"),
        PublicError::JavaScript { name, message } => {
            assert_eq!(name, JavaScriptErrorKind::TypeError);
            json!({"name":"TypeError","message":message.well_formed().unwrap()})
        }
        PublicError::Pele(e) => {
            json!({"name":e.name,"message":e.message,"code":e.code,"type":e.diagram_type,"line":e.line,"column":e.column,"snippet":e.snippet})
        }
        PublicError::PeleUtf16(e) => error(PublicError::Pele(e.well_formed().unwrap())),
        e => panic!("unexpected error {e:?}"),
    }
}
fn run(doc: &mut PublicDocument, id: &str, raw: &str, recovery: bool) -> Value {
    let argument = text(doc, JsString::from(raw));
    run_call(doc, id, 28, vec![argument], recovery)
}
fn run_call(
    doc: &mut PublicDocument,
    id: &str,
    method: u32,
    arguments: Vec<Atom>,
    recovery: bool,
) -> Value {
    let receiver = doc.snapshot().unwrap().root;
    let original_tasks = task_ids(doc);
    let mut job = doc
        .prepare_detached_invocation(
            doc.revision(),
            MethodCall {
                receiver,
                method: MethodId(method),
                arguments,
            },
        )
        .unwrap()
        .expect("owned start-date helper");
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
                        if id == "getter-edit" && reads == 1 {
                            change_date(doc, 1709251200000.);
                        }
                        if (id == "first-get-throw" && reads == 1)
                            || (id == "second-get-throw" && reads == 2)
                        {
                            Err(original())
                        } else if id.ends_with("warn-false") {
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
                let Atom::String(id) = value else {
                    panic!("native keyword")
                };
                let g = doc.snapshot().unwrap();
                assert!(g.strings[id as usize].is("after"));
                Ok(PublicHostValue::String(g.strings[id as usize].clone()))
            }
            PublicHostOperation::Call {
                function,
                receiver,
                arguments,
                result,
                not_callable_error,
            } => {
                assert_eq!(result, PublicHostResult::Discard);
                assert!(matches!(receiver, Atom::HostReference(10)));
                assert_eq!(not_callable_error, "this.options.warn is not a function");
                if matches!(function, Atom::Number(0.)) {
                    Err(PublicError::JavaScript {
                        name: JavaScriptErrorKind::TypeError,
                        message: JsString::from(not_callable_error),
                    })
                } else {
                    assert!(matches!(function, Atom::HostReference(20)));
                    let g = doc.snapshot().unwrap();
                    let Atom::String(message) = arguments[0] else {
                        panic!("message")
                    };
                    trace.push(json!(["call", true, g.strings[message as usize].0]));
                    if id == "warn-reentry" {
                        add_section(doc, "Nested");
                        change_date(doc, 1706745600000.);
                        let mut g = doc.snapshot().unwrap();
                        let Atom::Ref(map) = field(&g, "byId") else {
                            panic!("map")
                        };
                        let map = *map;
                        let Node::Map { entries, .. } = &mut g.nodes[map as usize] else {
                            panic!("map")
                        };
                        entries.clear();
                        doc.apply(doc.revision(), g).unwrap();
                        add_task(doc, "Replacement", ":a,2024-01-01,1d");
                    }
                    if id.ends_with("warn-reentry") && id != "warn-reentry" {
                        add_section(doc, "Nested");
                        let [a, _] = task_ids(doc);
                        let mut g = doc.snapshot().unwrap();
                        let Atom::Ref(raw) = own(&g, a, "raw") else {
                            panic!("raw")
                        };
                        let text = g.strings.len() as u32;
                        g.strings.push(JsString::from("2d"));
                        set(&mut g, raw, "end", Atom::String(text));
                        let Atom::Ref(tasks) = field(&g, "tasks") else {
                            panic!("tasks")
                        };
                        let Node::Array { length, .. } = g.nodes[*tasks as usize] else {
                            panic!("tasks")
                        };
                        doc.apply(doc.revision(), g).unwrap();
                        if length == 2 {
                            add_task(doc, "Appended", ":c,2024-01-05,1d");
                        }
                    }
                    if id.ends_with("warn-throw") {
                        Err(original())
                    } else {
                        Ok(PublicHostValue::Undefined)
                    }
                }
            }
            _ => panic!("unexpected start operation"),
        });
    }
    let outcome = doc.finish_detached_invocation(&mut job).unwrap();
    let (result, error) = match outcome.result {
        Ok(value) => (value, Value::Null),
        Err(e) => (Atom::Undefined, error(e)),
    };
    let g = doc.snapshot().unwrap();
    let aliases = original_tasks
        .map(|id| matches!((&result,own(&g,id,"endTime")),(Atom::Ref(a),Atom::Ref(b))if *a==b));
    if method == 28 {
        json!({"graph":graph(&g,&result),"trace":trace,"error":error,"aliases":aliases})
    } else {
        json!({"graph":graph(&g,&result),"trace":trace,"error":error,"returnedTasks":matches!((&result,field(&g,"tasks")),(Atom::Ref(a),Atom::Ref(b))if a==b)})
    }
}
#[test]
fn gantt_start_date_owns_warning_and_calendar_phases_without_losing_aliases_or_partial_effects() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/public-gantt-calendar-start.json"
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
        let raw = row["raw"].as_str().unwrap();
        let mut doc = factory
            .construct_with(ConstructorArgument::Opaque(10))
            .unwrap();
        add_task(&mut doc, "Alpha", ":a,2024-01-01,1d");
        add_task(&mut doc, "Beta", ":b,2024-01-02,1d");
        let tasks = task_ids(&doc);
        let mut g = doc.snapshot().unwrap();
        let Atom::Ref(root) = g.root else {
            panic!("root")
        };
        let format = g.strings.len() as u32;
        g.strings.push(JsString::from(if id == "timestamp" {
            "X"
        } else {
            "YYYY-MM-DD"
        }));
        set(&mut g, root, "dateFormat", Atom::String(format));
        set(&mut g, root, "now", Atom::Number(1704813630000.));
        for (i, task) in tasks.iter().enumerate() {
            let value = if id == "undefined-end" && i == 0 {
                Atom::Undefined
            } else {
                date(&mut g, 1704153600000. + i as f64 * 86400000.)
            };
            set(&mut g, *task, "endTime", value);
        }
        if matches!(id, "shadowed-known" | "shadowed-warn") {
            set(&mut g, root, "warn", Atom::Null);
        }
        doc.apply(doc.revision(), g).unwrap();
        assert_eq!(run(&mut doc, id, raw, false), row["first"], "{id} first");
        if matches!(id, "shadowed-known" | "shadowed-warn") {
            let mut g = doc.snapshot().unwrap();
            let strings = &g.strings;
            g.nodes[root as usize]
                .properties_mut()
                .retain(|p| !strings[p.key as usize].is("warn"));
            doc.apply(doc.revision(), g).unwrap();
        }
        assert_eq!(
            run(&mut doc, id, raw, true),
            row["recovery"],
            "{id} recovery"
        );
    }
}

#[test]
fn gantt_owned_scheduler_retains_warning_partial_writes_live_task_iteration_and_exclusion_dates() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/public-gantt-owned-scheduler.json"
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
        let mode = row["mode"].as_str().unwrap();
        let method = row["method"].as_u64().unwrap() as u32;
        let mut doc = factory
            .construct_with(ConstructorArgument::Opaque(10))
            .unwrap();
        add_task(
            &mut doc,
            "Alpha",
            if mode == "excludes" {
                ":a,2024-01-01,3d"
            } else {
                ":a,2024-01-01,bad-duration"
            },
        );
        add_task(&mut doc, "Beta", ":b,after a,1d");
        let [first, _] = task_ids(&doc);
        let mut g = doc.snapshot().unwrap();
        let Atom::Ref(root) = g.root else {
            panic!("root")
        };
        let format = g.strings.len() as u32;
        g.strings.push(JsString::from("YYYY-MM-DD"));
        set(&mut g, root, "dateFormat", Atom::String(format));
        set(&mut g, root, "now", Atom::Number(1704813630000.));
        if mode == "excludes" {
            let text = g.strings.len() as u32;
            g.strings.push(JsString::from("2024-01-02"));
            let key = g.strings.len() as u32;
            g.strings.push(JsString::from("0"));
            let excludes = g.nodes.len() as u32;
            g.nodes.push(Node::Array {
                length: 1,
                properties: vec![Property {
                    key,
                    flags: 7,
                    value: Atom::String(text),
                }],
            });
            set(&mut g, root, "excludes", Atom::Ref(excludes));
            let set_id = g.nodes.len() as u32;
            g.nodes.push(Node::Set {
                values: vec![Atom::String(text)],
                properties: vec![],
            });
            set(&mut g, root, "excludeSet", Atom::Ref(set_id));
            let mut properties = vec![];
            for i in 0..7 {
                let key = g.strings.len() as u32;
                g.strings.push(JsString::from(i.to_string().as_str()));
                properties.push(Property {
                    key,
                    flags: 7,
                    value: Atom::Bool(false),
                });
            }
            let days = g.nodes.len() as u32;
            g.nodes.push(Node::Array {
                length: 7,
                properties,
            });
            set(&mut g, root, "excludedDays", Atom::Ref(days));
            set(&mut g, root, "datedTokens", Atom::Bool(true));
        }
        doc.apply(doc.revision(), g).unwrap();
        for recovery in [false, true] {
            let args = match method {
                19 => vec![Atom::Number(1704813630000.)],
                25 => vec![],
                26 => vec![Atom::Ref(first)],
                30 => {
                    let raw = text(
                        &mut doc,
                        JsString::from(if mode == "excludes" {
                            "3d"
                        } else {
                            "bad-duration"
                        }),
                    );
                    let mut g = doc.snapshot().unwrap();
                    let value = date(&mut g, 1704067200000.);
                    doc.apply(doc.revision(), g).unwrap();
                    vec![value, raw]
                }
                _ => unreachable!(),
            };
            assert_eq!(
                run_call(&mut doc, id, method, args, recovery),
                row[if recovery { "recovery" } else { "first" }],
                "{id} recovery={recovery}"
            );
        }
    }
}
