//! Additive owner-bound callbacks for active built-in model methods. No graph copy.
use super::{
    calendar, providers::PeleForeignBytesV10, public, PeleHostResourcesV3, PelePublicDocument,
    PeleRenderNumbersV9, PeleResult,
};
use pele::compat::{
    public::{
        ActiveModel, Atom, HostExceptionToken, HostFunction, JsString, MethodCall, PublicError,
    },
    CalendarFields, CivilDateTime, PeleError, RenderResources, TextRequest, TrustedSvg,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    ptr,
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};
const MAX_BYTES: usize = 1024 * 1024;
const MAX_BINDINGS: usize = 1024;
// Identity allocation only: no callable, pointer, graph or host state is global.
// IDs never repeat, including after close; foreign-owner references cannot alias.
static NEXT_REFERENCE: AtomicU32 = AtomicU32::new(1);
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PeleHostFunctionV12 {
    pub version: u32,
    pub struct_size: u32,
    pub context: *mut c_void,
    pub retain: Option<unsafe extern "C" fn(*mut c_void)>,
    pub release: Option<unsafe extern "C" fn(*mut c_void)>,
    pub invoke: Option<
        unsafe extern "C" fn(
            *mut c_void,
            *mut PeleActiveModelV12,
            *const u8,
            usize,
            *mut PeleHostFunctionReplyV12,
        ),
    >,
    pub reserved: [u64; 2],
}
#[repr(C)]
#[derive(Default)]
pub struct PeleHostFunctionReplyV12 {
    /// 0 = Atom JSON, 1 = PublicError JSON, 2 = nonzero owner exception token.
    pub status: u32,
    pub exception: u32,
    pub payload: PeleForeignBytesV10,
}
struct Reply(PeleHostFunctionReplyV12);
impl Drop for Reply {
    fn drop(&mut self) {
        if let Some(release) = self.0.payload.release {
            unsafe { release(self.0.payload.owner) }
        }
    }
}
struct Function(PeleHostFunctionV12);
impl Drop for Function {
    fn drop(&mut self) {
        unsafe { self.0.release.expect("validated")(self.0.context) }
    }
}
#[derive(Default)]
pub(super) struct Bindings {
    functions: HashMap<u32, Rc<dyn HostFunction>>,
}
struct ActiveBorrow<'a> {
    model: &'a mut dyn ActiveModel,
    resources: &'a mut dyn RenderResources,
    remaining: usize,
}
/// Synchronous borrowed lease. Access only inside its callback; retaining this
/// pointer, concurrent access and access after callback return are caller errors.
pub struct PeleActiveModelV12<'a> {
    active: RefCell<ActiveBorrow<'a>>,
}
impl HostFunction for Function {
    fn call(
        &self,
        model: &mut dyn ActiveModel,
        receiver: Atom,
        arguments: &[Atom],
        resources: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        #[derive(Serialize)]
        struct Call<'a> {
            receiver: Atom,
            arguments: &'a [Atom],
        }
        let args = serde_json::to_vec(&Call {
            receiver,
            arguments,
        })
        .map_err(|_| public::invalid("Host call cannot be encoded"))?;
        if args.len() > MAX_BYTES {
            return Err(public::invalid("Host call exceeds byte budget").into());
        }
        let mut view = PeleActiveModelV12 {
            active: RefCell::new(ActiveBorrow {
                model,
                resources,
                remaining: 1_000_000,
            }),
        };
        let mut reply = Reply(Default::default());
        unsafe {
            self.0.invoke.expect("validated")(
                self.0.context,
                &mut view,
                args.as_ptr(),
                args.len(),
                &mut reply.0,
            )
        };
        let data = unsafe {
            super::compatibility_input(reply.0.payload.data, reply.0.payload.len, MAX_BYTES)
        }?;
        match reply.0.status {
            0 if reply.0.exception == 0 => {
                let atom: Atom = serde_json::from_str(data)
                    .map_err(|_| public::invalid("Host function returned an invalid Atom"))?;
                view.active.get_mut().model.validate(&atom)?;
                Ok(atom)
            }
            1 if reply.0.exception == 0 => Err(serde_json::from_str::<PublicError>(data)
                .map_err(|_| public::invalid("Host function returned an invalid PublicError"))?),
            2 if reply.0.exception != 0 && reply.0.payload.len == 0 => Err(
                PublicError::HostException(HostExceptionToken(reply.0.exception)),
            ),
            _ => Err(public::invalid(
                "Host function returned an invalid status or exception token",
            )
            .into()),
        }
    }
}
struct Resources<'a> {
    host: &'a mut dyn RenderResources,
    bindings: &'a Bindings,
}
impl RenderResources for Resources<'_> {
    fn host_function(&mut self, id: u32) -> Result<Rc<dyn HostFunction>, PublicError> {
        self.bindings.functions.get(&id).cloned().ok_or_else(|| {
            public::invalid("Unbound or foreign-owner host function reference").into()
        })
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
    fn direct_now_millis(&mut self) -> Result<f64, PeleError> {
        self.host.direct_now_millis()
    }
    fn raw_inline_icon(&mut self, s: &str) -> Result<Option<TrustedSvg>, PeleError> {
        self.host.raw_inline_icon(s)
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
    fn presentation_budget(&self) -> Option<pele::compat::PresentationBudget> {
        self.host.presentation_budget()
    }
    fn capture_presentation(&mut self, v: pele::compat::PresentationOutcome) {
        self.host.capture_presentation(v)
    }
}
/// Caller holds the public document operation guard. Reuses its owner bindings
/// for both orchestrated and raw render entry points.
pub(super) unsafe fn with_resources<T>(
    handle: *const PelePublicDocument,
    resources: *const PeleHostResourcesV3,
    run: impl FnOnce(&mut dyn RenderResources) -> Result<T, PeleError>,
) -> Result<T, PeleError> {
    let owner =
        unsafe { handle.as_ref() }.ok_or_else(|| public::invalid("Null public document"))?;
    let bindings = owner
        .host_functions
        .try_borrow()
        .map_err(|_| public::invalid("Host function bindings are already in use"))?;
    unsafe {
        calendar::with_resources(resources, |host| {
            run(&mut Resources {
                host,
                bindings: &bindings,
            })
        })
    }
}
/// Bind a callable without modifying graph or revision. On success the document
/// retains context once until close, even when its graph no longer references it.
/// # Safety
/// The document/table/output and callback pointers must remain valid as specified
/// in pele.h. Call on its owner thread; no concurrent close or callback unwind.
#[no_mangle]
pub unsafe extern "C" fn pele_public_bind_host_function_v12(
    handle: *mut PelePublicDocument,
    table: *const PeleHostFunctionV12,
    out_reference: *mut u32,
) -> *mut PeleResult {
    if !out_reference.is_null() {
        unsafe { *out_reference = 0 }
    };
    public::bounded_call(|| {
        if out_reference.is_null() || table.is_null() {
            return Err(public::invalid("Null host function table or output"));
        }
        let version = unsafe { ptr::addr_of!((*table).version).read_unaligned() };
        let size = unsafe { ptr::addr_of!((*table).struct_size).read_unaligned() };
        if version != 12 || size < (std::mem::size_of::<PeleHostFunctionV12>() as u32) {
            return Err(public::invalid("Invalid host function version or size"));
        }
        let table = unsafe { table.read_unaligned() };
        if table.reserved != [0; 2]
            || table.retain.is_none()
            || table.release.is_none()
            || table.invoke.is_none()
        {
            return Err(public::invalid(
                "Invalid host function callbacks or reserved fields",
            ));
        }
        let _document = unsafe { public::document(handle) }?;
        let owner = unsafe { &*handle };
        let mut bindings = owner
            .host_functions
            .try_borrow_mut()
            .map_err(|_| public::invalid("Host function bindings are already in use"))?;
        if bindings.functions.len() >= MAX_BINDINGS {
            return Err(PeleError::new(
                "Host function binding budget exceeded",
                pele::compat::ErrorCode::Limit,
            ));
        }
        let id = NEXT_REFERENCE
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| {
                PeleError::new(
                    "Host function identity space exhausted",
                    pele::compat::ErrorCode::Limit,
                )
            })?;
        let result = public::success(serde_json::json!({"reference":id}))?;
        unsafe { table.retain.expect("validated")(table.context) };
        bindings.functions.insert(id, Rc::new(Function(table)));
        unsafe { *out_reference = id };
        Ok(result)
    })
}
/// Same v9 numeric/output/host policies, with document-owned callable dispatch.
/// # Safety
/// Same borrowed pointer/owner requirements as the v9 operation.
#[no_mangle]
pub unsafe extern "C" fn pele_public_render_operation_with_functions_v12(
    handle: *mut PelePublicDocument,
    revision: u64,
    bytes: *const u8,
    len: usize,
    numbers: *const PeleRenderNumbersV9,
    kind: u32,
    bits: u64,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    unsafe {
        super::pele_public_render_operation_with_numbers_v9(
            handle, revision, bytes, len, numbers, kind, bits, resources,
        )
    }
}

