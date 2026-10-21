//! Owned render jobs release the mutable WASM document before host callbacks.
//! Finishing admits the current native graph; result factories run only on Ack.
use super::{prepared_entries, render_numbers, render_packet, MutableDocument};
use pele::compat::{
    public::{
        DetachedRenderCompletion, DetachedRenderJob, DetachedRenderRequest, DetachedRenderValue,
        JsString, LiveCallArgument, LiveRenderRead, LiveStringUse, LiveValueKind, PublicError,
        DEFAULT_BINARY_BUDGET,
    },
    ErrorCode, PeleError,
};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../live-render.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = executeLiveRead)]
    fn execute_live_read(
        read: &js_sys::Function,
        command: u32,
        receiver: &JsValue,
        key: &JsValue,
        context: u32,
    ) -> Result<JsValue, JsValue>;
}

#[repr(u32)]
enum ReadCommand {
    Get = 0,
    Index = 1,
    ToString = 2,
    RequireString = 3,
    SnapshotIterable = 4,
    HasIndex = 5,
    ShallowCopy = 6,
    MapGet = 7,
    ObjectIdentity = 8,
    ArrayJoin = 9,
    OwnKeys = 10,
    SnapshotMapValues = 11,
    IsArray = 12,
    Call = 13,
    IteratorBegin = 14,
    IteratorNext = 15,
    IteratorClose = 16,
    RetainedGraphValue = 17,
    ToNumber = 18,
    RetainValue = 19,
    RetainedValue = 20,
}

fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn failure(error: PeleError) -> JsValue {
    JsValue::from_str(&render_numbers::failure(&error))
}
fn revision(value: &str) -> Result<u64, JsValue> {
    value
        .parse()
        .map_err(|_| failure(invalid("Invalid public model revision")))
}

