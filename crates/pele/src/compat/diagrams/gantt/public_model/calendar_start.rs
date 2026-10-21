//! Owned native scheduler methods. Resources run with no document/store borrow.
use super::*;
use crate::compat::public::PublicResourceOperation;
enum Program {
    Date(Option<scheduler::CalendarRequest>),
    Invalid(scheduler::Invalid<DateValue>),
    Start(scheduler::Start<Atom>),
    End(scheduler::End<Atom, DateValue>),
    Task(scheduler::Task<Atom, DateValue>),
    Tasks(scheduler::Tasks<Atom, DateValue, Iter>),
    Schedule(scheduler::Schedule<Atom, DateValue, Iter>),
}
enum Await {
    Number,
    Text,
    Clock,
}
struct Invocation {
    receiver: Atom,
    method: u32,
    arguments: Option<Vec<Atom>>,
    program: Option<Program>,
    warning: Option<WarningInvocation>,
    awaiting: Option<Await>,
    direct_entry: Option<&'static str>,
    work: usize,
}
pub(super) fn invocation(call: MethodCall) -> DetachedPublicInvocation {
    let direct_entry = Some(match call.method.0 {
        19 => "getTasks",
        20 => "isInvalidDate",
        22 => "parse",
        23 => "today",
        25 => "compileTasks",
        26 => "compileTask",
        28 => "getStartDate",
        30 => "getEndDate",
        _ => unreachable!(),
    });
    DetachedPublicInvocation::new::<Gantt, GanttPublicState, _>(
        Invocation {
            receiver: call.receiver,
            method: call.method.0,
            arguments: Some(call.arguments),
            program: None,
            warning: None,
            awaiting: None,
            direct_entry,
            work: 0,
        },
        resume,
    )
}
fn resume(
    state: &mut GanttPublicState,
    invocation: &mut Invocation,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    invocation
        .step(&mut state.graph, reply)
        .unwrap_or_else(|error| PublicInvocationPhase::Complete(Err(error)))
}
impl Invocation {
    fn step(
        &mut self,
        graph: &mut ModelGraph,
        mut reply: Option<PublicHostValue>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        if let Some(warning) = &mut self.warning {
            let reply = reply
                .take()
                .map(|v| v.into_atom(|v| Context::new(graph).intern(v)))
                .transpose()?;
            match warning.step(graph, reply)? {
                PublicInvocationPhase::Complete(Ok(_)) => self.warning = None,
                phase => return Ok(phase),
            }
        }
        let calendar_reply = match self.awaiting.take() {
            Some(Await::Clock) => {
                let Some(PublicHostValue::Number(now)) = reply else {
                    return Err(invalid("Invalid Gantt clock response").into());
                };
                self.program = Some(Program::Schedule(scheduler::Schedule::new(now)));
                None
            }
            Some(Await::Number) => Some(scheduler::CalendarReply::Number(match reply {
                Some(PublicHostValue::Number(n)) => Some(n),
                Some(PublicHostValue::Undefined) => None,
                _ => return Err(invalid("Invalid Gantt calendar response").into()),
            })),
            Some(Await::Text) => Some(scheduler::CalendarReply::Text(match reply {
                Some(PublicHostValue::String(s)) => Some(
                    s.well_formed()
                        .ok_or_else(|| invalid("Invalid Gantt calendar text response"))?,
                ),
                Some(PublicHostValue::Undefined) => None,
                _ => return Err(invalid("Invalid Gantt calendar response").into()),
            })),
            None => None,
        };
        let mut arena = Arena {
            c: Context::new(graph),
            receiver: self.receiver.clone(),
            direct_entry: self.direct_entry,
        };
        arena.c.work = self.work;
        let result = (|| {
            if let Some(arguments) = self.arguments.take() {
                let a = arguments.first().cloned().unwrap_or(Atom::Undefined);
                let b = arguments.get(1).cloned().unwrap_or(Atom::Undefined);
                self.program = Some(match self.method {
                    20 => {
                        check_date_get_day(&mut arena.c, &a)?;
                        Program::Invalid(scheduler::Invalid::new(DateValue::Existing(a)))
                    }
                    22 => {
                        let text = arena.c.scalar(&a, "str.length")?;
                        let template = arena.c.scalar(&b, "template.length")?;
                        Program::Date(Some(scheduler::parse_request(&mut arena, text, template)?))
                    }
                    23 => Program::Date(Some(scheduler::today_request(&mut arena)?)),
                    28 => Program::Start(scheduler::Start::new(arena.c.scalar(&a, "raw.trim")?)),
                    30 => Program::End(scheduler::End::new(
                        DateValue::Existing(a),
                        arena.c.scalar(&b, "raw.trim")?,
                    )),
                    26 => Program::Task(scheduler::Task::new(a)),
                    25 => Program::Tasks(scheduler::Tasks::new()),
                    19 => {
                        let number = arena.c.numeric(&a)?;
                        if !number.is_finite() {
                            self.awaiting = Some(Await::Clock);
                            return Ok(PublicInvocationPhase::Resource(
                                PublicResourceOperation::new(|host| {
                                    host.direct_now_millis()
                                        .map(PublicHostValue::Number)
                                        .map_err(PublicError::from)
                                }),
                            ));
                        }
                        Program::Schedule(scheduler::Schedule::new(number))
                    }
                    _ => unreachable!(),
                });
            }
            let step = match self
                .program
                .as_mut()
                .expect("initialized scheduler program")
            {
                Program::Date(request) => {
                    if let Some(request) = request.take() {
                        scheduler::Step::Resource(request)
                    } else {
                        let date = calendar_reply.expect("native calendar date reply").number();
                        scheduler::Step::Complete(date.map_or(Ok(Atom::Undefined), |ms| {
                            arena.c.date_value(DateValue::Fresh(ms))
                        })?)
                    }
                }
                Program::Invalid(program) => match program.advance(&mut arena, calendar_reply)? {
                    scheduler::Step::Complete(value) => {
                        scheduler::Step::Complete(value.map_or(Atom::Undefined, Atom::Bool))
                    }
                    scheduler::Step::Resource(op) => scheduler::Step::Resource(op),
                    scheduler::Step::Warning(w) => scheduler::Step::Warning(w),
                },
                Program::Start(program) => match program.advance(&mut arena, calendar_reply)? {
                    scheduler::Step::Complete(date) => scheduler::Step::Complete(
                        date.map_or(Ok(Atom::Undefined), |d| arena.c.date_value(d))?,
                    ),
                    scheduler::Step::Resource(op) => scheduler::Step::Resource(op),
                    scheduler::Step::Warning(w) => scheduler::Step::Warning(w),
                },
                Program::End(program) => match program.advance(&mut arena, calendar_reply)? {
                    scheduler::Step::Complete(date) => scheduler::Step::Complete(
                        date.map_or(Ok(Atom::Undefined), |d| arena.c.date_value(d))?,
                    ),
                    scheduler::Step::Resource(op) => scheduler::Step::Resource(op),
                    scheduler::Step::Warning(w) => scheduler::Step::Warning(w),
                },
                Program::Task(program) => match program.advance(&mut arena, calendar_reply)? {
                    scheduler::Step::Complete(()) => scheduler::Step::Complete(Atom::Undefined),
                    scheduler::Step::Resource(op) => scheduler::Step::Resource(op),
                    scheduler::Step::Warning(w) => scheduler::Step::Warning(w),
                },
                Program::Tasks(program) => match program.advance(&mut arena, calendar_reply)? {
                    scheduler::Step::Complete(v) => scheduler::Step::Complete(Atom::Bool(v)),
                    scheduler::Step::Resource(op) => scheduler::Step::Resource(op),
                    scheduler::Step::Warning(w) => scheduler::Step::Warning(w),
                },
                Program::Schedule(program) => match program.advance(&mut arena, calendar_reply)? {
                    scheduler::Step::Complete(()) => scheduler::Step::Complete(arena.get("tasks")?),
                    scheduler::Step::Resource(op) => scheduler::Step::Resource(op),
                    scheduler::Step::Warning(w) => scheduler::Step::Warning(w),
                },
            };
            Ok(match step {
                scheduler::Step::Complete(value) => PublicInvocationPhase::Complete(Ok(value)),
                scheduler::Step::Warning(warning) => {
                    let expression = match &warning {
                        scheduler::Warning::References { .. } => {
                            PublicHostErrorExpression::WarnReferences
                        }
                        scheduler::Warning::Duration { .. } => {
                            PublicHostErrorExpression::WarnDuration
                        }
                    };
                    if let Some(failure) = arena.warning_expression_guard(expression)? {
                        return Ok(failure);
                    }
                    let mut warning = WarningInvocation {
                        phase: WarningPhase::Start {
                            receiver: self.receiver.clone(),
                            producer: WarningProducer::Scheduler(warning),
                        },
                        references: None,
                    };
                    let phase = warning.step(arena.c.g, None)?;
                    self.warning = Some(warning);
                    phase
                }
                scheduler::Step::Resource(operation) => {
                    self.awaiting = Some(if operation.returns_text() {
                        Await::Text
                    } else {
                        Await::Number
                    });
                    PublicInvocationPhase::Resource(PublicResourceOperation::new(
                        move |resources| {
                            operation
                                .run(&mut Calendar::public(resources))
                                .map(|reply| match reply {
                                    scheduler::CalendarReply::Number(value) => value.map_or(
                                        PublicHostValue::Undefined,
                                        PublicHostValue::Number,
                                    ),
                                    scheduler::CalendarReply::Text(value) => value
                                        .map_or(PublicHostValue::Undefined, |s| {
                                            PublicHostValue::String(JsString::from(s.as_str()))
                                        }),
                                })
                                .map_err(PublicError::from)
                        },
                    ))
                }
            })
        })();
        self.work = arena.c.work;
        self.direct_entry = arena.direct_entry;
        result
    }
}
