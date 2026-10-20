//! Sealed, detached builtin rendering. No document borrow survives a host call.
use super::{engine::Registration, GraphReadView, PublicError};
use crate::compat::{
    diagrams::sankey::{render_detached, Sankey, SankeyDetachedInput},
    ErrorCode, Object, OutputLimit, PeleError, PresentationCapture, PresentationOutcome,
    RenderOptions, RenderResources, RenderResult, RenderSettings, Rendered,
};
use std::{
    any::TypeId,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

pub use crate::compat::diagrams::sankey::{LiveRenderRead, LiveStringUse, LiveValueKind};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Sankey,
}
/// Metadata-only capability. Only this crate can mint a concrete creator token.
#[doc(hidden)]
pub struct DetachedRenderCapability {
    creator: TypeId,
    kind: Kind,
}
impl DetachedRenderCapability {
    pub(crate) fn sankey() -> Self {
        Self {
            creator: TypeId::of::<Sankey>(),
            kind: Kind::Sankey,
        }
    }
    pub(super) fn for_creator<D: 'static>(self) -> Option<Self> {
        (self.creator == TypeId::of::<D>()).then_some(self)
    }
}
/// Owned builtin input. Public providers cannot construct or replace its body.
#[doc(hidden)]
pub struct DetachedPublicRender {
    creator: TypeId,
    body: Body,
}
enum Body {
    Sankey(SankeyDetachedInput),
}
impl DetachedPublicRender {
    pub(crate) fn sankey(input: SankeyDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<Sankey>(),
            body: Body::Sankey(input),
        }
    }
    pub(super) fn for_creator<D: 'static>(
        self,
        capability: &DetachedRenderCapability,
    ) -> Option<Self> {
        let kind = match &self.body {
            Body::Sankey(_) => Kind::Sankey,
        };
        (self.creator == TypeId::of::<D>()
            && self.creator == capability.creator
            && kind == capability.kind)
            .then_some(self)
    }
    fn render<R: LiveRenderRead>(
        &self,
        config: &Object,
        settings: &RenderSettings,
        read: &mut R,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        match &self.body {
            Body::Sankey(input) => render_detached(input, config, settings, read, resources),
        }
    }
}

/// Already-decoded options. Raw config is the caller's explicit low-level config.
#[doc(hidden)]
pub enum DetachedRenderRequest {
    Normal {
        options: RenderOptions,
        limit: Option<OutputLimit>,
        enforce_native_bytes: bool,
    },
    Raw {
        config: Object,
        settings: RenderSettings,
    },
}
#[doc(hidden)]
pub enum DetachedRenderValue {
    Normal(RenderResult),
    Raw(Rendered),
}

pub(super) struct RenderBook {
    next_frame: u64,
    outstanding: u64,
    frames: Vec<Frame>,
}
struct Frame {
    id: u64,
    active: bool,
}
impl RenderBook {
    pub(super) fn new() -> Self {
        Self {
            next_frame: 0,
            outstanding: 0,
            frames: Vec::new(),
        }
    }
    fn trim(&mut self) {
        while self.frames.last().is_some_and(|frame| !frame.active) {
            self.frames.pop();
        }
    }
    pub(super) fn active(&self) -> bool {
        self.outstanding != 0
    }
}
fn book_lock(book: &Mutex<RenderBook>) -> MutexGuard<'_, RenderBook> {
    // No user callback executes under this lock. Recover only bookkeeping if a
    // native unwind interrupted a caller; never invent a document mutation.
    book.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn exhausted() -> PeleError {
    PeleError::new("Public model revision exhausted", ErrorCode::Limit)
}
pub(super) fn next_revision(
    revision: u64,
    book: Option<&Arc<Mutex<RenderBook>>>,
) -> Result<u64, PeleError> {
    let reserved = book.map_or(0, |book| book_lock(book).outstanding);
    revision
        .checked_add(reserved)
        .and_then(|v| v.checked_add(1))
        .ok_or_else(exhausted)?;
    Ok(revision + 1)
}
pub(super) fn has_active(book: Option<&Arc<Mutex<RenderBook>>>) -> bool {
    book.is_some_and(|book| book_lock(book).active())
}

