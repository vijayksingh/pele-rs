//! Unpublished owned constructor bodies. Host operations run between calls.
use super::{
    compatibility_input,
    constructor_host::{host_reply, operation, value, PeleHostValueV17},
    public, PelePublicConstructorV16, PelePublicDocument, PeleResult,
};
use pele::compat::public::{
    Atom, ConstructorArgumentTransfer, DetachedInvocationStep, GraphReadView, MemberKind,
    ModelSchemaView, Node, PrototypeParent, PublicConstructionJob, PublicHostValue,
};
use std::{
    borrow::Cow,
    panic::{catch_unwind, AssertUnwindSafe},
    ptr,
    sync::{Mutex, MutexGuard},
};

pub struct PeleConstructionJobV18 {
    inner: Mutex<PublicConstructionJob>,
    schema: ModelSchemaView,
}
unsafe fn job<'a>(
    handle: *mut PeleConstructionJobV18,
) -> Result<MutexGuard<'a, PublicConstructionJob>, pele::compat::PeleError> {
    unsafe { handle.as_ref() }
        .ok_or_else(|| public::invalid("Null construction job"))?
        .inner
        .try_lock()
        .map_err(|_| public::invalid("Constructor job is already in use"))
}
/// # Safety
/// Live factory, optional v17 argument pointers; writable job output. Only the
/// captured native factory's declared first two arguments are inspected.
#[no_mangle]
pub unsafe extern "C" fn pele_public_constructor_begin_v18(
    factory: *const PelePublicConstructorV16,
    first: *const PeleHostValueV17,
    second: *const PeleHostValueV17,
    out: *mut *mut PeleConstructionJobV18,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() };
    }
    public::bounded_call(|| {
        let out = unsafe { out.as_mut() }
            .ok_or_else(|| public::invalid("Null construction job output"))?;
        let factory = unsafe { factory.as_ref() }
            .ok_or_else(|| public::invalid("Null public constructor"))?;
        let count = factory.constructor.argument_count();
        let args = [
            if count > 0 {
                unsafe { value(first) }?
            } else {
                PublicHostValue::Undefined
            },
            if count > 1 {
                unsafe { value(second) }?
            } else {
                PublicHostValue::Undefined
            },
        ];
        let inner = factory.constructor.begin_construct_with_arguments(args)?;
        let schema = factory.constructor.schema_view();
        let response = public::success(
            serde_json::json!({"revision":inner.revision()?.to_string(),"constructorCreated":true}),
        )?;
        *out = Box::into_raw(Box::new(PeleConstructionJobV18 {
            inner: Mutex::new(inner),
            schema,
        }));
        Ok(response)
    })
}
/// # Safety
/// Live captured factory; no host operation is executed.
#[no_mangle]
pub unsafe extern "C" fn pele_public_constructor_policy_v18(
    factory: *const PelePublicConstructorV16,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let factory = unsafe { factory.as_ref() }
            .ok_or_else(|| public::invalid("Null public constructor"))?;
        public::success(
            serde_json::json!({"argumentCount":factory.constructor.argument_count(),"arity":factory.constructor.arity(),"argumentTransfers":factory.constructor.argument_transfers().iter().map(|transfer| match transfer { ConstructorArgumentTransfer::Retain => "retain", ConstructorArgumentTransfer::Value => "value" }).collect::<Vec<_>>()}),
        )
    })
}
/// # Safety
/// Job and output pointer live. Same reply vocabulary as C17. The job is not
/// borrowed while the host handles the exported operation. Latest revision is
/// acknowledged after every dispatched phase even when export fails.
#[no_mangle]
pub unsafe extern "C" fn pele_public_construction_resume_v18(
    handle: *mut PeleConstructionJobV18,
    state: u32,
    reply: *const PeleHostValueV17,
    out_revision: *mut u64,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let out = unsafe { out_revision.as_mut() }
            .ok_or_else(|| public::invalid("Null construction revision output"))?;
        let reply = unsafe { host_reply(state, reply) }?;
        let mut job = unsafe { job(handle) }?;
        let result = (|| {
            let step = job.resume(reply)?;
            let metadata = match step {
                DetachedInvocationStep::Complete => serde_json::json!({"complete":true}),
                DetachedInvocationStep::Host(op) => serde_json::json!({"host":operation(op)}),
            };
            let graph = job.snapshot()?;
            public::binary_metadata(
                job.revision()?,
                &GraphReadView::Arena(Cow::Borrowed(&graph)),
                &metadata,
            )
        })();
        *out = job.revision()?;
        result
    })
}
/// # Safety
/// Job is live and not concurrently freed. Snapshot grants no constructor success.
#[no_mangle]
pub unsafe extern "C" fn pele_public_construction_snapshot_v18(
    handle: *mut PeleConstructionJobV18,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let job = unsafe { job(handle) }?;
        let graph = job.snapshot()?;
        public::binary_metadata(
            job.revision()?,
            &GraphReadView::Arena(Cow::Borrowed(&graph)),
            &serde_json::json!({}),
        )
    })
}
/// # Safety
/// Live job and readable canonical binary input, at most 64 MiB.
#[no_mangle]
pub unsafe extern "C" fn pele_public_construction_apply_v18(
    handle: *mut PeleConstructionJobV18,
    bytes: *const u8,
    len: usize,
) -> *mut PeleResult {
    public::bounded_call(|| {
        if len > 64 * 1024 * 1024 {
            return Err(pele::compat::PeleError::new(
                "Public binary graph exceeds transport budget",
                pele::compat::ErrorCode::Limit,
            ));
        }
        if bytes.is_null() && len != 0 {
            return Err(public::invalid("Null public binary graph"));
        }
        let bytes = if len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(bytes, len) }
        };
        let exchange = pele::compat::public::decode_binary_exchange(bytes, 64 * 1024 * 1024)?;
        let mut job = unsafe { job(handle) }?;
        job.apply(exchange.revision, exchange.graph)?;
        public::success(serde_json::json!({"revision":job.revision()?.to_string()}))
    })
}
/// # Safety
/// Live job and readable MethodCall JSON. Invokes the captured native builtin,
/// with default native resources only; no host callable dispatch or callbacks.
#[no_mangle]
pub unsafe extern "C" fn pele_public_construction_invoke_v18(
    handle: *mut PeleConstructionJobV18,
    revision: u64,
    bytes: *const u8,
    len: usize,
    out_revision: *mut u64,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let out = unsafe { out_revision.as_mut() }
            .ok_or_else(|| public::invalid("Null construction revision output"))?;
        let call =
            serde_json::from_str(unsafe { compatibility_input(bytes, len, 64 * 1024 * 1024) }?)
                .map_err(|_| public::invalid("Invalid public model method call"))?;
        let mut job = unsafe { job(handle) }?;
        let result = (|| {
            let outcome = unsafe {
                super::calendar::with_resources(ptr::null(), |host| {
                    job.invoke(revision, call, host)
                })
            }?;
            public::binary_metadata(
                outcome.revision,
                &GraphReadView::Arena(Cow::Borrowed(&outcome.graph)),
                &outcome.result,
            )
        })();
        *out = job.revision()?;
        result
    })
}
/// Read an ordinary property or inherited native method from the provisional
/// graph. No JavaScript getter is synthesized. Host-valued own fields stay opaque.
/// # Safety
/// Live job and readable UTF8 key up to 4096 bytes.
#[no_mangle]
pub unsafe extern "C" fn pele_public_construction_get_v18(
    handle: *mut PeleConstructionJobV18,
    receiver: u32,
    key: *const u8,
    len: usize,
) -> *mut PeleResult {
    public::bounded_call(|| {
        let key = unsafe { compatibility_input(key, len, 4096) }?;
        let owner =
            unsafe { handle.as_ref() }.ok_or_else(|| public::invalid("Null construction job"))?;
        let job = unsafe { job(handle) }?;
        let graph = job.snapshot()?;
        let node = graph
            .nodes
            .get(receiver as usize)
            .ok_or_else(|| public::invalid("Unknown construction receiver"))?;
        if let Some(property) = node
            .properties()
            .iter()
            .find(|p| graph.strings[p.key as usize].is(key))
        {
            return public::success(property.value.clone());
        }
        let mut prototype = if let Node::Instance { prototype, .. } = node {
            Some(*prototype)
        } else {
            None
        };
        for _ in 0..owner.schema.model.prototypes.len() {
            let Some(id) = prototype else { break };
            let Some(p) = owner.schema.model.prototypes.iter().find(|p| p.id == id) else {
                break;
            };
            if let Some(member) = p.members.iter().find(|m| m.name == key) {
                return match member.member {
                    MemberKind::Method { method } => {
                        public::success(Atom::BuiltinFunction(method.0))
                    }
                    MemberKind::Accessor { .. } => Err(public::invalid(
                        "Native constructor accessor requires an explicit host adapter",
                    )),
                };
            }
            prototype = if let PrototypeParent::Prototype(id) = p.parent {
                Some(id)
            } else {
                None
            };
        }
        public::success(Atom::Undefined)
    })
}
/// # Safety
/// Writable document/revision outputs. Once complete, transfers the partial owner
/// even on body or export error. Caller MUST free every non-null returned owner.
/// The binary result contains Result<(),PublicError>; success and failure are
/// distinct from the existence of the retained receiver.
#[no_mangle]
pub unsafe extern "C" fn pele_public_construction_finish_v18(
    handle: *mut PeleConstructionJobV18,
    out: *mut *mut PelePublicDocument,
    out_revision: *mut u64,
) -> *mut PeleResult {
    if !out.is_null() {
        unsafe { *out = ptr::null_mut() };
    }
    public::bounded_call(|| {
        let out = unsafe { out.as_mut() }
            .ok_or_else(|| public::invalid("Null construction document output"))?;
        let revision = unsafe { out_revision.as_mut() }
            .ok_or_else(|| public::invalid("Null construction revision output"))?;
        let mut job = unsafe { job(handle) }?;
        let outcome = job.finish_retaining_receiver()?;
        *revision = outcome.document.revision();
        let result = (|| {
            let graph = outcome.document.snapshot_read()?;
            public::binary_metadata(*revision, &graph, &outcome.result)
        })();
        *out = public::owned_document(outcome.document);
        result
    })
}
/// # Safety
/// Free one owned job once, after its calls return. An unfinished receiver drops.
#[no_mangle]
pub unsafe extern "C" fn pele_public_construction_free_v18(handle: *mut PeleConstructionJobV18) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if !handle.is_null() {
            drop(unsafe { Box::from_raw(handle) });
        }
    }));
}
