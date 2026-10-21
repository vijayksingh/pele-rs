//! Native option-dependent AgentFlow helper continuations. Only host operations
//! yield; mutation bodies are shared with ordinary native invocation.
use super::*;
pub(super) const FIELDS: &[LiveConstructorField] = &[LiveConstructorField {
    key: "options",
    methods: &[
        MethodId(4),
        MethodId(5),
        MethodId(17),
        MethodId(32),
        MethodId(33),
        MethodId(34),
    ],
}];
pub(super) struct LinkPending {
    pub root: u32,
    pub edge: u32,
    pub pair: Atom,
    pub count: f64,
    pub id: Atom,
}
pub(super) struct SubgraphPending {
    pub root: u32,
    pub id: Option<JsString>,
    pub nodes: Vec<Atom>,
    pub dir: Atom,
    pub title: Atom,
    pub title_text: Atom,
}
pub(super) struct DiagnosticPending {
    pub id: Atom,
    pub severity: Atom,
    pub message: Atom,
}
struct Links {
    root: u32,
    starts: Vec<Atom>,
    ends: Vec<Atom>,
    info: Atom,
    id: Atom,
    start: usize,
    end: usize,
}
impl Links {
    fn new(a: &mut Arena<'_>, call: &MethodCall, root: u32) -> Result<Self, PublicError> {
        a.guard(root, "addSingleLink", 4)?;
        let starts = dense(a.g, &arg(call, 0))?;
        let ends = dense(a.g, &arg(call, 1))?;
        if starts
            .len()
            .checked_mul(ends.len())
            .is_none_or(|n| n > MAX_WORK)
        {
            return Err(limit().into());
        }
        let info = arg(call, 2);
        let id = if let Atom::Ref(i) = info {
            if let Atom::String(s) = a.field(i, "id") {
                let mut units = a.g.strings[s as usize].0.clone();
                if let Some(at) = units.iter().position(|u| *u == 64) {
                    units.remove(at);
                }
                a.string(JsString(units))?
            } else {
                Atom::Undefined
            }
        } else {
            Atom::Undefined
        };
        Ok(Self {
            root,
            starts,
            ends,
            info,
            id,
            start: 0,
            end: 0,
        })
    }
    fn next(&mut self, g: &ModelGraph) -> Option<(Atom, Atom, Atom)> {
        if self.start >= self.starts.len() || self.ends.is_empty() {
            return None;
        }
        let start = self.starts[self.start].clone();
        let end = self.ends[self.end].clone();
        let id = if self.starts.last().is_some_and(|v| same(g, v, &start))
            && self.ends.first().is_some_and(|v| same(g, v, &end))
        {
            self.id.clone()
        } else {
            Atom::Undefined
        };
        self.end += 1;
        if self.end == self.ends.len() {
            self.end = 0;
            self.start += 1;
        }
        Some((start, end, id))
    }
}
enum Stage {
    Start(MethodCall),
    MaxOptions(LinkPending),
    MaxGet(LinkPending),
    MaxCompare(LinkPending, Atom),
    MaxError(u32),
    SubOptions(SubgraphPending),
    SubInherit(SubgraphPending),
    SubTruthy(SubgraphPending),
    SubDirectionOptions(SubgraphPending),
    SubDirection(SubgraphPending),
    LogOptions(DiagnosticPending),
    LogGet(DiagnosticPending, Atom),
    LogId(DiagnosticPending, Atom, Atom),
    LogMessage(DiagnosticPending, Atom, Atom, JsString),
    LogDone,
    Done,
}
struct Invocation {
    stage: Stage,
    links: Option<Links>,
    text_work: usize,
}
fn get(receiver: Atom, key: &'static str) -> PublicInvocationPhase {
    PublicInvocationPhase::Host(PublicHostOperation::Get { receiver, key })
}
fn complete() -> PublicInvocationPhase {
    PublicInvocationPhase::Complete(Ok(Atom::Undefined))
}
impl Invocation {
    fn request_max(&mut self, p: LinkPending) -> PublicInvocationPhase {
        let root = p.root;
        self.stage = Stage::MaxOptions(p);
        get(Atom::Ref(root), "options")
    }
    fn next_link(&mut self, a: &mut Arena<'_>) -> Result<PublicInvocationPhase, PublicError> {
        let Some(batch) = self.links.as_mut() else {
            return Ok(complete());
        };
        let Some((start, end, id)) = batch.next(a.g) else {
            return Ok(complete());
        };
        let pending = a.single_link_prepare(batch.root, start, end, batch.info.clone(), id)?;
        Ok(self.request_max(pending))
    }
    fn step(
        &mut self,
        a: &mut Arena<'_>,
        reply: Option<Atom>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        Ok(match std::mem::replace(&mut self.stage, Stage::Done) {
            Stage::Start(call) => {
                let root = a.id(&call.receiver)?;
                match call.method.0 {
                    4 => {
                        let p = a.single_link_prepare(
                            root,
                            arg(&call, 0),
                            arg(&call, 1),
                            arg(&call, 2),
                            arg(&call, 3),
                        )?;
                        self.request_max(p)
                    }
                    5 => {
                        self.links = Some(Links::new(a, &call, root)?);
                        return self.next_link(a);
                    }
                    17 => {
                        let p =
                            a.subgraph_prepare(root, arg(&call, 0), arg(&call, 1), arg(&call, 2))?;
                        if matches!(p.dir, Atom::Undefined | Atom::Null) {
                            self.stage = Stage::SubOptions(p);
                            get(Atom::Ref(root), "options")
                        } else {
                            PublicInvocationPhase::Complete(a.subgraph_finish(p))
                        }
                    }
                    32..=34 => {
                        let (id, severity, message, ctx) = if call.method.0 == 32 {
                            (arg(&call, 0), arg(&call, 1), arg(&call, 2), arg(&call, 3))
                        } else {
                            a.guard(root, "emit", 32)?;
                            let severity = a.string(if call.method.0 == 33 {
                                "warning"
                            } else {
                                "error"
                            })?;
                            (arg(&call, 0), severity, arg(&call, 1), arg(&call, 2))
                        };
                        let Some(p) = a.emit_prepare(root, id, severity, message, ctx)? else {
                            return Ok(complete());
                        };
                        self.stage = Stage::LogOptions(p);
                        get(Atom::Ref(root), "options")
                    }
                    _ => return Err(error("AgentFlow detached method is unavailable").into()),
                }
            }
            Stage::MaxOptions(p) => {
                self.stage = Stage::MaxGet(p);
                get(reply.expect("options reply"), "maxEdges")
            }
            Stage::MaxGet(p) => {
                let max = reply.expect("maxEdges reply");
                if matches!(max, Atom::Undefined) {
                    a.single_link_commit(p)?;
                    return self.next_link(a);
                }
                let edges = a.field_id(p.root, "edges")?;
                let Node::Array { length, .. } = a.g.nodes[edges as usize] else {
                    return Err(ty("AgentFlow edges must be an Array"));
                };
                self.stage = Stage::MaxCompare(p, max.clone());
                PublicInvocationPhase::Host(PublicHostOperation::GreaterEqual {
                    left: Atom::Number(length as f64),
                    right: max,
                })
            }
            Stage::MaxCompare(p, max) => match reply.expect("comparison reply") {
                Atom::Bool(false) => {
                    a.single_link_commit(p)?;
                    return self.next_link(a);
                }
                Atom::Bool(true) => {
                    // Template reads current length after coercion, before max string conversion.
                    let edges = a.field_id(p.root, "edges")?;
                    let Node::Array { length, .. } = a.g.nodes[edges as usize] else {
                        return Err(ty("AgentFlow edges must be an Array"));
                    };
                    self.stage = Stage::MaxError(length);
                    PublicInvocationPhase::Host(PublicHostOperation::ToString { value: max })
                }
                _ => return Err(error("Invalid host comparison result").into()),
            },
            Stage::MaxError(length) => {
                let max = text(a.g, &reply.expect("limit string"))?;
                let mut e = error(format!(
                    "Edge limit exceeded. {length} edges found, but the limit is {max}."
                ));
                e.code = ErrorCode::Limit;
                return Err(e.into());
            }
            Stage::SubOptions(p) => {
                self.stage = Stage::SubInherit(p);
                get(reply.expect("options reply"), "inheritDir")
            }
            Stage::SubInherit(p) => {
                self.stage = Stage::SubTruthy(p);
                PublicInvocationPhase::Host(PublicHostOperation::Truthy {
                    value: reply.expect("inheritDir reply"),
                })
            }
            Stage::SubTruthy(mut p) => {
                match reply.expect("truthiness reply") {
                    Atom::Bool(false) => p.dir = Atom::Undefined,
                    Atom::Bool(true) => {
                        p.dir = a.field(p.root, "direction");
                        if matches!(p.dir, Atom::Undefined | Atom::Null) {
                            let root = p.root;
                            self.stage = Stage::SubDirectionOptions(p);
                            return Ok(get(Atom::Ref(root), "options"));
                        }
                    }
                    _ => return Err(error("Invalid host truthiness result").into()),
                }
                PublicInvocationPhase::Complete(a.subgraph_finish(p))
            }
            Stage::SubDirectionOptions(p) => {
                self.stage = Stage::SubDirection(p);
                get(reply.expect("options reply"), "direction")
            }
            Stage::SubDirection(mut p) => {
                p.dir = reply.expect("direction reply");
                PublicInvocationPhase::Complete(a.subgraph_finish(p))
            }
            Stage::LogOptions(p) => {
                let options = reply.expect("options reply");
                self.stage = Stage::LogGet(p, options.clone());
                get(options, "log")
            }
            Stage::LogGet(p, options) => {
                let function = reply.expect("log reply");
                if matches!(function, Atom::Null | Atom::Undefined) {
                    return Ok(complete());
                }
                let id = p.id.clone();
                self.stage = Stage::LogId(p, options, function);
                PublicInvocationPhase::Host(PublicHostOperation::ToString { value: id })
            }
            Stage::LogId(p, options, function) => {
                let Atom::String(id) = reply.expect("id string") else {
                    return Err(error("Invalid host string result").into());
                };
                let id = a.g.strings[id as usize].clone();
                let message = p.message.clone();
                self.stage = Stage::LogMessage(p, options, function, id);
                PublicInvocationPhase::Host(PublicHostOperation::ToString { value: message })
            }
            Stage::LogMessage(p, receiver, function, id) => {
                let Atom::String(message) = reply.expect("message string") else {
                    return Err(error("Invalid host string result").into());
                };
                let mut units = "agentflow[".encode_utf16().collect::<Vec<_>>();
                units.extend(id.0);
                units.extend("]: ".encode_utf16());
                units.extend_from_slice(&a.g.strings[message as usize].0);
                let message = a.string(JsString(units))?;
                self.stage = Stage::LogDone;
                PublicInvocationPhase::Host(PublicHostOperation::Call {
                    function,
                    receiver,
                    arguments: vec![p.severity, message],
                    not_callable_error: "this.options.log is not a function",
                    result: PublicHostResult::Discard,
                })
            }
            Stage::LogDone => complete(),
            Stage::Done => return Err(error("AgentFlow invocation already finished").into()),
        })
    }
}
fn resume(
    state: &mut AgentflowPublicState,
    phase: &mut Invocation,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    let mut a = Arena::new(&mut state.graph);
    a.text_work = phase.text_work;
    let result = reply
        .map(|v| v.into_atom(|s| a.string(s)))
        .transpose()
        .and_then(|reply| phase.step(&mut a, reply));
    phase.text_work = a.text_work;
    a.flush_sets();
    drop(a);
    if let Some(base) = &state.base {
        state.projected = project_render(&state.graph, base, &state.style_bindings);
    }
    match result {
        Ok(step) => step,
        Err(e) => PublicInvocationPhase::Complete(Err(e)),
    }
}
pub(super) fn program(call: MethodCall) -> Option<DetachedPublicInvocation> {
    matches!(call.method.0, 4 | 5 | 17 | 32..=34).then(|| {
        DetachedPublicInvocation::new::<AgentFlow, AgentflowPublicState, _>(
            Invocation {
                stage: Stage::Start(call),
                links: None,
                text_work: 0,
            },
            resume,
        )
    })
}
