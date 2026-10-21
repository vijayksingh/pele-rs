//! Scoped, bounded projection handoff. The normalized Value never escapes parse.
use super::{compatibility_call, parse_document_input, PeleResult};
use pele::compat::{ErrorCode, PeleError, Value};
use std::{ffi::c_void, marker::PhantomData};

const BATCH_SIZE: usize = 128;
const SWIFT_RESULT_LIMIT: usize = 64 * 1024 * 1024;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PeleProjectionEventV1 {
    pub tag: u32,
    pub count: usize,
    pub number: f64,
    pub text: *const u8,
}
impl PeleProjectionEventV1 {
    const EMPTY: Self = Self {
        tag: 0,
        count: 0,
        number: 0.0,
        text: std::ptr::null(),
    };
    fn tagged(tag: u32, count: usize) -> Self {
        Self {
            tag,
            count,
            ..Self::EMPTY
        }
    }
}
pub type PeleProjectionBatchV1 =
    unsafe extern "C" fn(*mut c_void, *const PeleProjectionEventV1, usize) -> u32;

struct Batch<'a> {
    events: [PeleProjectionEventV1; BATCH_SIZE],
    len: usize,
    context: *mut c_void,
    receive: PeleProjectionBatchV1,
    // Every borrowed string remains alive through the last synchronous flush.
    projection: PhantomData<&'a Value>,
}
impl<'a> Batch<'a> {
    fn flush(&mut self) -> Result<(), PeleError> {
        if self.len != 0 {
            // SAFETY: the caller supplies a synchronous C callback. Event storage
            // and its borrowed strings are live until this callback returns.
            let status = unsafe { (self.receive)(self.context, self.events.as_ptr(), self.len) };
            self.len = 0;
            if status != 0 {
                return Err(PeleError::new(
                    "Native projection callback aborted",
                    ErrorCode::Semantic,
                ));
            }
        }
        Ok(())
    }
    fn emit(&mut self, event: PeleProjectionEventV1) -> Result<(), PeleError> {
        self.events[self.len] = event;
        self.len += 1;
        if self.len == BATCH_SIZE {
            self.flush()?;
        }
        Ok(())
    }
    fn text(&mut self, tag: u32, text: &'a str) -> Result<(), PeleError> {
        self.emit(PeleProjectionEventV1 {
            text: text.as_ptr(),
            ..PeleProjectionEventV1::tagged(tag, text.len())
        })
    }
    fn value(&mut self, value: &'a Value) -> Result<(), PeleError> {
        match value {
            Value::Null => self.emit(PeleProjectionEventV1::EMPTY),
            Value::Bool(value) => self.emit(PeleProjectionEventV1::tagged(1, usize::from(*value))),
            Value::Number(value) if value.is_finite() => self.emit(PeleProjectionEventV1 {
                // Value::serialize uses ryu_js, whose JSON spelling of -0 is 0.
                number: if *value == 0.0 { 0.0 } else { *value },
                ..PeleProjectionEventV1::tagged(2, 0)
            }),
            Value::Number(_) => self.emit(PeleProjectionEventV1::EMPTY),
            Value::String(value) => self.text(3, value),
            Value::Array(values) => {
                self.emit(PeleProjectionEventV1::tagged(4, values.len()))?;
                for value in values {
                    self.value(value)?;
                }
                self.emit(PeleProjectionEventV1::tagged(5, 0))
            }
            Value::Object(object) => {
                // Same JS numeric-key ordering as Object::serialize. References
                // borrow object entries, never this temporary sorted vector.
                let entries = object.entries();
                self.emit(PeleProjectionEventV1::tagged(6, entries.len()))?;
                for (key, value) in entries {
                    self.text(7, key)?;
                    self.value(value)?;
                }
                self.emit(PeleProjectionEventV1::tagged(8, 0))
            }
        }
    }
}

/// Parse and eagerly copy normalized model events to a synchronous host consumer.
/// # Safety
/// Inputs must name their readable lengths. Callback/context must remain valid
/// for this call; callback must not unwind or retain event/string pointers.
#[no_mangle]
pub unsafe extern "C" fn pele_parse_eager_v1(
    source: *const u8,
    source_len: usize,
    options: *const u8,
    options_len: usize,
    context: *mut c_void,
    receive: Option<PeleProjectionBatchV1>,
) -> *mut PeleResult {
    compatibility_call(|| {
        let document = unsafe { parse_document_input(source, source_len, options, options_len) }?;
        let model = document.model_view()?;
        // Keep exact old Swift admission and serialize-before-host error order.
        // Counting is remaining serialization work, not a claimed CPU saving.
        #[derive(serde::Serialize)]
        struct Projection<'a> {
            #[serde(rename = "type")]
            kind: &'a pele::compat::DiagramId,
            model: &'a Value,
        }
        let projection_bytes = pele::compat::success_envelope_len(&Projection {
            kind: document.diagram_type(),
            model: &model,
        });
        if projection_bytes <= SWIFT_RESULT_LIMIT {
            let receive = receive.ok_or_else(|| {
                PeleError::new("Native projection callback is missing", ErrorCode::Semantic)
            })?;
            let mut batch = Batch {
                events: [PeleProjectionEventV1::EMPTY; BATCH_SIZE],
                len: 0,
                context,
                receive,
                projection: PhantomData,
            };
            batch.value(&model)?;
            batch.flush()?;
        }
        drop(model);
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Metadata<'a> {
            #[serde(rename = "type")]
            kind: &'a pele::compat::DiagramId,
            projection_bytes: usize,
        }
        let data = pele::compat::success_envelope(&Metadata {
            kind: document.diagram_type(),
            projection_bytes,
        })
        .into_bytes();
        Ok(PeleResult {
            status: 0,
            data,
            document: Some(document),
        })
    })
}
