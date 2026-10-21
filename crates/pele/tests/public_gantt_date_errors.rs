//! Public direct and owned Date validation preserves errors before calendar callbacks.
use pele::compat::{public::*, *};
use serde_json::{json, Value};
#[derive(Default)]
struct Host {
    calendar_calls: usize,
}
impl RenderResources for Host {
    fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(0.)
    }
    fn calendar_fields(&mut self, ms: f64) -> Result<Option<CalendarFields>, PeleError> {
        self.calendar_calls += 1;
        assert_eq!(ms, 0.);
        Ok(Some(CalendarFields {
            civil: CivilDateTime {
                year: 1970,
                month: 1,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 0,
            },
            weekday: 4,
            utc_offset_minutes: 0,
        }))
    }
}
fn normalize(result: Result<Atom, PublicError>) -> Value {
    match result {
        Ok(Atom::Undefined) => json!({"result":null}),
        Err(PublicError::JavaScript { name, message }) => {
            assert_eq!(name, JavaScriptErrorKind::TypeError);
            json!({"error":{"name":"TypeError","message":message.well_formed().unwrap()}})
        }
        other => panic!("unexpected public result {other:?}"),
    }
}
#[test]
fn gantt_direct_and_owned_date_validation_matches_pinned_errors_before_calendar_callbacks() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/public-gantt-date-errors.json"
    ))
    .unwrap();
    assert_eq!(
        fixture["oracleSHA256"],
        "d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d"
    );
    for case in fixture["rows"].as_array().unwrap() {
        for owned in [false, true] {
            let kind = case["kind"].as_str().unwrap();
            let mut doc = PublicEngine::with_builtins()
                .parse(
                    "gantt\n dateFormat YYYY-MM-DD\n Alpha :a,2024-01-01,2d",
                    &Default::default(),
                )
                .unwrap();
            let mut graph = doc.snapshot().unwrap();
            let receiver = graph.root.clone();
            let value = match kind {
                "undefined" => Atom::Undefined,
                "null" => Atom::Null,
                "number" => Atom::Number(0.),
                "object" => {
                    let id = graph.nodes.len() as u32;
                    graph.nodes.push(Node::Object { properties: vec![] });
                    Atom::Ref(id)
                }
                "date" | "date-shadowed" => {
                    let id = graph.nodes.len() as u32;
                    let properties = if kind == "date-shadowed" {
                        let key = graph.strings.len() as u32;
                        graph.strings.push("getDay".into());
                        vec![Property {
                            key,
                            value: Atom::Number(0.),
                            flags: 7,
                        }]
                    } else {
                        vec![]
                    };
                    graph.nodes.push(Node::Date {
                        milliseconds: 0.,
                        properties,
                    });
                    Atom::Ref(id)
                }
                _ => panic!("fixture kind"),
            };
            doc.apply(doc.revision(), graph).unwrap();
            let call = MethodCall {
                method: MethodId(20),
                receiver,
                arguments: vec![value],
            };
            let mut host = Host::default();
            let result = if owned {
                let mut job = doc
                    .prepare_detached_invocation(doc.revision(), call)
                    .unwrap()
                    .unwrap();
                let mut reply = None;
                loop {
                    match doc
                        .resume_detached_invocation(&mut job, reply.take())
                        .unwrap()
                    {
                        DetachedInvocationStep::Complete => break,
                        DetachedInvocationStep::Resource(op) => reply = Some(op.run(&mut host)),
                        DetachedInvocationStep::Host(_) => {
                            panic!("ordinary Date validation never calls a foreign method")
                        }
                    }
                }
                doc.finish_detached_invocation(&mut job).unwrap().result
            } else {
                doc.invoke(doc.revision(), call, &mut host).unwrap().result
            };
            let mut expected = case.as_object().unwrap().clone();
            expected.remove("kind");
            assert_eq!(normalize(result), Value::Object(expected), "{kind}/{owned}");
            assert_eq!(
                host.calendar_calls,
                usize::from(kind == "date"),
                "{kind}/{owned}: error must precede calendar callbacks"
            );
            assert!(!doc.has_active_detached_render());
        }
    }
}
