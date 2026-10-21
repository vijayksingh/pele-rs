//! Owned evaluation of a native constructor body before document publication.
use super::{
    DetachedInvocationJob, DetachedInvocationStep, DetachedPublicInvocation, InvocationOutcome,
    MethodCall, ModelGraph, PublicDocument, PublicError, PublicHostValue,
};
use crate::compat::{ErrorCode, PeleError, RenderResources};

/// A captured factory creates ordinary defaults, then Rust requests precisely
/// the host operations required by its constructor body. No state borrow crosses
/// a getter/callback. Failed bodies never publish a successful document.
#[doc(hidden)]
pub struct PublicConstructionJob {
    document: Option<PublicDocument>,
    invocation: Option<DetachedInvocationJob>,
    complete: bool,
}
#[doc(hidden)]
pub struct PublicConstructionOutcome {
    pub document: PublicDocument,
    pub result: Result<(), PublicError>,
}
impl PublicConstructionJob {
    pub(super) fn new(
        mut document: PublicDocument,
        program: Option<DetachedPublicInvocation>,
    ) -> Result<Self, PeleError> {
        let invocation = program
            .map(|program| document.prepare_constructor_body(program))
            .transpose()?;
        Ok(Self {
            document: Some(document),
            complete: invocation.is_none(),
            invocation,
        })
    }
    fn live(&self) -> Result<&PublicDocument, PeleError> {
        self.document
            .as_ref()
            .ok_or_else(|| PeleError::new("Constructor job is closed", ErrorCode::Semantic))
    }
    /// A provisional graph for the host's allocated receiver and requested values;
    /// this grants no parsed-document provenance or public constructor success.
    pub fn error_context(&self) -> Option<&super::PublicHostErrorContext> {
        self.invocation
            .as_ref()
            .and_then(DetachedInvocationJob::error_context)
    }
    pub fn snapshot(&self) -> Result<ModelGraph, PeleError> {
        self.live()?.snapshot()
    }
    pub fn revision(&self) -> Result<u64, PeleError> {
        Ok(self.live()?.revision())
    }
    pub fn apply(&mut self, expected_revision: u64, graph: ModelGraph) -> Result<(), PeleError> {
        self.live()?;
        self.document
            .as_mut()
            .expect("checked live")
            .apply(expected_revision, graph)
    }
    pub fn invoke(
        &mut self,
        expected_revision: u64,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<InvocationOutcome, PeleError> {
        self.live()?;
        self.document
            .as_mut()
            .expect("checked live")
            .invoke(expected_revision, call, host)
    }
    pub fn resume(
        &mut self,
        reply: Option<Result<PublicHostValue, PublicError>>,
    ) -> Result<DetachedInvocationStep, PeleError> {
        self.live()?;
        if self.complete {
            if reply.is_some() {
                return Err(PeleError::new(
                    "Constructor body is complete",
                    ErrorCode::Semantic,
                ));
            }
            return Ok(DetachedInvocationStep::Complete);
        }
        let result = self
            .document
            .as_mut()
            .expect("checked live")
            .resume_detached_invocation(
                self.invocation.as_mut().expect("active constructor body"),
                reply,
            )?;
        self.complete = matches!(result, DetachedInvocationStep::Complete);
        Ok(result)
    }
    pub fn finish(&mut self) -> Result<Result<PublicDocument, PublicError>, PeleError> {
        let outcome = self.finish_retaining_receiver()?;
        Ok(outcome.result.map(|()| outcome.document))
    }
    /// JavaScript virtual constructor callbacks can expose their receiver before
    /// throwing. Keep that partial owner available without claiming construction
    /// success or parsed provenance; ordinary finish drops it on failure.
    pub fn finish_retaining_receiver(&mut self) -> Result<PublicConstructionOutcome, PeleError> {
        self.live()?;
        if !self.complete {
            return Err(PeleError::new(
                "Constructor body has not completed",
                ErrorCode::Semantic,
            ));
        }
        let result = if let Some(invocation) = self.invocation.as_mut() {
            self.document
                .as_mut()
                .expect("checked live")
                .finish_detached_invocation(invocation)?
                .result
                .map(|_| ())
        } else {
            Ok(())
        };
        self.invocation.take();
        self.document
            .as_mut()
            .expect("checked live")
            .finish_constructor_body();
        Ok(PublicConstructionOutcome {
            document: self.document.take().expect("checked live"),
            result,
        })
    }
}
