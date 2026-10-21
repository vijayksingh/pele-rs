//! Owned builtin method continuations. A host request never carries a graph borrow.
use super::{
    detached_render::{RenderBook, RenderLease},
    Atom, PublicError, PublicHostValue,
};
use crate::compat::{ErrorCode, PeleError, RenderResources};
use std::{
    any::{Any, TypeId},
    sync::{Arc, Mutex},
};

/// Result transfer is selected by Rust. Retain forwards an untouched host value;
/// Value imports an owned scalar; Discard never inspects the returned value.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicHostResult {
    Discard,
    Retain,
    Value,
}
/// A fixed host expression whose engine-generated TypeError is observable.
/// No expression text is executed or supplied by the caller.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PublicHostErrorExpression {
    ThisOptionsWarn,
    OptionalWarnStyle,
    WarnProducer,
    Producer,
    WarnReferences,
    WarnDuration,
}
#[doc(hidden)]
#[derive(Clone, Debug, serde::Serialize)]
pub struct PublicHostErrorContext {
    pub expression: PublicHostErrorExpression,
    pub value: Option<Atom>,
}
/// A primitive requested at an exact point in a native method. Hosts execute no
/// diagram decisions; values retain their document-local graph/foreign identity.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum PublicHostOperation {
    Truthy {
        value: Atom,
    },
    StrictEqual {
        left: Atom,
        right: Atom,
    },
    GreaterEqual {
        left: Atom,
        right: Atom,
    },
    ToString {
        value: Atom,
    },
    Get {
        receiver: Atom,
        key: &'static str,
    },
    Call {
        function: Atom,
        receiver: Atom,
        arguments: Vec<Atom>,
        not_callable_error: &'static str,
        result: PublicHostResult,
    },
}
/// One owned native resource computation. Only builtin code can mint it;
/// execution consumes it and never borrows the document or its graph. Adapters
/// retain their normal callback error/identity handling around the resource set.
#[doc(hidden)]
pub struct PublicResourceOperation {
    run: Box<dyn FnOnce(&mut dyn RenderResources) -> Result<PublicHostValue, PublicError> + Send>,
}
impl PublicResourceOperation {
    pub(crate) fn new(
        run: impl FnOnce(&mut dyn RenderResources) -> Result<PublicHostValue, PublicError>
            + Send
            + 'static,
    ) -> Self {
        Self { run: Box::new(run) }
    }
    /// Run after releasing the short document borrow. Pass its result unchanged
    /// to the existing invocation resume operation; a failure terminates there.
    pub fn run(self, resources: &mut dyn RenderResources) -> Result<PublicHostValue, PublicError> {
        (self.run)(resources)
    }
}
#[doc(hidden)]
pub enum DetachedInvocationStep {
    Host(PublicHostOperation),
    Resource(PublicResourceOperation),
    Complete,
}
#[doc(hidden)]
pub enum PublicInvocationPhase {
    Host(PublicHostOperation),
    HostWithContext {
        operation: PublicHostOperation,
        expression: PublicHostErrorExpression,
    },
    FailedWithContext {
        error: PublicError,
        expression: PublicHostErrorExpression,
        value: Atom,
    },
    Resource(PublicResourceOperation),
    Complete(Result<Atom, PublicError>),
}
type Resume =
    Box<dyn FnMut(&mut (dyn Any + Send), Option<PublicHostValue>) -> PublicInvocationPhase + Send>;
