//! Owner-bound opaque values. No document/binding borrow crosses foreign code.
use super::{public, PeleHostValueV17, PelePublicDocument, PeleResult};
use pele::compat::{
    public::{Atom, HostExceptionToken, JavaScriptErrorKind, PublicError, PublicHostValue},
    ErrorCode, PeleError,
};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    ptr,
    rc::Rc,
};

#[repr(C)]
pub struct PeleHostRequestV20 {
    /// Get, Call, Truthy, ToString, GreaterEqual, StrictEqual, ToNumber.
    pub operation: u32,
    /// Value=0, Retain=1, Discard=2. Only Call uses this policy.
    pub result: u32,
    pub values: *const PeleHostValueV17,
    pub count: usize,
    pub key: *const u16,
    pub key_count: usize,
    pub error: *const u16,
    pub error_count: usize,
}
#[repr(C)]
pub struct PeleHostReplyV20 {
    /// Unhandled=0, Value=1, original exception=2, TypeError text=3.
    pub status: u32,
    pub exception: u32,
    pub value: PeleHostValueV17,
    pub owner: *mut c_void,
    pub release: Option<unsafe extern "C" fn(*mut c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PeleHostAccessV20 {
    pub version: u32,
    pub struct_size: u32,
    pub context: *mut c_void,
    pub retain: Option<unsafe extern "C" fn(*mut c_void)>,
    pub release: Option<unsafe extern "C" fn(*mut c_void)>,
    pub invoke:
        Option<unsafe extern "C" fn(*mut c_void, *const PeleHostRequestV20, *mut PeleHostReplyV20)>,
    pub reserved: [u64; 2],
}
struct Table(PeleHostAccessV20);
impl Drop for Table {
    fn drop(&mut self) {
        unsafe { self.0.release.expect("validated")(self.0.context) }
    }
}
struct Reply(PeleHostReplyV20);
impl Drop for Reply {
    fn drop(&mut self) {
        if let Some(release) = self.0.release {
            unsafe { release(self.0.owner) }
        }
    }
}
#[derive(Default)]
pub(super) struct Access {
    table: RefCell<Option<Rc<Table>>>,
    depth: Cell<usize>,
}
struct Depth<'a>(&'a Cell<usize>);
impl Drop for Depth<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1)
    }
}
pub(super) fn packet(tag: u32, payload: u64, units: &[u16]) -> PeleHostValueV17 {
    PeleHostValueV17 {
        version: 17,
        struct_size: std::mem::size_of::<PeleHostValueV17>() as u32,
        tag,
        reserved: 0,
        payload,
        units: units.as_ptr(),
        unit_count: units.len(),
    }
}
fn limit() -> PeleError {
    PeleError::new("Opaque host access exceeds work budget", ErrorCode::Limit)
}
/// # Safety
/// Live owner and readable table. Owner-thread only; no free during an operation.
/// Retain/release must manage context only and must not unwind. Null clears the
/// optional dispatcher. Requests/replies are synchronous borrowed leases.
#[no_mangle]
pub unsafe extern "C" fn pele_public_set_host_access_v20(
    owner: *mut PelePublicDocument,
    table: *const PeleHostAccessV20,
) -> *mut PeleResult {
    public::bounded_call(|| {
        {
            let _document = unsafe { public::document(owner) }?;
        }
        let next = if table.is_null() {
            None
        } else {
            let version = unsafe { ptr::addr_of!((*table).version).read_unaligned() };
            let size = unsafe { ptr::addr_of!((*table).struct_size).read_unaligned() };
            if version != 20 || size as usize != std::mem::size_of::<PeleHostAccessV20>() {
                return Err(public::invalid("Invalid host access table"));
            }
            let table = unsafe { table.read_unaligned() };
            if table.reserved != [0; 2]
                || table.retain.is_none()
                || table.release.is_none()
                || table.invoke.is_none()
            {
                return Err(public::invalid("Invalid host access callbacks"));
            }
            unsafe { table.retain.expect("validated")(table.context) };
            Some(Rc::new(Table(table)))
        };
        let access = &unsafe { &*owner }.host_values;
        let previous = {
            let mut slot = access
                .table
                .try_borrow_mut()
                .map_err(|_| public::invalid("Host access table is already in use"))?;
            std::mem::replace(&mut *slot, next)
        };
        drop(previous);
        public::success(serde_json::json!({"installed":!table.is_null()}))
    })
}
pub(super) fn dispatch(
    owner: *const PelePublicDocument,
    request: &PeleHostRequestV20,
) -> Result<Option<PublicHostValue>, PublicError> {
    let owner = unsafe { owner.as_ref() }.ok_or_else(|| public::invalid("Null public owner"))?;
    let access = &owner.host_values;
    let table = access
        .table
        .try_borrow()
        .map_err(|_| public::invalid("Host access table is already in use"))?
        .clone();
    let Some(table) = table else { return Ok(None) };
    if access.depth.get() >= 64 || request.count > 1_000_002 {
        return Err(limit().into());
    }
    let values = if request.count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(request.values, request.count) }
    };
    let units = values.iter().fold(
        request.key_count.saturating_add(request.error_count),
        |n, v| n.saturating_add(v.unit_count),
    );
    if units > 16_000_000 {
        return Err(limit().into());
    }
    access.depth.set(access.depth.get() + 1);
    let _depth = Depth(&access.depth);
    let mut reply = Reply(PeleHostReplyV20 {
        status: 0,
        exception: 0,
        value: packet(0, 0, &[]),
        owner: ptr::null_mut(),
        release: None,
    });
    unsafe { table.0.invoke.expect("validated")(table.0.context, request, &mut reply.0) };
    if reply.0.status == 0 {
        return Ok(None);
    }
    if reply.0.status == 2 && reply.0.exception != 0 {
        return Err(PublicError::HostException(HostExceptionToken(
            reply.0.exception,
        )));
    }
    if !matches!(reply.0.status, 1 | 3) {
        return Err(public::invalid("Invalid opaque host reply status").into());
    }
    if request.operation == 1 && request.result == 2 && reply.0.status == 1 {
        return Ok(Some(PublicHostValue::Undefined));
    }
    let p = &reply.0.value;
    if request.operation == 1 && request.result == 1 && reply.0.status == 1 && p.tag != 5 {
        return Err(public::invalid("Retained host reply requires an opaque identity").into());
    }
    let value = if matches!(p.tag, 6 | 7) {
        if p.version != 17
            || p.struct_size as usize != std::mem::size_of::<PeleHostValueV17>()
            || p.reserved != 0
            || p.unit_count != 0
        {
            return Err(public::invalid("Invalid graph-coordinate reply").into());
        }
        let id = u32::try_from(p.payload)
            .map_err(|_| public::invalid("Invalid graph-coordinate reply"))?;
        let value = if p.tag == 6 {
            Atom::Ref(id)
        } else {
            Atom::BuiltinFunction(id)
        };
        if p.tag == 6 {
            let document = unsafe { public::document(owner) }?;
            document.snapshot_read()?.reader().header(id as usize)?;
        }
        PublicHostValue::Graph(value)
    } else {
        unsafe { super::constructor_host::value(p) }?
    };
    if reply.0.status == 3 {
        let PublicHostValue::String(message) = value else {
            return Err(public::invalid("Invalid TypeError reply").into());
        };
        return Err(PublicError::JavaScript {
            name: JavaScriptErrorKind::TypeError,
            message,
        });
    }
    Ok(Some(value))
}
