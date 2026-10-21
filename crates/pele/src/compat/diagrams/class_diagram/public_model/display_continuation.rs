//! The existing getDisplayDetails body, suspended only around parseClassifier.
use super::*;
use crate::compat::public::{
    DetachedPublicInvocation, PublicHostOperation, PublicHostResult, PublicHostValue,
    PublicInvocationPhase,
};
enum Phase {
    Start(MethodCall),
    Function { receiver: Atom, display: JsString },
    Result { display: JsString },
    Done,
}
struct Program {
    phase: Phase,
    work: usize,
}
pub(super) fn prepare(call: MethodCall) -> Option<DetachedPublicInvocation> {
    (call.method.0 == 26).then(|| {
        DetachedPublicInvocation::new::<Class, ClassPublicState, _>(
            Program {
                phase: Phase::Start(call),
                work: 0,
            },
            resume,
        )
    })
}
fn resume(
    state: &mut ClassPublicState,
    program: &mut Program,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    let mut c = Context::new(&mut state.graph);
    c.work = program.work;
    let phase = &mut program.phase;
    let result = (|| {
        Ok(match std::mem::replace(phase, Phase::Done) {
            Phase::Start(call) => {
                c.tick(1)?;
                c.validate(&call.receiver)?;
                let display = c.display_text(&call.receiver)?;
                let display = ActiveModel::string(&mut c, &display)?;
                c.tick(1)?;
                *phase = Phase::Function {
                    receiver: call.receiver.clone(),
                    display,
                };
                PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver: call.receiver,
                    key: "parseClassifier",
                })
            }
            Phase::Function { receiver, display } => {
                let function = reply
                    .ok_or_else(|| invalid("Missing Class classifier function"))?
                    .into_atom(|s| c.intern(s))?;
                *phase = Phase::Result { display };
                PublicInvocationPhase::Host(PublicHostOperation::Call {
                    function,
                    receiver,
                    arguments: vec![],
                    not_callable_error: "this.parseClassifier is not a function",
                    result: PublicHostResult::Value,
                })
            }
            Phase::Result { display } => {
                let css = reply
                    .ok_or_else(|| invalid("Missing Class classifier result"))?
                    .into_atom(|s| c.intern(s))?;
                let display = c.intern(display)?;
                PublicInvocationPhase::Complete(
                    c.record(vec![("displayText", display), ("cssStyle", css)]),
                )
            }
            Phase::Done => return Err(invalid("Class display continuation is closed").into()),
        })
    })();
    program.work = c.work;
    drop(c);
    state.model = imported(&state.graph);
    match result {
        Ok(step) => step,
        Err(error) => PublicInvocationPhase::Complete(Err(error)),
    }
}
