//! Lossless explicit host values and short-borrow constructor method jobs.
use super::{
    compatibility_input, public, PelePublicConstructorV16, PelePublicDocument, PeleResult,
};
use pele::compat::{
    public::{
        DetachedInvocationJob, DetachedInvocationStep, HostExceptionToken, JsString, PublicError,
        PublicHostOperation, PublicHostResult, PublicHostValue,
    },
    ErrorCode, PeleError,
};
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
    sync::Mutex,
};

/// UTF16 is borrowed only during a call. Numbers carry all IEEE bits; opaque
/// identities belong to the caller's document-local retained host store.
#[repr(C)]
pub struct PeleHostValueV17 {
    pub version: u32,
    pub struct_size: u32,
    pub tag: u32,
    pub reserved: u32,
    pub payload: u64,
    pub units: *const u16,
    pub unit_count: usize,
}
pub(super) unsafe fn value(input: *const PeleHostValueV17) -> Result<PublicHostValue, PeleError> {
    let Some(v) = (unsafe { input.as_ref() }) else {
        return Ok(PublicHostValue::Undefined);
    };
    if v.version != 17
        || v.struct_size as usize != std::mem::size_of::<PeleHostValueV17>()
        || v.reserved != 0
    {
        return Err(public::invalid("Invalid v17 host value"));
    }
    Ok(match v.tag {
        0 => PublicHostValue::Undefined,
        1 => PublicHostValue::Null,
        2 if v.payload <= 1 => PublicHostValue::Bool(v.payload != 0),
        3 => PublicHostValue::Number(f64::from_bits(v.payload)),
        4 => {
            if v.unit_count > 16_000_000 {
                return Err(PeleError::new(
                    "Public model graph exceeds work budget",
                    ErrorCode::Limit,
                ));
            }
            if v.units.is_null() && v.unit_count != 0 {
                return Err(public::invalid("Null host UTF16 input"));
            }
            let units = if v.unit_count == 0 {
                Vec::new()
            } else {
                unsafe { std::slice::from_raw_parts(v.units, v.unit_count) }.to_vec()
            };
            PublicHostValue::String(JsString(units))
        }
        5 => PublicHostValue::Opaque(
            u32::try_from(v.payload).map_err(|_| public::invalid("Invalid host identity"))?,
        ),
        _ => return Err(public::invalid("Invalid host value tag")),
    })
}
/// # Safety
/// Live constructor, readable value/UTF16 for this call, writable output. Ignored
/// factories do not read the argument. No host code runs during construction.
#[no_mangle]
pub unsafe extern "C" fn pele_public_constructor_construct_with_v17(
    handle: *const PelePublicConstructorV16,
    argument: *const PeleHostValueV17,
    out: *mut *mut PelePublicDocument,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() };
    }
    public::bounded_call(|| {
        let out = unsafe { out.as_mut() }
            .ok_or_else(|| public::invalid("Null constructor document output"))?;
        let factory =
            unsafe { handle.as_ref() }.ok_or_else(|| public::invalid("Null public constructor"))?;
        let argument = if factory.constructor.ignores_arguments() {
            PublicHostValue::Undefined
        } else {
            unsafe { value(argument) }?
        };
        let document = factory.constructor.construct_with(argument)?;
        let result = public::success(
            serde_json::json!({"type":document.diagram_type(),"revision":document.revision().to_string(),"constructorCreated":document.is_constructor_created()}),
        )?;
        *out = public::owned_document(document);
        Ok(result)
    })
}
pub struct PeleInvocationJobV17 {
    owner: *const PelePublicDocument,
    inner: Mutex<DetachedInvocationJob>,
}
/// # Safety
/// Document remains live until the job is freed. JSON readable <=64MiB. The job
/// owns no document borrow. Each accepted resume may mutate and advance revision.
#[no_mangle]
pub unsafe extern "C" fn pele_public_invocation_prepare_v17(
    owner: *mut PelePublicDocument,
    revision: u64,
    bytes: *const u8,
    len: usize,
    out: *mut *mut PeleInvocationJobV17,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() };
    }
    public::bounded_call(|| {
        let out =
            unsafe { out.as_mut() }.ok_or_else(|| public::invalid("Null invocation job output"))?;
        let input = unsafe { compatibility_input(bytes, len, 64 * 1024 * 1024) }?;
        let call = serde_json::from_str(input)
            .map_err(|_| public::invalid("Invalid public model method call"))?;
        let mut document = unsafe { public::document(owner) }?;
        let Some(job) = document.prepare_detached_invocation(revision, call)? else {
            return public::success(serde_json::json!({"selected":false}));
        };
        let result = public::success(serde_json::json!({"selected":true}))?;
        *out = Box::into_raw(Box::new(PeleInvocationJobV17 {
            owner,
            inner: Mutex::new(job),
        }));
        Ok(result)
    })
}
pub(super) fn operation(op: PublicHostOperation) -> serde_json::Value {
    use serde_json::json;
    match op {
        PublicHostOperation::Get { receiver, key } => {
            json!({"kind":"get","receiver":receiver,"key":key})
        }
        PublicHostOperation::Call {
            function,
            receiver,
            arguments,
            not_callable_error,
            result,
        } => {
            json!({"kind":"call","function":function,"receiver":receiver,"arguments":arguments,"notCallableError":not_callable_error,"result":match result{PublicHostResult::Discard=>"discard",PublicHostResult::Retain=>"retain",PublicHostResult::Value=>"value"}})
        }
        PublicHostOperation::Truthy { value } => json!({"kind":"truthy","value":value}),
        PublicHostOperation::ToString { value } => json!({"kind":"toString","value":value}),
        PublicHostOperation::GreaterEqual { left, right } => {
            json!({"kind":"greaterEqual","left":left,"right":right})
        }
        PublicHostOperation::StrictEqual { left, right } => {
            json!({"kind":"strictEqual","left":left,"right":right})
        }
    }
}
/// # Safety
/// Owner/job live and paired, reply readable. reply_state 0=start,1=value,2=host
/// exception token in reply.payload. out_revision is always written after native
/// dispatch, including post-dispatch export failure. No callbacks execute here.
#[no_mangle]
pub unsafe extern "C" fn pele_public_invocation_resume_v17(
    owner: *mut PelePublicDocument,
    job: *mut PeleInvocationJobV17,
    reply_state: u32,
    reply: *const PeleHostValueV17,
    out_revision: *mut u64,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let acknowledged = unsafe { out_revision.as_mut() }
            .ok_or_else(|| public::invalid("Null invocation revision output"))?;
        let job = unsafe { job.as_ref() }.ok_or_else(|| public::invalid("Null invocation job"))?;
        if job.owner != owner {
            return Err(public::invalid(
                "Public invocation belongs to another document",
            ));
        }
        let mut job = job
            .inner
            .try_lock()
            .map_err(|_| public::invalid("Invocation job is already in use"))?;
        let reply = unsafe { host_reply(reply_state, reply) }?;
        let mut document = unsafe { public::document(owner) }?;
        let result = (|| {
            let step = document.resume_detached_invocation(&mut job, reply)?;
            let metadata = match step {
                DetachedInvocationStep::Complete => serde_json::json!({"complete":true}),
                DetachedInvocationStep::Host(op) => serde_json::json!({"host":operation(op)}),
            };
            let graph = document.snapshot_read()?;
            public::binary_metadata(document.revision(), &graph, &metadata)
        })();
        *acknowledged = document.revision();
        result
    })
}
/// # Safety
/// Same paired ownership as resume. Finish a completed job once; it consumes its
/// reservation, publishes the current graph, and does not invoke host code.
#[no_mangle]
pub unsafe extern "C" fn pele_public_invocation_finish_v17(
    owner: *mut PelePublicDocument,
    job: *mut PeleInvocationJobV17,
    out_revision: *mut u64,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let acknowledged = unsafe { out_revision.as_mut() }
            .ok_or_else(|| public::invalid("Null invocation revision output"))?;
        let job = unsafe { job.as_ref() }.ok_or_else(|| public::invalid("Null invocation job"))?;
        if job.owner != owner {
            return Err(public::invalid(
                "Public invocation belongs to another document",
            ));
        }
        let mut job = job
            .inner
            .try_lock()
            .map_err(|_| public::invalid("Invocation job is already in use"))?;
        let mut document = unsafe { public::document(owner) }?;
        let result = (|| {
            let outcome = document.finish_detached_invocation(&mut job)?;
            public::binary_metadata(outcome.revision, &outcome.graph, &outcome.result)
        })();
        *acknowledged = document.revision();
        result
    })
}
/// # Safety
/// Free exactly once after its calls return. Dropping unfinished jobs preserves
/// already dispatched partial mutations at their last acknowledged revision.
#[no_mangle]
pub unsafe extern "C" fn pele_public_invocation_free_v17(job: *mut PeleInvocationJobV17) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !job.is_null() {
            drop(unsafe { Box::from_raw(job) });
        }
    }));
}
/// Exact existing JavaScript number formatter; no Swift/C reimplementation.
#[no_mangle]
pub extern "C" fn pele_host_number_string_v17(bits: u64) -> *mut PeleResult {
    public::bounded_call(|| {
        public::success(JsString::from(
            pele::compat::public::live_number_string(f64::from_bits(bits)).as_str(),
        ))
    })
}