#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "camelCase", deny_unknown_fields)]
enum Access {
    Root,
    Read {
        receiver: Atom,
        key: String,
    },
    Set {
        receiver: Atom,
        key: String,
        value: Atom,
    },
    Intern {
        units: JsString,
    },
    String {
        value: Atom,
    },
    Object,
    Array,
    Date {
        value: Atom,
    },
    DateMillis {
        value: Atom,
    },
    Push {
        receiver: Atom,
        value: Atom,
    },
    InvokeBuiltin {
        call: MethodCall,
    },
}
#[derive(Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
enum Value {
    Atom(Atom),
    String(JsString),
}
/// Execute one bounded primitive on the callback's active arena. Success envelope
/// contains Result<Value,PublicError>; partial writes survive an operation error.
/// # Safety
/// View is valid only inside its owning callback. Inputs are synchronous borrows.
#[no_mangle]
pub unsafe extern "C" fn pele_active_model_access_v12(
    view: *mut PeleActiveModelV12,
    bytes: *const u8,
    len: usize,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let input = unsafe { super::compatibility_input(bytes, len, MAX_BYTES) }?;
        let access: Access = serde_json::from_str(input)
            .map_err(|_| public::invalid("Invalid active model operation"))?;
        let view =
            unsafe { view.as_ref() }.ok_or_else(|| public::invalid("Null active model view"))?;
        let mut active = view
            .active
            .try_borrow_mut()
            .map_err(|_| public::invalid("Active model view is already in use"))?;
        active.remaining = active.remaining.checked_sub(1).ok_or_else(|| {
            PeleError::new(
                "Active model operation budget exceeded",
                pele::compat::ErrorCode::Limit,
            )
        })?;
        let ActiveBorrow {
            model, resources, ..
        } = &mut *active;
        let result: Result<Value, PublicError> = (|| {
            Ok(Value::Atom(match access {
                Access::Root => model.root(),
                Access::Read { receiver, key } => model.read(&receiver, &key)?,
                Access::Set {
                    receiver,
                    key,
                    value,
                } => {
                    model.set(&receiver, &key, value)?;
                    Atom::Undefined
                }
                Access::Intern { units } => model.intern(units)?,
                Access::String { value } => return model.string(&value).map(Value::String),
                Access::Object => model.object()?,
                Access::Array => model.array()?,
                Access::Date {
                    value: Atom::Number(v),
                } => model.date(v)?,
                Access::Date { .. } => {
                    return Err(public::invalid("Active Date requires a numeric Atom").into())
                }
                Access::DateMillis { value } => Atom::Number(model.date_millis(&value)?),
                Access::Push { receiver, value } => {
                    model.push(&receiver, value)?;
                    Atom::Undefined
                }
                Access::InvokeBuiltin { call } => model.invoke_builtin(call, *resources)?,
            }))
        })();
        public::success(result)
    })
}

/// Invoke a captured builtin with the document's retained caller functions.
/// # Safety
/// Same synchronous owner-thread pointer requirements as the v12 render call.
#[no_mangle]
pub unsafe extern "C" fn pele_public_invoke_with_functions_v12(
    handle: *mut PelePublicDocument,
    revision: u64,
    bytes: *const u8,
    len: usize,
    resources: *const PeleHostResourcesV3,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let json = unsafe { super::compatibility_input(bytes, len, 64 * 1024 * 1024) }?;
        let call = serde_json::from_str(json)
            .map_err(|_| public::invalid("Invalid public model method call"))?;
        let mut document = unsafe { public::document(handle) }?;
        let outcome = unsafe {
            with_resources(handle, resources, |host| {
                document.invoke_read(revision, call, host)
            })
        }?;
        public::binary_metadata(outcome.revision, &outcome.graph, &outcome.result)
    })
}
