//! Native ClassMember constructor body. Helpers remain the existing public
//! parseMethod/parseAttribute implementation; host requests preserve dispatch.
use super::*;
use crate::compat::public::{
    DetachedPublicInvocation, PublicHostOperation, PublicHostResult, PublicHostValue,
    PublicInvocationPhase,
};

pub(super) fn factory() -> PublicConstructorFactory<ClassPublicState> {
    PublicConstructorFactory::two_value_body::<Class>(MEMBER, initial, body, &[])
}
fn initial(arguments: [PublicHostValue; 2]) -> Result<ClassPublicState, PeleError> {
    let mut writer = GraphWriter::default();
    let empty = text(&mut writer, "");
    let member_type = arguments[1]
        .clone()
        .into_atom(|value| Ok::<_, PeleError>(Atom::String(writer.string(value))))?;
    let root = rec(
        &mut writer,
        vec![
            ("id", empty.clone()),
            ("memberType", member_type),
            ("visibility", empty.clone()),
            ("classifier", empty.clone()),
            ("parameters", empty.clone()),
            ("returnType", empty),
        ],
        Some(MEMBER),
    );
    let graph = writer.finish(Atom::Ref(root))?;
    let model = imported(&graph);
    Ok(ClassPublicState { graph, model })
}
enum Phase {
    Start([PublicHostValue; 2]),
    Method(Atom),
    MethodResult(Atom),
    MethodTruth(Atom),
    Attribute(Atom),
    AttributeResult,
    IdPrefix,
    IdTrim(bool),
    Done,
}
fn body(arguments: [PublicHostValue; 2]) -> DetachedPublicInvocation {
    DetachedPublicInvocation::new::<Class, ClassPublicState, _>(Phase::Start(arguments), resume)
}
fn get(receiver: Atom, key: &'static str) -> PublicInvocationPhase {
    PublicInvocationPhase::Host(PublicHostOperation::Get { receiver, key })
}
fn resume(
    state: &mut ClassPublicState,
    phase: &mut Phase,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    fn run(
        state: &mut ClassPublicState,
        phase: &mut Phase,
        reply: Option<PublicHostValue>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        let root = state.graph.root.clone();
        let mut context = Context::new(&mut state.graph);
        Ok(match std::mem::replace(phase, Phase::Done) {
            Phase::Start([input, kind]) => {
                let input = input.into_atom(|s| context.intern(s))?;
                if matches!(kind, PublicHostValue::String(ref s) if s.is("method")) {
                    *phase = Phase::Method(input);
                    get(root, "parseMethod")
                } else {
                    *phase = Phase::Attribute(input);
                    get(root, "parseAttribute")
                }
            }
            Phase::Method(input) => {
                let function = reply
                    .expect("parseMethod reply")
                    .into_atom(|s| context.intern(s))?;
                *phase = Phase::MethodResult(input.clone());
                PublicInvocationPhase::Host(PublicHostOperation::Call {
                    function,
                    receiver: root,
                    arguments: vec![input],
                    not_callable_error: "this.parseMethod is not a function",
                    result: PublicHostResult::Retain,
                })
            }
            Phase::MethodResult(input) => {
                let value = reply
                    .expect("parseMethod result")
                    .into_atom(|s| context.intern(s))?;
                *phase = Phase::MethodTruth(input);
                PublicInvocationPhase::Host(PublicHostOperation::Truthy { value })
            }
            Phase::MethodTruth(input) => {
                let PublicHostValue::Bool(parsed) = reply.expect("truthiness reply") else {
                    return Err(invalid("Invalid ClassMember truthiness reply").into());
                };
                if parsed {
                    *phase = Phase::IdPrefix;
                    get(root, "id")
                } else {
                    *phase = Phase::Attribute(input);
                    get(root, "parseAttribute")
                }
            }
            Phase::Attribute(input) => {
                let function = reply
                    .expect("parseAttribute reply")
                    .into_atom(|s| context.intern(s))?;
                *phase = Phase::AttributeResult;
                PublicInvocationPhase::Host(PublicHostOperation::Call {
                    function,
                    receiver: root,
                    arguments: vec![input],
                    not_callable_error: "this.parseAttribute is not a function",
                    result: PublicHostResult::Discard,
                })
            }
            Phase::AttributeResult => {
                *phase = Phase::IdPrefix;
                get(root, "id")
            }
            Phase::IdPrefix => {
                let value = reply
                    .expect("id prefix reply")
                    .into_atom(|s| context.intern(s))?;
                let units = context.string_units(&value, "this.id.startsWith")?;
                *phase = Phase::IdTrim(units.first() == Some(&32));
                get(root, "id")
            }
            Phase::IdTrim(prefix) => {
                let value = reply
                    .expect("id trim reply")
                    .into_atom(|s| context.intern(s))?;
                let units = context.string_units(&value, "this.id.trim")?;
                let mut normalized = Vec::new();
                if prefix {
                    normalized.push(32);
                }
                normalized.extend_from_slice(trimmed(&units));
                let value = context.intern(JsString(normalized))?;
                context.set(&root, "id", value)?;
                PublicInvocationPhase::Complete(Ok(Atom::Undefined))
            }
            Phase::Done => return Err(invalid("ClassMember constructor already finished").into()),
        })
    }
    let result = run(state, phase, reply);
    state.model = imported(&state.graph);
    match result {
        Ok(phase) => phase,
        Err(error) => PublicInvocationPhase::Complete(Err(error)),
    }
}