/// # Safety
/// The captured factory must remain live for this call.
#[no_mangle]
pub unsafe extern "C" fn pele_public_constructor_policy_v17(
    handle: *const PelePublicConstructorV16,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let factory =
            unsafe { handle.as_ref() }.ok_or_else(|| public::invalid("Null public constructor"))?;
        public::success(
            serde_json::json!({"ignoresArguments":factory.constructor.ignores_arguments()}),
        )
    })
}

fn js_space(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000d | 0x0020 | 0x00a0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff)
}
fn string_number(units: &[u16]) -> f64 {
    let start = units
        .iter()
        .position(|u| !js_space(*u))
        .unwrap_or(units.len());
    let end = units
        .iter()
        .rposition(|u| !js_space(*u))
        .map_or(start, |i| i + 1);
    let units = &units[start..end];
    if units.is_empty() {
        return 0.;
    }
    // Numeric syntax is ASCII; lone UTF16 and all other Unicode are NaN.
    if units.iter().any(|u| *u > 127) {
        return f64::NAN;
    }
    let bytes: Vec<u8> = units.iter().map(|u| *u as u8).collect();
    if matches!(bytes.as_slice(), b"Infinity" | b"+Infinity") {
        return f64::INFINITY;
    }
    if bytes == b"-Infinity" {
        return f64::NEG_INFINITY;
    }
    if bytes.len() >= 2 && bytes[0] == b'0' {
        let width = match bytes[1] {
            b'x' | b'X' => 4,
            b'o' | b'O' => 3,
            b'b' | b'B' => 1,
            _ => 0,
        };
        if width != 0 {
            if bytes.len() == 2 {
                return f64::NAN;
            }
            // Power-of-two radix conversion rounds once, ties to even, rather
            // than accumulating rounded f64 products or truncating to u64.
            let mut significant = 0usize;
            let mut top = 0u64;
            let mut guard = false;
            let mut sticky = false;
            for byte in &bytes[2..] {
                let digit = match byte {
                    b'0'..=b'9' => byte - b'0',
                    b'a'..=b'f' => byte - b'a' + 10,
                    b'A'..=b'F' => byte - b'A' + 10,
                    _ => return f64::NAN,
                };
                if u32::from(digit) >= 1 << width {
                    return f64::NAN;
                }
                for shift in (0..width).rev() {
                    let bit = digit & (1 << shift) != 0;
                    if significant == 0 && !bit {
                        continue;
                    }
                    significant += 1;
                    if significant <= 53 {
                        top = (top << 1) | u64::from(bit);
                    } else if significant == 54 {
                        guard = bit;
                    } else {
                        sticky |= bit;
                    }
                }
            }
            if significant <= 53 {
                return top as f64;
            }
            if significant > 1024 {
                return f64::INFINITY;
            }
            if guard && (sticky || top & 1 != 0) {
                top += 1;
            }
            return top as f64 * 2f64.powi((significant - 53) as i32);
        }
    }
    let mut i = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    let mut digits = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
        digits += 1;
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
            digits += 1;
        }
    }
    if digits == 0 {
        return f64::NAN;
    }
    if matches!(bytes.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(bytes.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return f64::NAN;
        }
    }
    if i != bytes.len() {
        return f64::NAN;
    }
    std::str::from_utf8(&bytes)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(f64::NAN)
}
/// Scalar ToNumber after the host's explicit ToPrimitive. No object reads or
/// graph import; the returned Atom number preserves IEEE bits in existing JSON.
/// # Safety
/// Readable v17 value and UTF16 for this call.
#[no_mangle]
pub unsafe extern "C" fn pele_host_primitive_number_v17(
    input: *const PeleHostValueV17,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let number = match unsafe { value(input) }? {
            PublicHostValue::Undefined => f64::NAN,
            PublicHostValue::Null => 0.,
            PublicHostValue::Bool(v) => u8::from(v) as f64,
            PublicHostValue::Number(v) => v,
            PublicHostValue::String(v) => string_number(&v.0),
            PublicHostValue::Opaque(_) => {
                return Err(public::invalid("Expected primitive host value"))
            }
        };
        public::success(pele::compat::public::Atom::Number(number))
    })
}

/// Allocates an identity in the same never-reused space as v12 bound functions.
#[no_mangle]
pub extern "C" fn pele_host_identity_v17() -> *mut PeleResult {
    public::bounded_call(|| public::success(super::host_functions::allocate_reference()?))
}

pub(super) unsafe fn host_reply(
    reply_state: u32,
    reply: *const PeleHostValueV17,
) -> Result<Option<Result<PublicHostValue, PublicError>>, PeleError> {
    Ok(match reply_state {
        0 => None,
        1 => Some(Ok(unsafe { value(reply) }?)),
        2 => {
            let PublicHostValue::Opaque(token) = unsafe { value(reply) }? else {
                return Err(public::invalid("Invalid host exception token"));
            };
            Some(Err(PublicError::HostException(HostExceptionToken(token))))
        }
        3 => {
            let PublicHostValue::String(message) = (unsafe { value(reply) })? else {
                return Err(public::invalid("Invalid host TypeError message"));
            };
            Some(Err(PublicError::JavaScript {
                name: pele::compat::public::JavaScriptErrorKind::TypeError,
                message,
            }))
        }
        _ => return Err(public::invalid("Invalid host reply state")),
    })
}
