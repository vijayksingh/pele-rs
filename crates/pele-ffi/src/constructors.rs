//! Captured native constructors retain registration independently of registry lifetime.
use super::{compatibility_input, public, registry, PelePublicDocument, PeleRegistry, PeleResult};
use pele::compat::{
    public::{CapturedPublicConstructor, PrototypeId},
    DiagramId,
};
use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
};

pub struct PelePublicConstructorV16 {
    pub(super) constructor: CapturedPublicConstructor,
}
/// # Safety
/// Registry is live, id is readable UTF-8 up to 4096 bytes, and out is writable.
#[no_mangle]
pub unsafe extern "C" fn pele_registry_capture_constructor_v16(
    handle: *const PeleRegistry,
    id: *const u8,
    len: usize,
    prototype: u32,
    out: *mut *mut PelePublicConstructorV16,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() };
    }
    public::bounded_call(|| {
        let out = unsafe { out.as_mut() }
            .ok_or_else(|| public::invalid("Null public constructor output"))?;
        let name = unsafe { compatibility_input(id, len, 4096) }?;
        let engine = unsafe { registry::registry(handle) }?;
        // A host provider overlay cannot inherit a builtin capability by name.
        if registry::builtin(name)
            .is_some_and(|kind| unsafe { &*handle }.providers.get(kind).is_some())
        {
            return public::success(serde_json::json!({"selected":false}));
        }
        let id =
            registry::builtin(name).map_or_else(|| DiagramId::extension(name), DiagramId::from);
        let Some(constructor) = engine.capture_constructor(&id, PrototypeId(prototype))? else {
            return public::success(serde_json::json!({"selected":false}));
        };
        let result = public::success(serde_json::json!({"selected":true}))?;
        *out = Box::into_raw(Box::new(PelePublicConstructorV16 { constructor }));
        Ok(result)
    })
}
/// # Safety
/// Constructor is live, out writable, no concurrent free. No callbacks run.
/// Returned document uses ordinary v4 ownership.
#[no_mangle]
pub unsafe extern "C" fn pele_public_constructor_construct_v16(
    handle: *const PelePublicConstructorV16,
    out: *mut *mut PelePublicDocument,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() };
    }
    public::bounded_call(|| {
        let out = unsafe { out.as_mut() }
            .ok_or_else(|| public::invalid("Null constructor document output"))?;
        let constructor =
            unsafe { handle.as_ref() }.ok_or_else(|| public::invalid("Null public constructor"))?;
        let document = constructor.constructor.construct()?;
        let result = public::success(serde_json::json!({
            "type":document.diagram_type(), "revision":document.revision().to_string(),
            "constructorCreated":document.is_constructor_created(),
        }))?;
        *out = public::owned_document(document);
        Ok(result)
    })
}
/// # Safety
/// Free null or one owned constructor once, after its calls return.
#[no_mangle]
pub unsafe extern "C" fn pele_public_constructor_free_v16(handle: *mut PelePublicConstructorV16) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !handle.is_null() {
            drop(unsafe { Box::from_raw(handle) });
        }
    }));
}
