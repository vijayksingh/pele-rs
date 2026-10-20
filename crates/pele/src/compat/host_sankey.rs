//! Hidden captured Sankey owner boundary. No host values or native graph mirror.
pub use super::public::{
    captured_admission::{AdmissionKind, AdmissionRead, Extent, Header as AdmissionHeader},
    captured_read::{
        CapturedRead, NodeHeader, NodeKind, PropertyPrefix, PROPERTY_CHUNK, STRING_CHUNK,
    },
    captured_transport::{admit_observed, admit_outbound, canonical_frame_bytes},
    captured_write::{CapturedWrite, GraphCounts, MapMatch, OwnQuery},
    InitialWriter,
};
use super::{
    diagrams::sankey::{self, Sankey, SankeyModel},
    public::{
        captured_admission, Atom, CapturedSankeyRegistration, MethodCall, ModelSchemaView,
        PublicDocument, PublicError,
    },
    DiagramModule, ErrorCode, Object, OutputLimit, PeleError, RenderOptions, RenderResources,
    RenderResult, RenderSettings, Rendered,
};

pub enum Parsed<T> {
    Captured { document: Document, storage: T },
    Arena(PublicDocument),
}
pub struct Outcome<T> {
    pub revision: u64,
    pub result: Result<T, PublicError>,
}
/// Storage remains consumer-owned; only captured policy and the eager render
/// projection survive native calls. Supply the same accepted storage epoch until
/// apply or dispatched invocation replaces it. Tokens do not waive admission.
pub struct Document {
    registration: CapturedSankeyRegistration,
    config: Object,
    revision: u64,
    projection: Result<SankeyModel, PeleError>,
}
impl Document {
    pub(crate) fn captured(
        registration: CapturedSankeyRegistration,
        config: Object,
        projection: Result<SankeyModel, PeleError>,
    ) -> Self {
        Self {
            registration,
            config,
            revision: 0,
            projection,
        }
    }
    pub fn diagram_type(&self) -> &super::DiagramId {
        &self.registration.descriptor().id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn config(&self) -> &Object {
        &self.config
    }
    pub fn schema(&self) -> ModelSchemaView {
        self.registration.schema()
    }
    fn next(&self, expected: u64) -> Result<u64, PeleError> {
        if expected != self.revision {
            return Err(PeleError::new(
                "Stale public model revision",
                ErrorCode::Semantic,
            ));
        }
        self.revision
            .checked_add(1)
            .ok_or_else(|| PeleError::new("Public model revision exhausted", ErrorCode::Limit))
    }
    pub fn snapshot_admission<S: AdmissionRead>(&self, store: &mut S) -> Result<(), PeleError> {
        self.registration.validate(store)
    }
    pub fn apply<S: CapturedRead + AdmissionRead>(
        &mut self,
        expected: u64,
        observed: &mut S,
    ) -> Result<(), PeleError> {
        let next = self.next(expected)?;
        self.registration.validate(observed)?;
        // Keep the concrete codec's apply_public structural gate as well.
        captured_admission::validate(observed)?;
        let projection = sankey::imported_from(observed);
        self.projection = projection;
        self.revision = next;
        Ok(())
    }
    pub fn invoke<S: CapturedWrite + AdmissionRead>(
        &mut self,
        expected: u64,
        working: &mut S,
        call: MethodCall,
    ) -> Result<Outcome<Atom>, PeleError> {
        let next = self.next(expected)?;
        self.registration.validate_call(working, &call)?;
        let mut result = sankey::invoke_store(working, call);
        self.projection = sankey::imported_from(working);
        self.revision = next;
        self.snapshot_admission(working)?;
        if let Ok(value) = &result {
            if let Err(error) = self.registration.validate_return(working, value) {
                result = Err(error.into());
            }
        }
        Ok(Outcome {
            revision: next,
            result,
        })
    }
    pub fn render<S: AdmissionRead>(
        &mut self,
        expected: u64,
        accepted: &mut S,
        options: &RenderOptions,
        host: &mut dyn RenderResources,
        limit: Option<OutputLimit>,
        enforce_native_bytes: bool,
    ) -> Result<Outcome<RenderResult>, PeleError> {
        let next = self.next(expected)?;
        self.snapshot_admission(accepted)?;
        let mut presentation = super::PresentationCapture::deferred(host);
        let result = super::engine::render_registered_policy(
            self.registration.descriptor(),
            &self.config,
            options,
            &mut presentation,
            limit,
            enforce_native_bytes,
            |settings, session| {
                Sankey.render(
                    self.projection.as_ref().map_err(Clone::clone)?,
                    &self.config,
                    settings,
                    session,
                )
            },
        )
        .map_err(Into::into);
        self.revision = next;
        self.snapshot_admission(accepted)?;
        presentation.publish();
        Ok(Outcome {
            revision: next,
            result,
        })
    }
    pub fn render_raw<S: AdmissionRead>(
        &mut self,
        expected: u64,
        accepted: &mut S,
        config: &Object,
        settings: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Outcome<Rendered>, PeleError> {
        let next = self.next(expected)?;
        self.snapshot_admission(accepted)?;
        let result = super::engine::render_raw_registered(
            self.registration.descriptor(),
            settings,
            host,
            |session| {
                Sankey.render(
                    self.projection.as_ref().map_err(Clone::clone)?,
                    config,
                    settings,
                    session,
                )
            },
        )
        .map_err(Into::into);
        self.revision = next;
        self.snapshot_admission(accepted)?;
        Ok(Outcome {
            revision: next,
            result,
        })
    }
}
