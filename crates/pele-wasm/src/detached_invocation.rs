//! Short owner borrows around native continuations; host operations happen in JS.
use super::{render_numbers, MutableDocument};
use pele::compat::{
    public::{
        DetachedInvocationJob, DetachedInvocationStep, HostExceptionToken, JsString, PublicError,
        PublicHostOperation, PublicHostResult, PublicHostValue, PublicResourceOperation,
        DEFAULT_BINARY_BUDGET,
    },
    ErrorCode, PeleError,
};
use std::rc::Rc;
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
    resource: ResourceSlot,
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
                .map(|job| {
                    job.map(|inner| PublicInvocationJob {
                        inner,
                        resource: ResourceSlot::default(),
                    })
                })
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
            let reply = job.resource.reply(reply_state, value_tag, &value)?;
            let step = self
                .inner
                .resume_detached_invocation(&mut job.inner, reply)?;
            let mut metadata = job.resource.metadata(step);
            error_context_metadata(&mut metadata, job.inner.error_context());
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
        DetachedInvocationStep::Resource(_) => {
            unreachable!("resource step retained before metadata")
        }
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

/// The resource is a separate owner so neither the document nor a provisional
/// constructor job is borrowed during callbacks. Replies never traverse JS.
#[wasm_bindgen(js_name=PublicInvocationResource)]
pub struct PublicInvocationResource {
    origin: Rc<()>,
    operation: Option<PublicResourceOperation>,
    result: Option<Result<PublicHostValue, PublicError>>,
}
#[wasm_bindgen]
impl PublicInvocationResource {
    pub fn run(&mut self, dispatcher: js_sys::Function) -> Result<(), JsValue> {
        let operation = self
            .operation
            .take()
            .ok_or_else(|| failure(invalid("Resource operation already consumed")))?;
        let mut host = super::BrowserResources {
            width: None,
            icons: None,
        };
        let mut calendar = super::browser_calendar::Resources {
            browser: &mut host,
            error: None,
        };
        self.result = Some(operation.run(&mut super::host_functions::Resources {
            host: &mut calendar,
            dispatcher,
        }));
        // Return the original JS value only after operation.run has unwound its
        // native borrows. wasm-bindgen then releases this resource's borrow
        // before JS observes the throw and executes its resource.free finally.
        if let Some(error) = calendar.error.take() {
            self.result = None;
            return Err(error);
        }
        Ok(())
    }
}
#[derive(Default)]
pub(super) struct ResourceSlot {
    origin: Option<Rc<()>>,
    operation: Option<PublicResourceOperation>,
    result: Option<Result<PublicHostValue, PublicError>>,
    waiting: bool,
}
impl ResourceSlot {
    pub(super) fn metadata(&mut self, step: DetachedInvocationStep) -> serde_json::Value {
        match step {
            DetachedInvocationStep::Resource(operation) => {
                self.origin = Some(Rc::new(()));
                self.operation = Some(operation);
                self.waiting = true;
                serde_json::json!({"resource":true})
            }
            other => step_metadata(other),
        }
    }
    pub(super) fn take(&mut self) -> Result<PublicInvocationResource, JsValue> {
        let operation = self
            .operation
            .take()
            .ok_or_else(|| failure(invalid("No pending resource operation")))?;
        Ok(PublicInvocationResource {
            origin: Rc::clone(self.origin.as_ref().expect("pending resource origin")),
            operation: Some(operation),
            result: None,
        })
    }
    pub(super) fn accept(
        &mut self,
        resource: &mut PublicInvocationResource,
    ) -> Result<(), JsValue> {
        if !self.waiting
            || self.operation.is_some()
            || self.result.is_some()
            || !self
                .origin
                .as_ref()
                .is_some_and(|origin| Rc::ptr_eq(origin, &resource.origin))
        {
            return Err(failure(invalid("Unexpected resource reply")));
        }
        self.result = Some(
            resource
                .result
                .take()
                .ok_or_else(|| failure(invalid("Resource operation has not completed")))?,
        );
        Ok(())
    }
    pub(super) fn reply(
        &mut self,
        state: u32,
        tag: u32,
        value: &JsValue,
    ) -> Result<Option<Result<PublicHostValue, PublicError>>, PeleError> {
        if self.waiting {
            let reply = match state {
                4 => Some(
                    self.result
                        .take()
                        .ok_or_else(|| invalid("Resource reply is not ready"))?,
                ),
                2 => host_reply(state, tag, value)?,
                _ => return Err(invalid("Resource operation requires its owned reply")),
            };
            self.operation = None;
            self.result = None;
            self.waiting = false;
            self.origin = None;
            Ok(reply)
        } else {
            host_reply(state, tag, value)
        }
    }
}
#[wasm_bindgen]
impl PublicInvocationJob {
    pub fn take_resource(&mut self) -> Result<PublicInvocationResource, JsValue> {
        self.resource.take()
    }
    pub fn accept_resource(
        &mut self,
        resource: &mut PublicInvocationResource,
    ) -> Result<(), JsValue> {
        self.resource.accept(resource)
    }
}

/// Expression metadata changes no operation, graph, revision or native result.
pub(super) fn error_context_metadata(
    metadata: &mut serde_json::Value,
    context: Option<&pele::compat::public::PublicHostErrorContext>,
) {
    if let Some(context) = context {
        if let Some(host) = metadata.get_mut("host") {
            host["errorExpression"] = serde_json::json!(context.expression);
        } else if metadata.get("complete") == Some(&serde_json::Value::Bool(true)) {
            metadata["errorContext"] = serde_json::json!(context);
        }
    }
}