pub(super) struct RenderLease {
    owner: Weak<Mutex<RenderBook>>,
    id: u64,
    active: bool,
}
impl RenderLease {
    pub(super) fn reserve(
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
    ) -> Result<Self, PeleError> {
        let mut book = book_lock(owner);
        revision
            .checked_add(book.outstanding)
            .and_then(|v| v.checked_add(1))
            .ok_or_else(exhausted)?;
        let id = book.next_frame.checked_add(1).ok_or_else(|| {
            PeleError::new("Public render frame identity exhausted", ErrorCode::Limit)
        })?;
        book.frames.push(Frame { id, active: true });
        book.next_frame = id;
        book.outstanding += 1;
        Ok(Self {
            owner: Arc::downgrade(owner),
            id,
            active: true,
        })
    }
    pub(super) fn check_owner(&self, expected: &Arc<Mutex<RenderBook>>) -> Result<(), PeleError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(|| invalid("Public render document is disposed"))?;
        if !Arc::ptr_eq(&owner, expected) {
            return Err(invalid("Public render job belongs to another document"));
        }
        self.check_top()
    }
    fn check_top(&self) -> Result<(), PeleError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(|| invalid("Public render document is disposed"))?;
        let book = book_lock(&owner);
        if !self.active
            || !book
                .frames
                .last()
                .is_some_and(|frame| frame.active && frame.id == self.id)
        {
            return Err(invalid("Public render jobs must finish in nesting order"));
        }
        Ok(())
    }
    pub(super) fn finish(
        &mut self,
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
    ) -> Result<u64, PeleError> {
        self.check_owner(owner)?;
        let next = revision.checked_add(1).ok_or_else(exhausted)?;
        self.release(owner)?;
        Ok(next)
    }
    pub(super) fn release(
        &mut self,
        expected_owner: &Arc<Mutex<RenderBook>>,
    ) -> Result<(), PeleError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(|| invalid("Public render document is disposed"))?;
        if !Arc::ptr_eq(&owner, expected_owner) {
            return Err(invalid("Public render job belongs to another document"));
        }
        let mut book = book_lock(&owner);
        if !self.active
            || !book
                .frames
                .last()
                .is_some_and(|frame| frame.active && frame.id == self.id)
        {
            return Err(invalid("Public render jobs must finish in nesting order"));
        }
        book.frames.pop();
        book.outstanding -= 1;
        book.trim();
        self.active = false;
        Ok(())
    }
}
impl Drop for RenderLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Some(owner) = self.owner.upgrade() {
            let mut book = book_lock(&owner);
            if let Some(frame) = book
                .frames
                .iter_mut()
                .find(|frame| frame.id == self.id && frame.active)
            {
                frame.active = false;
                book.outstanding -= 1;
                book.trim();
            }
        }
    }
}

/// An owned render stack frame. It never holds PublicDocument, its graph or state.
#[doc(hidden)]
pub struct DetachedRenderJob {
    lease: RenderLease,
    registration: Arc<Registration>,
    config: Object,
    request: DetachedRenderRequest,
    input: DetachedPublicRender,
}
/// The same one-use frame after execution, including dispatched operation errors.
#[doc(hidden)]
pub struct DetachedRenderCompletion {
    lease: RenderLease,
    result: Result<DetachedRenderValue, PublicError>,
    presentation: Option<PresentationOutcome>,
}
/// Only created after resource execution. Drop its graph borrow before any host
/// presentation or result-factory callback. It is not a reconciliation request.
#[doc(hidden)]
pub struct DetachedRenderOutcome<'a> {
    pub revision: u64,
    pub graph: GraphReadView<'a>,
    pub result: Result<DetachedRenderValue, PublicError>,
    pub presentation: Option<PresentationOutcome>,
}
impl DetachedRenderJob {
    pub(super) fn new(
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
        registration: Arc<Registration>,
        config: Object,
        request: DetachedRenderRequest,
        input: DetachedPublicRender,
    ) -> Result<Self, PeleError> {
        Ok(Self {
            lease: RenderLease::reserve(owner, revision)?,
            registration,
            config,
            request,
            input,
        })
    }
    /// Consume the job without borrowing its document. Original JavaScript
    /// exceptions are retained by the caller's per-job channel, not this value.
    pub fn run<R: LiveRenderRead>(
        self,
        read: &mut R,
        resources: &mut dyn RenderResources,
    ) -> DetachedRenderCompletion {
        let Self {
            lease,
            registration,
            config,
            request,
            input,
        } = self;
        if let Err(error) = lease.check_top() {
            return DetachedRenderCompletion {
                lease,
                result: Err(error.into()),
                presentation: None,
            };
        }
        let mut operation_error = None;
        let (result, presentation) = match request {
            DetachedRenderRequest::Normal {
                options,
                limit,
                enforce_native_bytes,
            } => {
                let mut presentation = PresentationCapture::deferred(resources);
                let result = crate::compat::engine::render_registered_policy(
                    &registration.descriptor,
                    &config,
                    &options,
                    &mut presentation,
                    limit,
                    enforce_native_bytes,
                    |settings, host| {
                        input
                            .render(&config, settings, read, host)
                            .map_err(|error| {
                                operation_error = Some(error);
                                invalid("Public render operation failed")
                            })
                    },
                )
                .map(DetachedRenderValue::Normal)
                .map_err(|error| operation_error.unwrap_or_else(|| error.into()));
                (result, presentation.take())
            }
            DetachedRenderRequest::Raw { config, settings } => {
                let result = crate::compat::engine::render_raw_registered(
                    &registration.descriptor,
                    &settings,
                    resources,
                    |host| {
                        input
                            .render(&config, &settings, read, host)
                            .map_err(|error| {
                                operation_error = Some(error);
                                invalid("Public raw render operation failed")
                            })
                    },
                )
                .map(DetachedRenderValue::Raw)
                .map_err(|error| operation_error.unwrap_or_else(|| error.into()));
                (result, None)
            }
        };
        DetachedRenderCompletion {
            lease,
            result,
            presentation,
        }
    }
}
impl DetachedRenderCompletion {
    pub(super) fn finish(
        mut self,
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
    ) -> Result<
        (
            u64,
            Result<DetachedRenderValue, PublicError>,
            Option<PresentationOutcome>,
        ),
        PeleError,
    > {
        let next = self.lease.finish(owner, revision)?;
        Ok((next, self.result, self.presentation))
    }
}

/// Native live readers share the canonical JavaScript number formatter.
#[doc(hidden)]
pub fn live_number_string(value: f64) -> String {
    crate::compat::svg::js_num(value)
}
