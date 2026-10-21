//! Default AgentFlow raw drawing: inherited diagnostics, partial effects and repair.
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
        "../../../fixtures/public-agentflow-render-diagnostics.json"
    ))
    .unwrap()
}
#[derive(Default)]
struct Host {
    trace: Vec<Value>,
}
impl RenderResources for Host {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.trace
            .push(json!([r.text, format!("{:016x}", r.size.to_bits())]));
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
            Node::Map { entries, .. } => entries
                .iter()
                .map(|(k, v)| json!([val(g, k), val(g, v)]))
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

fn at(g: &ModelGraph, atom: &Atom, key: &str) -> Atom {
    let Atom::Ref(id) = atom else { panic!("ref") };
    g.nodes[*id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(Atom::Undefined, |p| p.value.clone())
}
fn state(g: &ModelGraph) -> Value {
    let diagnostics = field(g, "diagnostics");
    let Atom::Ref(id) = diagnostics else {
        panic!("diagnostics")
    };
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        panic!("array")
    };
    let Atom::Ref(mapping) = field(g, "firstMapping") else {
        panic!("map")
    };
    let Node::Map { entries, .. } = &g.nodes[mapping as usize] else {
        panic!("map")
    };
    let aliases: Vec<_> = (0..*length)
        .map(|i| {
            let d = &properties
                .iter()
                .find(|p| g.strings[p.key as usize].is(&i.to_string()))
                .unwrap()
                .value;
            let position = at(g, d, "position");
            if matches!(position, Atom::Undefined) {
                return true;
            }
            let node = at(g, d, "nodeId");
            let key = if matches!(node, Atom::Undefined | Atom::Null) {
                at(g, d, "edgeId")
            } else {
                node
            };
            let mapping = &entries
                .iter()
                .find(|(k, _)| val(g, k) == val(g, &key))
                .unwrap()
                .1;
            matches!((position, at(g, mapping, "position")), (Atom::Ref(a), Atom::Ref(b)) if a == b)
        })
        .collect();
    let Atom::Ref(reported) = field(g, "reported") else {
        panic!("reported")
    };
    let Node::Map {
        entries: reports, ..
    } = &g.nodes[reported as usize]
    else {
        panic!("reported map")
    };
    let reported_aliases: Vec<_> = (0..*length)
        .map(|i| {
            let d = &properties
                .iter()
                .find(|p| g.strings[p.key as usize].is(&i.to_string()))
                .unwrap()
                .value;
            let Atom::Ref(did) = d else {
                panic!("diagnostic")
            };
            reports.iter().any(|(_, list)| {
                let Atom::Ref(list) = list else {
                    panic!("reported list")
                };
                g.nodes[*list as usize]
                    .properties()
                    .iter()
                    .any(|p| matches!(p.value,Atom::Ref(id) if id==*did))
            })
        })
        .collect();
    json!({"diagnostics":val(g,&diagnostics),"mappings":val(g,&field(g,"mappings")),"reported":val(g,&field(g,"reported")),"mappingAlias":aliases,"reportedAlias":reported_aliases})
}
fn raw(doc: &mut PublicDocument) -> Value {
    let mut host = Host::default();
    let r = doc
        .render_raw_operation(
            doc.revision(),
            &Object::default(),
            &RenderSettings::default(),
            &mut host,
        )
        .unwrap();
    match r.result {
        Ok(r) => {
            json!({"ok":true,"svg":r.svg,"width":format!("{:016x}",r.width.to_bits()),"height":format!("{:016x}",r.height.to_bits()),"trace":host.trace})
        }
        Err(PublicError::JavaScript { name, message }) => {
            assert_eq!(name, JavaScriptErrorKind::TypeError);
            json!({"ok":false,"name":"TypeError","message":message.well_formed().unwrap(),"trace":host.trace})
        }
        Err(e) => panic!("unexpected error {e:?}"),
    }
}
fn prepare(case: &Value) -> PublicDocument {
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramType::Agentflow.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    let mut doc = factory.construct().unwrap();
    for a in case["actions"].as_array().unwrap() {
        let method = doc
            .model_schema()
            .methods
            .iter()
            .find(|m| m.name == a["name"].as_str().unwrap())
            .unwrap()
            .id
            .0;
        let c = call(&mut doc, method, a["args"].clone());
        doc.invoke(doc.revision(), c, &mut Host::default())
            .unwrap()
            .result
            .unwrap();
    }
    doc
}
fn edit(doc: &mut PublicDocument, mode: &str, repair: bool) {
    if mode == "writable-length" && repair {
        return;
    }
    let mut g = doc.snapshot().unwrap();
    let Atom::Ref(root) = g.root else {
        panic!("root")
    };
    match mode {
        "readonly" | "writable-length" => {
            let Atom::Ref(id) = field(&g, "diagnostics") else {
                panic!("array")
            };
            if repair {
                let mut n = g.nodes[id as usize].clone();
                let strings = &g.strings;
                n.properties_mut()
                    .retain(|p| !strings[p.key as usize].is("length"));
                let new = g.nodes.len() as u32;
                g.nodes.push(n);
                let strings = &g.strings;
                g.nodes[root as usize]
                    .properties_mut()
                    .iter_mut()
                    .find(|p| strings[p.key as usize].is("diagnostics"))
                    .unwrap()
                    .value = Atom::Ref(new);
            } else {
                let Atom::String(key) = string(&mut g, "length") else {
                    unreachable!()
                };
                g.nodes[id as usize].properties_mut().push(Property {
                    key,
                    flags: if mode == "readonly" { 0 } else { 2 },
                    value: Atom::Number(0.),
                });
            }
        }
        "shadowed" => {
            if repair {
                let strings = &g.strings;
                g.nodes[root as usize]
                    .properties_mut()
                    .retain(|p| !strings[p.key as usize].is("emitError"));
            } else {
                let Atom::String(key) = string(&mut g, "emitError") else {
                    unreachable!()
                };
                g.nodes[root as usize].properties_mut().push(Property {
                    key,
                    flags: 7,
                    value: Atom::Null,
                });
            }
        }
        "plain" | "copied-emitter" => {
            let mut properties = g.nodes[root as usize].properties().to_vec();
            if mode == "copied-emitter" {
                if repair {
                    properties.retain(|p| !g.strings[p.key as usize].is("emitError"));
                } else {
                    let Atom::String(key) = string(&mut g, "emitError") else {
                        unreachable!()
                    };
                    properties.push(Property {
                        key,
                        flags: 7,
                        value: Atom::BuiltinFunction(34),
                    });
                }
            }
            g.nodes[root as usize] = if repair {
                Node::Instance {
                    prototype: PrototypeId(1),
                    properties,
                }
            } else {
                Node::Object { properties }
            };
        }
        _ => return,
    }
    doc.apply(doc.revision(), g).unwrap();
}
#[test]
fn agentflow_default_raw_diagnostics_aliases_partial_failure_and_recovery() {
    for case in fixture()["cases"].as_array().unwrap() {
        let mode = case["mode"].as_str().unwrap();
        let mut doc = prepare(case);
        edit(&mut doc, mode, false);
        assert_json(&raw(&mut doc), &case["first"]);
        assert_json(&state(&doc.snapshot().unwrap()), &case["after"]);
        if !case["second"].is_null() {
            assert_json(&raw(&mut doc), &case["second"]);
        }
        assert_json(&state(&doc.snapshot().unwrap()), &case["afterSecond"]);
        edit(&mut doc, mode, true);
        assert_json(&raw(&mut doc), &case["recovered"]);
        assert_json(&state(&doc.snapshot().unwrap()), &case["afterRecovery"]);
    }
}
#[test]
fn agentflow_raw_foreign_log_is_explicit_and_keeps_diagnostic_partial_state() {
    let data = fixture();
    let case = &data["cases"][0];
    let mut doc = prepare(case);
    let mut g = doc.snapshot().unwrap();
    let Atom::Ref(options) = field(&g, "options") else {
        panic!("options")
    };
    let Atom::String(key) = string(&mut g, "log") else {
        unreachable!()
    };
    g.nodes[options as usize].properties_mut().push(Property {
        key,
        flags: 7,
        value: Atom::HostReference(901),
    });
    doc.apply(doc.revision(), g).unwrap();
    let result = doc
        .render_raw_operation(
            doc.revision(),
            &Object::default(),
            &RenderSettings::default(),
            &mut Host::default(),
        )
        .unwrap();
    let Err(PublicError::Pele(error)) = result.result else {
        panic!("explicit unsupported callback")
    };
    assert_eq!(
        error.message,
        "AgentFlow diagnostic log callback dispatch is not implemented"
    );
    assert_json(&state(&doc.snapshot().unwrap()), &case["after"]);
    // Default logging restored; prior diagnostic is deduplicated and drawing recovers.
    let mut g = doc.snapshot().unwrap();
    let strings = &g.strings;
    g.nodes[options as usize]
        .properties_mut()
        .retain(|p| !strings[p.key as usize].is("log"));
    doc.apply(doc.revision(), g).unwrap();
    assert_json(&raw(&mut doc), &case["first"]);
}
