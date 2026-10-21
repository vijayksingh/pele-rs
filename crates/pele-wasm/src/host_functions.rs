//! Scoped caller functions over an active native arena. Only changed records
//! cross callbacks; no global function registry, raw pointers or JS renderer.
use super::{render_numbers, BrowserResources, MutableDocument};
use pele::compat::{
    public::{ActiveModel, Atom, HostFunction, MethodCall, ModelDelta, PublicError},
    CalendarFields, CivilDateTime, PeleError, RenderResources, TextRequest, TrustedSvg,
};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use wasm_bindgen::{closure::ScopedClosure, prelude::*};
const MAX_BYTES: usize = 64 * 1024 * 1024;
fn invalid(message: &str) -> PeleError {
    PeleError::new(message, pele::compat::ErrorCode::Semantic)
}
fn json(value: &impl Serialize) -> Result<String, PeleError> {
    String::from_utf8(render_numbers::encode(value)?)
        .map_err(|_| invalid("Invalid host model JSON"))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Call {
    delta: ModelDelta,
    call: MethodCall,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reply {
    delta: ModelDelta,
    result: Result<Atom, PublicError>,
    #[serde(default = "accepted")]
    accepted: bool,
}
fn accepted() -> bool {
    true
}
struct Function {
    reference: u32,
    dispatcher: js_sys::Function,
}
impl HostFunction for Function {
    fn call(
        &self,
        model: &mut dyn ActiveModel,
        receiver: Atom,
        arguments: &[Atom],
        resources: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        model.begin_sync()?;
        let initial = json(&model.take_delta()?)?;
        let receiver = json(&receiver)?;
        let arguments = json(&arguments)?;
        let result = {
            let mut remaining = 1_000_000usize;
            let mut invoke = |request: String| -> Result<String, JsValue> {
                let mut run = || -> Result<String, PeleError> {
                    remaining = remaining
                        .checked_sub(1)
                        .ok_or_else(|| invalid("Host method call budget exceeded"))?;
                    if request.len() > MAX_BYTES {
                        return Err(invalid("Host method input exceeds byte budget"));
                    }
                    let request: Call = serde_json::from_str(&request)
                        .map_err(|_| invalid("Invalid active model invocation"))?;
                    let applied = model.apply_delta(request.delta);
                    let accepted = applied.is_ok();
                    let result =
                        applied.and_then(|()| model.invoke_builtin(request.call, resources));
                    let delta = model
                        .take_delta()
                        .map_err(|e| invalid(&format!("Active model delta failed: {e:?}")))?;
                    json(&Reply {
                        delta,
                        result,
                        accepted,
                    })
                };
                run().map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
            };
            let closure = ScopedClosure::<dyn FnMut(String) -> Result<String, JsValue>>::borrow_mut(
                &mut invoke,
            );
            self.dispatcher.call5(
                &JsValue::UNDEFINED,
                &JsValue::from(self.reference),
                &JsValue::from_str(&receiver),
                &JsValue::from_str(&arguments),
                closure.as_ref(),
                &JsValue::from_str(&initial),
            )
        }
        .map_err(super::callback_error)?;
        let reply = result
            .as_string()
            .ok_or_else(|| invalid("Host method must return encoded result"))?;
        if reply.len() > MAX_BYTES {
            return Err(invalid("Host method reply exceeds byte budget").into());
        }
        let reply: Reply =
            serde_json::from_str(&reply).map_err(|_| invalid("Invalid host model reply"))?;
        model.apply_delta(reply.delta)?;
        if let Ok(value) = &reply.result {
            model.validate(value)?;
        }
        reply.result
    }
}
pub(super) struct Resources<'a> {
    pub(super) host: &'a mut dyn RenderResources,
    pub(super) dispatcher: js_sys::Function,
}
impl RenderResources for Resources<'_> {
    fn javascript_failure(
        &mut self,
        operation: pele::compat::JavaScriptFailure,
    ) -> pele::compat::public::PublicError {
        self.host.javascript_failure(operation)
    }

    fn host_function(&mut self, reference: u32) -> Result<Rc<dyn HostFunction>, PublicError> {
        Ok(Rc::new(Function {
            reference,
            dispatcher: self.dispatcher.clone(),
        }))
    }
    fn image(&mut self, uri: &str) -> Result<pele::compat::ImageResource, PeleError> {
        self.host.image(uri)
    }
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.host.text_width(r)
    }
    fn icon(&mut self, s: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.icon(s)
    }
    fn direct_icon(&mut self, s: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.direct_icon(s)
    }
    fn raw_inline_icon(&mut self, s: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.raw_inline_icon(s)
    }
    fn presentation_budget(&self) -> Option<pele::compat::PresentationBudget> {
        self.host.presentation_budget()
    }
    fn capture_presentation(&mut self, v: pele::compat::PresentationOutcome) {
        self.host.capture_presentation(v)
    }
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.direct_now_millis()
    }
    fn now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.now_millis()
    }
    fn parse_date(&mut self, s: &str) -> Result<Option<f64>, PeleError> {
        self.host.parse_date(s)
    }
    fn calendar_fields(&mut self, v: f64) -> Result<Option<CalendarFields>, PeleError> {
        self.host.calendar_fields(v)
    }
    fn calendar_instant(&mut self, v: CivilDateTime) -> Result<Option<f64>, PeleError> {
        self.host.calendar_instant(v)
    }
}
#[wasm_bindgen]
impl MutableDocument {
    /// Public captured method calls share the same owner-bound callable arena.
    pub fn invoke_binary_with_functions(
        &mut self,
        revision: &str,
        call_json: &str,
        dispatcher: js_sys::Function,
    ) -> Result<Vec<u8>, JsValue> {
        let run = || -> Result<Vec<u8>, PeleError> {
            if call_json.len() > MAX_BYTES {
                return Err(invalid(
                    "Public method JSON exceeds 64 MiB transport budget",
                ));
            }
            let revision = revision
                .parse::<u64>()
                .map_err(|_| invalid("Invalid public model revision"))?;
            let call = serde_json::from_str(call_json)
                .map_err(|_| invalid("Invalid public model method call"))?;
            let mut host = BrowserResources {
                width: None,
                icons: None,
            };
            let outcome = self.inner.invoke(
                revision,
                call,
                &mut Resources {
                    host: &mut host,
                    dispatcher,
                },
            )?;
            let metadata = render_numbers::encode(&outcome.result)?;
            pele::compat::public::encode_binary_exchange(
                outcome.revision,
                &outcome.graph,
                &metadata,
                pele::compat::public::DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
    }

    pub fn invoke_source_layout_with_functions(
        &mut self,
        revision: &str,
        call_json: &str,
        dispatcher: js_sys::Function,
    ) -> Result<JsValue, JsValue> {
        let run = || -> Result<JsValue, PeleError> {
            if call_json.len() > MAX_BYTES {
                return Err(invalid(
                    "Public method JSON exceeds 64 MiB transport budget",
                ));
            }
            let revision = revision
                .parse::<u64>()
                .map_err(|_| invalid("Invalid public model revision"))?;
            let call = serde_json::from_str(call_json)
                .map_err(|_| invalid("Invalid public model method call"))?;
            let mut host = BrowserResources {
                width: None,
                icons: None,
            };
            let outcome = self.inner.invoke_read(
                revision,
                call,
                &mut Resources {
                    host: &mut host,
                    dispatcher,
                },
            )?;
            let metadata = render_numbers::encode(&outcome.result)?;
            super::source_layout::outcome(outcome.revision, &outcome.graph, &metadata)
        };
        run().map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_numbers_operation_binary_with_functions(
        &mut self,
        revision: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
        dispatcher: js_sys::Function,
    ) -> Result<Vec<u8>, JsValue> {
        let run = || -> Result<Vec<u8>, PeleError> {
            let (options, limit) =
                render_numbers::options(json, mask, numbers, output_kind, output)?;
            let revision = revision
                .parse::<u64>()
                .map_err(|_| invalid("Invalid public model revision"))?;
            let mut host = render_numbers::NumericResources {
                browser: BrowserResources { width, icons },
                clock,
            };
            let outcome = self.inner.render_operation_bounded(
                revision,
                &options,
                limit,
                &mut Resources {
                    host: &mut host,
                    dispatcher,
                },
            )?;
            let metadata = render_numbers::encode(
                &outcome
                    .result
                    .as_ref()
                    .map(pele::compat::render_result_transport),
            )?;
            pele::compat::public::encode_binary_exchange(
                outcome.revision,
                &outcome.graph,
                &metadata,
                pele::compat::public::DEFAULT_BINARY_BUDGET,
            )
        };
        run().map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))
    }
    #[allow(clippy::too_many_arguments)]
    pub fn render_numbers_operation_packet_with_functions(
        &mut self,
        revision: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
        dispatcher: js_sys::Function,
    ) -> Result<JsValue, JsValue> {
        let run = || -> Result<JsValue, JsValue> {
            let (options, limit) =
                render_numbers::options(json, mask, numbers, output_kind, output)
                    .map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))?;
            let revision = revision.parse::<u64>().map_err(|_| {
                JsValue::from_str(&render_numbers::failure(&invalid(
                    "Invalid public model revision",
                )))
            })?;
            let mut host = render_numbers::NumericResources {
                browser: BrowserResources { width, icons },
                clock,
            };
            let outcome = self
                .inner
                .render_operation_bounded_view(
                    revision,
                    &options,
                    limit,
                    &mut Resources {
                        host: &mut host,
                        dispatcher,
                    },
                )
                .map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))?;
            let transport = outcome
                .result
                .as_ref()
                .map(pele::compat::render_result_transport);
            super::render_packet::arena(
                outcome.revision,
                &outcome.graph,
                false,
                outcome
                    .result
                    .as_ref()
                    .map_or("", |value| value.diagram_type.as_str()),
                outcome.result.as_ref().ok().map(|value| &value.rendered),
                &transport,
            )
        };
        run()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_numbers_operation_source_layout_with_functions(
        &mut self,
        revision: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
        dispatcher: js_sys::Function,
    ) -> Result<JsValue, JsValue> {
        let run = || -> Result<JsValue, JsValue> {
            let (options, limit) =
                render_numbers::options(json, mask, numbers, output_kind, output)
                    .map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))?;
            let revision = revision.parse::<u64>().map_err(|_| {
                JsValue::from_str(&render_numbers::failure(&invalid(
                    "Invalid public model revision",
                )))
            })?;
            let mut host = render_numbers::NumericResources {
                browser: BrowserResources { width, icons },
                clock,
            };
            let outcome = self
                .inner
                .render_operation_bounded_read(
                    revision,
                    &options,
                    limit,
                    &mut Resources {
                        host: &mut host,
                        dispatcher,
                    },
                )
                .map_err(|error| JsValue::from_str(&render_numbers::failure(&error)))?;
            let transport = outcome
                .result
                .as_ref()
                .map(pele::compat::render_result_transport);
            super::source_layout::render(
                outcome.revision,
                &outcome.graph,
                false,
                outcome
                    .result
                    .as_ref()
                    .map_or("", |value| value.diagram_type.as_str()),
                outcome.result.as_ref().ok().map(|value| &value.rendered),
                &transport,
            )
        };
        run()
    }
}
