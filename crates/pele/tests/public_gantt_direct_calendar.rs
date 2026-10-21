//! Public parse/today Calendar callbacks, retained effects and fresh Date identities.
use pele::compat::{public::*, *};
use serde_json::{json, Value};
struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(1.)
    }
}
fn new_document() -> PublicDocument {
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramType::Gantt.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    drop(engine);
    factory
        .construct_with(ConstructorArgument::Undefined)
        .unwrap()
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
fn set(g: &mut ModelGraph, name: &str, value: Atom) {
    let Atom::Ref(root) = g.root else {
        panic!("root")
    };
    let index = g.nodes[root as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(name))
        .unwrap();
    g.nodes[root as usize].properties_mut()[index].value = value;
}
fn prepare(doc: &mut PublicDocument, args: &[&str], now: f64, work: f64) -> Vec<Atom> {
    let mut g = doc.snapshot().unwrap();
    set(&mut g, "now", Atom::Number(now));
    set(&mut g, "work", Atom::Number(work));
    let args = args
        .iter()
        .map(|s| {
            let id = g.strings.len() as u32;
            g.strings.push(JsString::from(*s));
            Atom::String(id)
        })
        .collect();
    doc.apply(doc.revision(), g).unwrap();
    args
}
fn owned(
    doc: &mut PublicDocument,
    method: u32,
    args: Vec<Atom>,
    mut resource: impl FnMut(
        &mut PublicDocument,
        PublicResourceOperation,
    ) -> Result<PublicHostValue, PublicError>,
) -> Result<Atom, PublicError> {
    let call = MethodCall {
        receiver: doc.snapshot().unwrap().root,
        method: MethodId(method),
        arguments: args,
    };
    let mut job = doc
        .prepare_detached_invocation(doc.revision(), call)
        .unwrap()
        .expect("owned direct Calendar method");
    let mut reply = None;
    loop {
        match doc
            .resume_detached_invocation(&mut job, reply.take())
            .unwrap()
        {
            DetachedInvocationStep::Complete => break,
            DetachedInvocationStep::Resource(op) => reply = Some(resource(doc, op)),
            DetachedInvocationStep::Host(op) => panic!("Unexpected host operation: {op:?}"),
        }
    }
    doc.finish_detached_invocation(&mut job).unwrap().result
}
fn result(g: &ModelGraph, a: &Atom) -> Value {
    match a {
        Atom::Undefined => json!({"kind":"undefined"}),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Date { milliseconds, .. } => {
                json!({"kind":"date","bits":format!("{:016x}",milliseconds.to_bits())})
            }
            _ => panic!("Date"),
        },
        _ => panic!("Date or undefined"),
    }
}
fn error(e: PublicError) -> Value {
    match e {
        PublicError::Pele(e) => {
            json!({"name":e.name,"message":e.message,"code":e.code,"type":e.diagram_type,"line":e.line,"column":e.column,"snippet":e.snippet})
        }
        e => panic!("unexpected error {e:?}"),
    }
}
#[test]
fn direct_calendar_methods_match_pinned_dates_work_limits_and_synchronous_api() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/public-gantt-direct-calendar.json"
    ))
    .unwrap();
    assert_eq!(
        fixture["oracleSHA256"],
        "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d"
    );
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 8);
    for row in fixture["cases"].as_array().unwrap() {
        for detached in [false, true] {
            let mut doc = new_document();
            let args: Vec<_> = row["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect();
            let args = prepare(
                &mut doc,
                &args,
                row["now"].as_f64().unwrap_or(f64::NAN),
                if row["id"] == "parse-limit" {
                    400000000.
                } else {
                    0.
                },
            );
            let method = row["method"].as_u64().unwrap() as u32;
            let mut calls = 0;
            let returned = if detached {
                owned(&mut doc, method, args, |_, op| {
                    calls += 1;
                    op.run(&mut Host)
                })
            } else {
                doc.invoke(
                    doc.revision(),
                    MethodCall {
                        receiver: doc.snapshot().unwrap().root,
                        method: MethodId(method),
                        arguments: args,
                    },
                    &mut Host,
                )
                .unwrap()
                .result
            };
            let graph = doc.snapshot().unwrap();
            let (actual, err) = match returned {
                Ok(a) => (result(&graph, &a), Value::Null),
                Err(e) => (json!({"kind":"undefined"}), error(e)),
            };
            assert_eq!(actual, row["result"], "{} owned={detached}", row["id"]);
            assert_eq!(err, row["error"], "{} owned={detached}", row["id"]);
            let Atom::Number(work) = field(&graph, "work") else {
                panic!("work")
            };
            assert_eq!(*work, row["work"].as_f64().unwrap());
            if detached {
                assert_eq!(calls, if row["id"] == "parse-limit" { 0 } else { 1 });
            }
        }
    }
}
fn original() -> PeleError {
    PeleError::new("original Calendar callback", ErrorCode::Semantic)
}
struct Reentrant<'a> {
    doc: &'a mut PublicDocument,
    at_instant: bool,
    fail: bool,
    entered: bool,
    nested: Option<u32>,
    trace: Vec<&'static str>,
}
impl Reentrant<'_> {
    fn enter(&mut self) -> Result<(), PeleError> {
        if self.entered {
            return Ok(());
        }
        self.entered = true;
        let mut g = self.doc.snapshot().unwrap();
        let id = g.strings.len() as u32;
        g.strings.push(JsString::from("Calendar callback"));
        set(&mut g, "title", Atom::String(id));
        self.doc.apply(self.doc.revision(), g).unwrap();
        let args = prepare(
            self.doc,
            &["Nested"],
            1704813630000.,
            if self.at_instant { 13. } else { 17. },
        );
        let out = self
            .doc
            .invoke(
                self.doc.revision(),
                MethodCall {
                    receiver: self.doc.snapshot().unwrap().root,
                    method: MethodId(14),
                    arguments: args,
                },
                &mut Host,
            )
            .unwrap();
        assert!(out.result.is_ok());
        let nested = owned(self.doc, 23, vec![], |_, op| op.run(&mut Host)).unwrap();
        let Atom::Ref(id) = nested else {
            panic!("nested Date")
        };
        self.nested = Some(id);
        if self.fail {
            Err(original())
        } else {
            Ok(())
        }
    }
}
impl RenderResources for Reentrant<'_> {
    fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(1.)
    }
    fn calendar_fields(&mut self, n: f64) -> Result<Option<CalendarFields>, PeleError> {
        self.trace.push("fields");
        if !self.at_instant {
            self.enter()?
        }
        Host.calendar_fields(n)
    }
    fn calendar_instant(&mut self, c: CivilDateTime) -> Result<Option<f64>, PeleError> {
        self.trace.push("instant");
        if self.at_instant {
            self.enter()?
        }
        Host.calendar_instant(c)
    }
}
#[test]
fn direct_calendar_callbacks_reenter_preserve_failure_writes_and_allocate_in_latest_graph() {
    for method in [22, 23] {
        for at_instant in [false, true] {
            let mut doc = new_document();
            let args = prepare(
                &mut doc,
                if method == 22 {
                    &["2024-01-09", "YYYY-MM-DD"]
                } else {
                    &[]
                },
                1704813630000.,
                0.,
            );
            let mut dates = vec![];
            for fail in [true, false] {
                let before = doc.revision();
                let mut trace = vec![];
                let mut entered = false;
                let returned = owned(&mut doc, method, args.clone(), |doc, op| {
                    let mut resources = Reentrant {
                        doc,
                        at_instant,
                        fail,
                        entered: false,
                        nested: None,
                        trace: vec![],
                    };
                    let reply = op.run(&mut resources);
                    entered = resources.entered;
                    trace = resources.trace;
                    dates.push(resources.nested.unwrap());
                    reply
                });
                assert!(entered);
                assert!(trace.contains(&"fields"));
                if at_instant {
                    assert!(trace.contains(&"instant"));
                }
                assert!(doc.revision() > before);
                let g = doc.snapshot().unwrap();
                let Atom::String(title) = field(&g, "title") else {
                    panic!("title")
                };
                assert!(g.strings[*title as usize].is("Calendar callback"));
                let Atom::Number(work) = field(&g, "work") else {
                    panic!("work")
                };
                assert_eq!(
                    *work,
                    if at_instant { 13. } else { 17. },
                    "outer resume must retain callback edits"
                );
                for id in &dates {
                    assert!(matches!(g.nodes[*id as usize], Node::Date { .. }));
                }
                if fail {
                    assert_eq!(error(returned.unwrap_err()), error(original().into()));
                } else {
                    let a = returned.unwrap();
                    assert_eq!(
                        result(&g, &a),
                        json!({"kind":"date","bits":"4278ceb84d400000"})
                    );
                    let Atom::Ref(id) = a else { unreachable!() };
                    assert!(
                        !dates.contains(&id),
                        "fresh outer Date cannot reuse nested identities"
                    );
                }
            }
        }
    }
}