struct Reader {
    root: JsValue,
    read: js_sys::Function,
    keys: HashMap<String, JsValue>,
    key_bytes: usize,
}
impl Reader {
    fn key(&mut self, key: &str) -> JsValue {
        if let Some(value) = self.keys.get(key) {
            return value.clone();
        }
        let value = JsValue::from_str(key);
        // Retain immutable control operands only, never public field values.
        // This bounds cache storage without adding a new rejection/limit to
        // dynamic keys supplied by a future renderer.
        if self.keys.len() < 128 && key.len() <= 16 * 1024 - self.key_bytes {
            self.key_bytes += key.len();
            self.keys.insert(key.to_owned(), value.clone());
        }
        value
    }
    fn primitive(
        &self,
        command: ReadCommand,
        receiver: &JsValue,
        key: &JsValue,
        context: u32,
    ) -> Result<JsValue, PublicError> {
        execute_live_read(&self.read, command as u32, receiver, key, context)
            // The JS job channel retains the original thrown value, including
            // undefined. Do not inspect it here and invoke a second getter.
            .map_err(|_| invalid("Host callback failed").into())
    }
    fn units(value: &JsValue) -> Result<Option<JsString>, PublicError> {
        if !value.is_string() {
            return Ok(None);
        }
        let string: &js_sys::JsString = value.unchecked_ref();
        if string.length() > 16_000_000 {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        Ok(Some(JsString(
            (0..string.length())
                .map(|i| string.char_code_at(i) as u16)
                .collect(),
        )))
    }
}
impl LiveRenderRead for Reader {
    type Value = JsValue;
    fn call(
        &mut self,
        function: &JsValue,
        receiver: &JsValue,
        arguments: &[LiveCallArgument<JsValue>],
        _resources: &mut dyn pele::compat::RenderResources,
        not_callable_error: &'static str,
    ) -> Result<JsValue, PublicError> {
        if arguments.len() > 1_000_000 {
            return Err(PeleError::new("Live call exceeds work budget", ErrorCode::Limit).into());
        }
        let mut total = 0usize;
        let mut fields = 0usize;
        for argument in arguments {
            match argument {
                LiveCallArgument::Text(text) => total = total.saturating_add(text.0.len()),
                LiveCallArgument::Record(properties) => {
                    fields = fields.saturating_add(properties.len());
                    for (key, _) in properties {
                        total = total.saturating_add(key.0.len());
                    }
                }
                _ => {}
            }
            if fields > 1_000_000 {
                return Err(
                    PeleError::new("Live call exceeds field budget", ErrorCode::Limit).into(),
                );
            }
            if total > 16_000_000 {
                return Err(
                    PeleError::new("Live call exceeds text budget", ErrorCode::Limit).into(),
                );
            }
        }
        fn text_value(text: &JsString) -> JsValue {
            let mut result = js_sys::JsString::from("");
            for chunk in text.0.chunks(4096) {
                result = result.concat(&js_sys::JsString::from_char_code(chunk));
            }
            result.into()
        }
        let args = js_sys::Array::new();
        let dates = js_sys::Array::new();
        let records = js_sys::Array::new();
        for argument in arguments {
            args.push(&match argument {
                LiveCallArgument::Value(v) => v.clone(),
                LiveCallArgument::Number(value) => JsValue::from_f64(*value),
                LiveCallArgument::Date(value) => {
                    dates.push(&JsValue::from_f64(args.length() as f64));
                    JsValue::from_f64(*value)
                }
                LiveCallArgument::Text(text) => text_value(text),
                LiveCallArgument::Record(properties) => {
                    let fields = js_sys::Array::new();
                    for (key, value) in properties {
                        let field = js_sys::Array::new();
                        field.push(&text_value(key));
                        field.push(value);
                        fields.push(&field);
                    }
                    let record = js_sys::Array::new();
                    record.push(&JsValue::from_f64(args.length() as f64));
                    record.push(&fields);
                    records.push(&record);
                    JsValue::UNDEFINED
                }
            });
        }
        let packet = js_sys::Array::new();
        packet.push(receiver);
        packet.push(&args);
        packet.push(&JsValue::from_str(not_callable_error));
        packet.push(&dates);
        packet.push(&records);
        Reader::primitive(self, ReadCommand::Call, function, &packet, 0)
    }
    fn to_number(
        &mut self,
        value: &JsValue,
        _resources: &mut dyn pele::compat::RenderResources,
    ) -> Result<f64, PublicError> {
        Reader::primitive(self, ReadCommand::ToNumber, value, &JsValue::UNDEFINED, 0)?
            .as_f64()
            .ok_or_else(|| invalid("Invalid live numeric conversion reply").into())
    }
    fn retain_value(&mut self, value: &JsValue) -> Result<u32, PublicError> {
        let slot = Reader::primitive(
            self,
            ReadCommand::RetainValue,
            value,
            &JsValue::UNDEFINED,
            0,
        )?;
        if slot.is_null() {
            return Err(
                PeleError::new("Retained live value budget exceeded", ErrorCode::Limit).into(),
            );
        }
        let slot = slot
            .as_f64()
            .ok_or_else(|| invalid("Invalid retained live value token"))?;
        if !slot.is_finite() || slot < 1. || slot > u32::MAX as f64 || slot.fract() != 0. {
            return Err(invalid("Invalid retained live value token").into());
        }
        Ok(slot as u32)
    }
    fn retained_value(&mut self, slot: u32) -> Result<JsValue, PublicError> {
        Reader::primitive(
            self,
            ReadCommand::RetainedValue,
            &JsValue::UNDEFINED,
            &JsValue::from_f64(slot as f64),
            0,
        )
    }
    fn iterator_begin(&mut self, receiver: &JsValue) -> Result<JsValue, PublicError> {
        let value = Reader::primitive(
            self,
            ReadCommand::IteratorBegin,
            receiver,
            &JsValue::UNDEFINED,
            0,
        )?;
        if value.is_null() {
            return Err(
                PeleError::new("Live iteration exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        Ok(value)
    }
    fn iterator_next(&mut self, iterator: &JsValue) -> Result<Option<JsValue>, PublicError> {
        let value = Reader::primitive(
            self,
            ReadCommand::IteratorNext,
            iterator,
            &JsValue::UNDEFINED,
            0,
        )?;
        if value.is_null() {
            let _ = Reader::primitive(
                self,
                ReadCommand::IteratorClose,
                iterator,
                &JsValue::UNDEFINED,
                0,
            );
            return Err(
                PeleError::new("Live iteration exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let packet: js_sys::Array = value
            .dyn_into()
            .map_err(|_| invalid("Invalid live iterator reply"))?;
        if packet.length() != 2 {
            return Err(invalid("Invalid live iterator reply").into());
        }
        match packet.get(0).as_bool() {
            Some(true) => Ok(None),
            Some(false) => Ok(Some(packet.get(1))),
            None => Err(invalid("Invalid live iterator completion").into()),
        }
    }
    fn iterator_close(&mut self, iterator: &JsValue) -> Result<(), PublicError> {
        Reader::primitive(
            self,
            ReadCommand::IteratorClose,
            iterator,
            &JsValue::UNDEFINED,
            0,
        )
        .map(|_| ())
    }
    fn retained_graph_value(&mut self, node: u32) -> Result<JsValue, PublicError> {
        Reader::primitive(
            self,
            ReadCommand::RetainedGraphValue,
            &JsValue::UNDEFINED,
            &JsValue::from_f64(node as f64),
            0,
        )
    }
    fn root(&self) -> JsValue {
        self.root.clone()
    }
    fn get(&mut self, receiver: &JsValue, key: &str) -> Result<JsValue, PublicError> {
        let key = self.key(key);
        self.primitive(ReadCommand::Get, receiver, &key, 0)
    }
    fn index(&mut self, receiver: &JsValue, index: usize) -> Result<JsValue, PublicError> {
        self.primitive(
            ReadCommand::Index,
            receiver,
            &JsValue::from_f64(index as f64),
            0,
        )
    }
    fn has_index(&mut self, receiver: &JsValue, index: usize) -> Result<bool, PublicError> {
        self.primitive(
            ReadCommand::HasIndex,
            receiver,
            &JsValue::from_f64(index as f64),
            0,
        )?
        .as_bool()
        .ok_or_else(|| invalid("Invalid live membership reply").into())
    }
    fn shallow_copy(&mut self, value: &JsValue) -> Result<JsValue, PublicError> {
        self.primitive(ReadCommand::ShallowCopy, value, &JsValue::UNDEFINED, 0)
    }
    fn map_get(&mut self, receiver: &JsValue, key: &JsValue) -> Result<JsValue, PublicError> {
        self.primitive(ReadCommand::MapGet, receiver, key, 0)
    }
    fn map_get_text(&mut self, receiver: &JsValue, key: &str) -> Result<JsValue, PublicError> {
        self.primitive(ReadCommand::MapGet, receiver, &JsValue::from_str(key), 0)
    }
    fn array_join(&mut self, receiver: &JsValue, separator: &str) -> Result<JsString, PublicError> {
        let result = self.primitive(
            ReadCommand::ArrayJoin,
            receiver,
            &JsValue::from_str(separator),
            0,
        )?;
        if result.is_null() {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        Self::units(&result)?.ok_or_else(|| invalid("Invalid live array join reply").into())
    }
    fn object_identity(&mut self, value: &JsValue) -> Result<Option<u64>, PublicError> {
        let identity =
            self.primitive(ReadCommand::ObjectIdentity, value, &JsValue::UNDEFINED, 0)?;
        if identity.is_undefined() {
            return Ok(None);
        }
        if identity.is_null() {
            return Err(PeleError::new(
                "Public model graph exceeds identity budget",
                ErrorCode::Limit,
            )
            .into());
        }
        let number = identity
            .as_f64()
            .ok_or_else(|| invalid("Invalid live identity reply"))?;
        if !number.is_finite() || number.fract() != 0. || !(1. ..=1_000_000.).contains(&number) {
            return Err(invalid("Invalid live identity reply").into());
        }
        Ok(Some(number as u64))
    }
    fn snapshot_iterable(
        &mut self,
        value: &JsValue,
        max_count: usize,
    ) -> Result<Vec<JsValue>, PublicError> {
        let bound = max_count.min(8_000_000);
        let values = self.primitive(
            ReadCommand::SnapshotIterable,
            value,
            &JsValue::from_f64(bound as f64),
            0,
        )?;
        if values.is_null() {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let array: js_sys::Array = values
            .dyn_into()
            .map_err(|_| invalid("Invalid captured iterable"))?;
        if array.length() as usize > bound {
            return Err(invalid("Invalid captured iterable length").into());
        }
        Ok(array.iter().collect())
    }
    fn is_array(&mut self, value: &JsValue) -> Result<bool, PublicError> {
        self.primitive(ReadCommand::IsArray, value, &JsValue::UNDEFINED, 0)?
            .as_bool()
            .ok_or_else(|| invalid("Invalid live Array classification").into())
    }
    fn own_keys(&mut self, receiver: &JsValue) -> Result<Vec<JsString>, PublicError> {
        let result = self.primitive(ReadCommand::OwnKeys, receiver, &JsValue::UNDEFINED, 0)?;
        if result.is_null() {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let keys: js_sys::Array = result
            .dyn_into()
            .map_err(|_| invalid("Invalid live own-key reply"))?;
        if keys.length() > 1_000_000 {
            return Err(invalid("Invalid live own-key count").into());
        }
        // Verify the complete transfer length before allocating any UTF16 key.
        let mut total = 0usize;
        for value in keys.iter() {
            if !value.is_string() {
                return Err(invalid("Invalid live own-key value").into());
            }
            let text: &js_sys::JsString = value.unchecked_ref();
            total = total.saturating_add(text.length() as usize);
            if total > 16_000_000 {
                return Err(PeleError::new(
                    "Public model graph exceeds work budget",
                    ErrorCode::Limit,
                )
                .into());
            }
        }
        keys.iter()
            .map(|key| {
                Self::units(&key)?.ok_or_else(|| invalid("Invalid live own-key value").into())
            })
            .collect()
    }
    fn snapshot_map_values(
        &mut self,
        receiver: &JsValue,
        max_count: usize,
    ) -> Result<Vec<JsValue>, PublicError> {
        let bound = max_count.min(8_000_000);
        let result = self.primitive(
            ReadCommand::SnapshotMapValues,
            receiver,
            &JsValue::from_f64(bound as f64),
            0,
        )?;
        if result.is_null() {
            return Err(
                PeleError::new("Public model graph exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let values: js_sys::Array = result
            .dyn_into()
            .map_err(|_| invalid("Invalid live Map values reply"))?;
        if values.length() as usize > bound {
            return Err(invalid("Invalid live Map values count").into());
        }
        Ok(values.iter().collect())
    }
    fn kind(&self, value: &JsValue) -> LiveValueKind {
        if value.is_undefined() {
            LiveValueKind::Undefined
        } else if value.is_null() {
            LiveValueKind::Null
        } else if value.as_bool().is_some() {
            LiveValueKind::Boolean
        } else if value.as_f64().is_some() {
            LiveValueKind::Number
        } else if value.is_string() {
            LiveValueKind::String
        } else if value.is_symbol() {
            LiveValueKind::Symbol
        } else if value.is_bigint() {
            LiveValueKind::BigInt
        } else if value.is_function() {
            LiveValueKind::Function
        } else {
            LiveValueKind::Object
        }
    }
    fn number(&self, value: &JsValue) -> Option<f64> {
        value.as_f64()
    }
    fn truthy(&self, value: &JsValue) -> bool {
        value.is_truthy()
    }
    fn string_units(&self, value: &JsValue) -> Result<Option<JsString>, PublicError> {
        Self::units(value)
    }
    fn to_string(&mut self, value: &JsValue) -> Result<JsString, PublicError> {
        let text = self.primitive(ReadCommand::ToString, value, &JsValue::UNDEFINED, 0)?;
        Self::units(&text)?
            .ok_or_else(|| invalid("Live reader returned a non-string conversion").into())
    }
    fn require_string(
        &mut self,
        value: &JsValue,
        site: LiveStringUse,
    ) -> Result<JsString, PublicError> {
        let method = self.key(site.method);
        let context = u32::from(site.expression_receiver == "t");
        let text = self.primitive(ReadCommand::RequireString, value, &method, context)?;
        Self::units(&text)?.ok_or_else(|| invalid("Live reader returned a non-string value").into())
    }
}

#[wasm_bindgen]
pub struct NativeDetachedRenderJob {
    job: Option<DetachedRenderJob>,
    completion: Option<DetachedRenderCompletion>,
}
impl NativeDetachedRenderJob {
    fn new(job: DetachedRenderJob) -> Self {
        Self {
            job: Some(job),
            completion: None,
        }
    }
}
#[wasm_bindgen]
impl NativeDetachedRenderJob {
    #[allow(clippy::too_many_arguments)]
    pub fn run(
        &mut self,
        root: JsValue,
        read: js_sys::Function,
        width: Option<js_sys::Function>,
        icons: Option<js_sys::Function>,
        clock: Option<js_sys::Function>,
        inline_icons: Option<js_sys::Function>,
        dispatcher: Option<js_sys::Function>,
    ) -> Result<(), JsValue> {
        let job = self
            .job
            .take()
            .ok_or_else(|| failure(invalid("Public render job already dispatched")))?;
        let mut reader = Reader {
            root,
            read,
            keys: HashMap::new(),
            key_bytes: 0,
        };
        let mut host = prepared_entries::host_er_raw_resources(width, icons, clock, inline_icons);
        let completion = match dispatcher {
            Some(dispatcher) => job.run(
                &mut reader,
                &mut super::host_functions::Resources {
                    host: &mut host,
                    dispatcher,
                },
            ),
            None => job.run(&mut reader, &mut host),
        };
        self.completion = Some(completion);
        Ok(())
    }
}

enum AckValue {
    Error(String),
    Render {
        value: DetachedRenderValue,
        prepared: render_packet::Prepared,
    },
}
#[wasm_bindgen]
pub struct NativeDetachedRenderAck {
    revision: u64,
    canonical_bytes: usize,
    value: Option<AckValue>,
}
#[wasm_bindgen]
impl NativeDetachedRenderAck {
    #[wasm_bindgen(getter)]
    pub fn revision(&self) -> String {
        self.revision.to_string()
    }
    pub fn materialize(&mut self) -> Result<JsValue, JsValue> {
        let value = self.value.take().ok_or_else(|| {
            failure(invalid(
                "Public render acknowledgement already materialized",
            ))
        })?;
        let (status, payload) = match value {
            AckValue::Error(json) => (0., JsValue::from_str(&json)),
            AckValue::Render { value, prepared } => {
                let result = match value {
                    DetachedRenderValue::Normal(value) => render_packet::emit(
                        false,
                        value.diagram_type.as_str(),
                        &value.rendered,
                        prepared,
                    ),
                    DetachedRenderValue::Raw(value) => {
                        render_packet::emit(true, "", &value, prepared)
                    }
                }?;
                (1., result)
            }
        };
        let packet = js_sys::Array::new();
        packet.push(&JsValue::from_f64(1.));
        packet.push(&JsValue::from_str(&self.revision.to_string()));
        packet.push(&JsValue::from_f64(self.canonical_bytes as f64));
        packet.push(&JsValue::from_f64(status));
        packet.push(&payload);
        Ok(packet.into())
    }
}

#[wasm_bindgen]
impl MutableDocument {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_detached_render_numbers(
        &mut self,
        expected_revision: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
        output_kind: u32,
        output: f64,
    ) -> Result<Option<NativeDetachedRenderJob>, JsValue> {
        if !self.inner.has_detached_public_render() {
            return Ok(None);
        }
        let (options, limit) =
            render_numbers::options(json, mask, numbers, output_kind, output).map_err(failure)?;
        let revision = revision(expected_revision)?;
        self.inner
            .prepare_detached_render(
                revision,
                DetachedRenderRequest::Normal {
                    options,
                    limit,
                    enforce_native_bytes: true,
                },
            )
            .map(|job| job.map(NativeDetachedRenderJob::new))
            .map_err(failure)
    }
    pub fn prepare_detached_render_raw(
        &mut self,
        expected_revision: &str,
        config_json: &str,
        json: &str,
        mask: u32,
        numbers: &[f64],
    ) -> Result<Option<NativeDetachedRenderJob>, JsValue> {
        if !self.inner.has_detached_public_render() {
            return Ok(None);
        }
        let config = prepared_entries::config(config_json).map_err(failure)?;
        let (options, _) = render_numbers::options(json, mask, numbers, 3, 0.).map_err(failure)?;
        let revision = revision(expected_revision)?;
        self.inner
            .prepare_detached_render(
                revision,
                DetachedRenderRequest::Raw {
                    config,
                    settings: options.render,
                },
            )
            .map(|job| job.map(NativeDetachedRenderJob::new))
            .map_err(failure)
    }
    pub fn finish_detached_render(
        &mut self,
        job: &mut NativeDetachedRenderJob,
    ) -> Result<NativeDetachedRenderAck, JsValue> {
        let completion = job
            .completion
            .take()
            .ok_or_else(|| failure(invalid("Public render job has not completed")))?;
        let outcome = self
            .inner
            .finish_detached_render(completion)
            .map_err(failure)?;
        // Numeric browser resources never request native presentation. The
        // rendered Result uses precisely the legacy normal/raw metadata shape.
        let (value, metadata_bytes) = match outcome.result {
            Err(error) => {
                let transport: Result<(), &PublicError> = Err(&error);
                let bytes = render_numbers::encode(&transport).map_err(failure)?;
                let len = bytes.len();
                (
                    AckValue::Error(String::from_utf8(bytes).expect("JSON UTF8")),
                    len,
                )
            }
            Ok(value) => {
                let prepared = match &value {
                    DetachedRenderValue::Normal(value) => {
                        let transport: Result<_, &PublicError> =
                            Ok(pele::compat::render_result_transport(value));
                        render_packet::prepare(&value.rendered, &transport)
                    }
                    DetachedRenderValue::Raw(value) => {
                        let transport: Result<_, &PublicError> =
                            Ok(prepared_entries::transport(value));
                        render_packet::prepare(value, &transport)
                    }
                }
                .map_err(failure)?;
                let len = prepared.metadata_bytes();
                (AckValue::Render { value, prepared }, len)
            }
        };
        let canonical_bytes = outcome
            .graph
            .canonical_binary_size(metadata_bytes, DEFAULT_BINARY_BUDGET)
            .map_err(failure)?;
        Ok(NativeDetachedRenderAck {
            revision: outcome.revision,
            canonical_bytes,
            value: Some(value),
        })
    }
}