/// Only builtin modules can mint a phase program. Forwarded discovery does not
/// authorize the forwarding provider to operate on another creator's state.
#[doc(hidden)]
pub struct DetachedPublicInvocation {
    creator: TypeId,
    resume: Resume,
}
impl DetachedPublicInvocation {
    pub(crate) fn new<D: 'static, S: Send + 'static, T: Send + 'static>(
        mut phase: T,
        resume: fn(&mut S, &mut T, Option<PublicHostValue>) -> PublicInvocationPhase,
    ) -> Self {
        Self {
            creator: TypeId::of::<D>(),
            resume: Box::new(move |state, reply| {
                resume(
                    state
                        .downcast_mut::<S>()
                        .expect("creator-paired invocation state"),
                    &mut phase,
                    reply,
                )
            }),
        }
    }
    pub(super) fn for_creator_type(self, creator: TypeId) -> Option<Self> {
        (self.creator == creator).then_some(self)
    }
    pub(super) fn for_creator<D: 'static>(self) -> Option<Self> {
        (self.creator == TypeId::of::<D>()).then_some(self)
    }
}
#[doc(hidden)]
pub struct DetachedInvocationJob {
    lease: RenderLease,
    program: DetachedPublicInvocation,
    awaiting_reply: bool,
    error_context: Option<PublicHostErrorContext>,
    result: Option<Result<Atom, PublicError>>,
}
impl DetachedInvocationJob {
    /// Metadata for the just-completed step. A reply clears it before advancing.
    pub fn error_context(&self) -> Option<&PublicHostErrorContext> {
        self.error_context.as_ref()
    }

    pub(super) fn new(
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
        program: DetachedPublicInvocation,
    ) -> Result<Self, PeleError> {
        Ok(Self {
            lease: RenderLease::reserve(owner, revision)?,
            program,
            awaiting_reply: false,
            error_context: None,
            result: None,
        })
    }
    pub(super) fn check_resume(
        &self,
        owner: &Arc<Mutex<RenderBook>>,
        has_reply: bool,
    ) -> Result<(), PeleError> {
        self.lease.check_owner(owner)?;
        if self.result.is_some() || self.awaiting_reply != has_reply {
            return Err(PeleError::new(
                "Invalid public invocation phase",
                ErrorCode::Semantic,
            ));
        }
        Ok(())
    }
    pub(super) fn advance(
        &mut self,
        owner: &Arc<Mutex<RenderBook>>,
        state: &mut (dyn Any + Send),
        reply: Option<Result<PublicHostValue, PublicError>>,
    ) -> Result<DetachedInvocationStep, PeleError> {
        self.lease.check_owner(owner)?;
        if self.result.is_some() || self.awaiting_reply != reply.is_some() {
            return Err(PeleError::new(
                "Invalid public invocation phase",
                ErrorCode::Semantic,
            ));
        }
        let phase = match reply {
            Some(Err(error)) => PublicInvocationPhase::Complete(Err(error)),
            other => (self.program.resume)(state, other.map(Result::unwrap)),
        };
        self.error_context = None;
        match phase {
            PublicInvocationPhase::HostWithContext {
                operation,
                expression,
            } => {
                self.error_context = Some(PublicHostErrorContext {
                    expression,
                    value: None,
                });
                self.awaiting_reply = true;
                Ok(DetachedInvocationStep::Host(operation))
            }
            PublicInvocationPhase::FailedWithContext {
                error,
                expression,
                value,
            } => {
                self.error_context = Some(PublicHostErrorContext {
                    expression,
                    value: Some(value),
                });
                self.awaiting_reply = false;
                self.result = Some(Err(error));
                Ok(DetachedInvocationStep::Complete)
            }
            PublicInvocationPhase::Host(operation) => {
                self.awaiting_reply = true;
                Ok(DetachedInvocationStep::Host(operation))
            }
            PublicInvocationPhase::Resource(operation) => {
                self.awaiting_reply = true;
                Ok(DetachedInvocationStep::Resource(operation))
            }
            PublicInvocationPhase::Complete(result) => {
                self.awaiting_reply = false;
                self.result = Some(result);
                Ok(DetachedInvocationStep::Complete)
            }
        }
    }
    pub(super) fn finish(
        &mut self,
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
    ) -> Result<(u64, Result<Atom, PublicError>), PeleError> {
        self.lease.check_owner(owner)?;
        if self.result.is_none() {
            return Err(PeleError::new(
                "Public invocation has not completed",
                ErrorCode::Semantic,
            ));
        }
        self.lease.release(owner)?;
        Ok((revision, self.result.take().expect("checked result")))
    }
}
