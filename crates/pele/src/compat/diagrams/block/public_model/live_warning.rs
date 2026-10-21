//! Retained Block hierarchy warning callbacks. The hierarchy algorithm remains
//! shared with native synchronous invocation; host calls happen outside borrows.
use super::*;
pub(super) const FIELDS: &[LiveConstructorField] = &[LiveConstructorField {
    key: "warn",
    methods: &[MethodId(10)],
}];
enum Phase {
    Start(MethodCall),
    Warn(HierarchyWarning),
    Called,
    Done,
}
struct Invocation {
    hierarchy: Option<Hierarchy>,
    phase: Phase,
}
impl Invocation {
    fn advance(&mut self, g: &mut ModelGraph) -> Result<PublicInvocationPhase, PublicError> {
        let hierarchy = self.hierarchy.as_mut().expect("started hierarchy");
        if let Some(warning) = hierarchy.advance(g, 0)? {
            self.phase = Phase::Warn(warning);
            Ok(PublicInvocationPhase::Host(PublicHostOperation::Get {
                receiver: hierarchy.receiver.clone(),
                key: "warn",
            }))
        } else {
            Ok(PublicInvocationPhase::Complete(Ok(Atom::Undefined)))
        }
    }
    fn step(
        &mut self,
        g: &mut ModelGraph,
        reply: Option<PublicHostValue>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        match std::mem::replace(&mut self.phase, Phase::Done) {
            Phase::Start(call) => {
                let size = g.nodes.iter().fold(g.strings.len(), |n, v| {
                    n.saturating_add(v.properties().len())
                });
                if size > 2_000_000 {
                    return Err(limit().into());
                }
                let statements = call.arguments.first().cloned().unwrap_or(Atom::Undefined);
                self.hierarchy = Some(Hierarchy::new(g, call.receiver, statements)?);
                self.advance(g)
            }
            Phase::Warn(warning) => {
                let function = reply
                    .expect("warning getter reply")
                    .into_atom(|s| intern(g, s).map(Atom::String))?;
                if nullish(&function) {
                    return self.advance(g);
                }
                let message = warning.message(g)?;
                self.phase = Phase::Called;
                Ok(PublicInvocationPhase::Host(PublicHostOperation::Call {
                    function,
                    receiver: self.hierarchy.as_ref().unwrap().receiver.clone(),
                    arguments: vec![message],
                    not_callable_error: "this.warn is not a function",
                    result: PublicHostResult::Discard,
                }))
            }
            Phase::Called => self.advance(g),
            Phase::Done => Err(invalid("Block hierarchy invocation already finished").into()),
        }
    }
}
fn resume(
    state: &mut BlockPublicState,
    invocation: &mut Invocation,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    let result = invocation.step(&mut state.graph, reply);
    state.model = imported(&state.graph);
    result.unwrap_or_else(|e| PublicInvocationPhase::Complete(Err(e)))
}
pub(super) fn program(call: MethodCall) -> Option<DetachedPublicInvocation> {
    (call.method == MethodId(10)).then(|| {
        DetachedPublicInvocation::new::<Block, BlockPublicState, _>(
            Invocation {
                hierarchy: None,
                phase: Phase::Start(call),
            },
            resume,
        )
    })
}
