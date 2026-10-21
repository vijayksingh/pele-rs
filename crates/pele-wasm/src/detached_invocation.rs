//! Short owner borrows around native continuations; host operations happen in JS.
use super::{render_numbers, MutableDocument};
use pele::compat::{
    public::{
        DetachedInvocationJob, DetachedInvocationStep, HostExceptionToken, JsString, PublicError,
        PublicHostOperation, PublicHostResult, PublicHostValue, DEFAULT_BINARY_BUDGET,
    },
    ErrorCode, PeleError,
};
use wasm_bindgen::prelude::*;

fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn failure(error: PeleError) -> JsValue {
    JsValue::from_str(&render_numbers::failure(&error))
}
fn token(value: &JsValue) -> Result<u32, PeleError> {
    let n = value
        .as_f64()
        .ok_or_else(|| invalid("Invalid owner host token"))?;
    if !n.is_finite() || n.fract() != 0. || !(0. ..1_000_000.).contains(&n) {
        return Err(invalid("Invalid owner host token"));
    }
    Ok(n as u32)
}
pub(super) fn host_value(tag: u32, value: &JsValue) -> Result<PublicHostValue, PeleError> {
    Ok(match tag {
        0 => PublicHostValue::Undefined,
        1 => PublicHostValue::Null,
        2 => PublicHostValue::Bool(
            value
                .as_bool()
                .ok_or_else(|| invalid("Invalid host Boolean"))?,
        ),
        3 => PublicHostValue::Number(
            value
                .as_f64()
                .ok_or_else(|| invalid("Invalid host number"))?,
        ),
        4 => {
            if !value.is_string() {
                return Err(invalid("Invalid host string"));
            }
            let text: &js_sys::JsString = value.unchecked_ref();
            if text.length() > 16_000_000 {
                return Err(PeleError::new(
                    "Public model graph exceeds work budget",
                    ErrorCode::Limit,
                ));
            }
            PublicHostValue::String(JsString(
                (0..text.length())
                    .map(|i| text.char_code_at(i) as u16)
                    .collect(),
            ))
        }
        5 => PublicHostValue::Opaque(token(value)?),
        _ => return Err(invalid("Invalid host value tag")),
    })
}
#[wasm_bindgen(js_name=PublicInvocationJob)]
pub struct PublicInvocationJob {
    inner: DetachedInvocationJob,
}
#[wasm_bindgen]
impl MutableDocument {
    pub fn prepare_detached_invocation(
        &mut self,
        revision: &str,
        call_json: &str,
    ) -> Result<Option<PublicInvocationJob>, JsValue> {
        let mut run = || -> Result<Option<PublicInvocationJob>, PeleError> {
            if call_json.len() > 64 * 1024 * 1024 {
                return Err(invalid(
                    "Public method JSON exceeds 64 MiB transport budget",
                ));
            }
            let revision = revision
                .parse()
                .map_err(|_| invalid("Invalid public model revision"))?;
            let call = serde_json::from_str(call_json)
                .map_err(|_| invalid("Invalid public model method call"))?;
            self.inner
                .prepare_detached_invocation(revision, call)
                .map(|job| job.map(|inner| PublicInvocationJob { inner }))
        };
        run().map_err(failure)
    }
    pub fn resume_detached_invocation(
        &mut self,
        job: &mut PublicInvocationJob,
        reply_state: u32,
        value_tag: u32,
        value: JsValue,
    ) -> Result<Vec<u8>, JsValue> {
        let mut run = || -> Result<Vec<u8>, PeleError> {
            let reply = host_reply(reply_state, value_tag, &value)?;
            let step = self
                .inner
                .resume_detached_invocation(&mut job.inner, reply)?;
            let metadata = step_metadata(step);
            let metadata = render_numbers::encode(&metadata)?;
            let graph = self.inner.snapshot_read()?.into_model()?;
            pele::compat::public::encode_binary_exchange(
                self.inner.revision(),
                &graph,
                &metadata,
                DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(failure)
    }
    pub fn finish_detached_invocation(
        &mut self,
        job: &mut PublicInvocationJob,
    ) -> Result<Vec<u8>, JsValue> {
        let mut run = || -> Result<Vec<u8>, PeleError> {
            let outcome = self.inner.finish_detached_invocation(&mut job.inner)?;
            let metadata = render_numbers::encode(&outcome.result)?;
            let graph = outcome.graph.into_model()?;
            pele::compat::public::encode_binary_exchange(
                outcome.revision,
                &graph,
                &metadata,
                DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(failure)
    }
}

pub(super) fn host_reply(
    reply_state: u32,
    value_tag: u32,
    value: &JsValue,
) -> Result<Option<Result<PublicHostValue, PublicError>>, PeleError> {
    let reply = match reply_state {
        0 => None,
        1 => Some(Ok(host_value(value_tag, value)?)),
        2 => Some(Err(PublicError::HostException(HostExceptionToken(token(
            value,
        )?)))),
        _ => return Err(invalid("Invalid host reply state")),
    };

    Ok(reply)
}

pub(super) fn step_metadata(step: DetachedInvocationStep) -> serde_json::Value {
    match step {
        DetachedInvocationStep::Complete => serde_json::json!({"complete":true}),
        DetachedInvocationStep::Host(PublicHostOperation::GreaterEqual { left, right }) => {
            serde_json::json!({"host":{"kind":"greaterEqual","left":left,"right":right}})
        }
        DetachedInvocationStep::Host(PublicHostOperation::StrictEqual { left, right }) => {
            serde_json::json!({"host":{"kind":"strictEqual","left":left,"right":right}})
        }
        DetachedInvocationStep::Host(PublicHostOperation::ToString { value }) => {
            serde_json::json!({"host":{"kind":"toString","value":value}})
        }
        DetachedInvocationStep::Host(PublicHostOperation::Truthy { value }) => {
            serde_json::json!({"host":{"kind":"truthy","value":value}})
        }
        DetachedInvocationStep::Host(PublicHostOperation::Get { receiver, key }) => {
            serde_json::json!({"host":{"kind":"get","receiver":receiver,"key":key}})
        }
        DetachedInvocationStep::Host(PublicHostOperation::Call {
            function,
            receiver,
            arguments,
            not_callable_error,
            result,
        }) => {
            serde_json::json!({"host":{"kind":"call","function":function,"receiver":receiver,"arguments":arguments,"notCallableError":not_callable_error,"result":match result {PublicHostResult::Discard=>"discard",PublicHostResult::Retain=>"retain",PublicHostResult::Value=>"value"}}})
        }
    }
}
