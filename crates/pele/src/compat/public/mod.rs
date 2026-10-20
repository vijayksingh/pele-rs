//! Mutable host documents augment the immutable typed Rust API.
//! Public codecs are opt-in; an unimplemented codec never silently ignores edits.
pub(crate) mod captured_admission;
pub(crate) mod captured_read;
pub(crate) mod captured_write;
mod engine;
mod graph;
pub(crate) mod graph_read;
#[doc(hidden)]
pub use graph_read::{GraphReadView, PublicGraphRead, TrustedGraphRead};
#[doc(hidden)]
pub use captured_admission::{Extent as GraphExtent, Header as GraphHeader, AdmissionKind as GraphNodeKind};
mod initial_store;
#[doc(hidden)]
pub use initial_store::InitialWriter;
mod host_function;
pub use host_function::{ActiveModel, HostFunction, ModelDelta};
mod schema;
#[cfg(feature = "web-host-er")]
#[doc(hidden)]
pub use engine::HostErEntrySelection;
pub use engine::{
    ReadOutcome,
    ParsedPublicModel, PublicDiagramModule, PublicDocument, PublicEngine, RawRenderOutcome,
    RawRenderOutcomeView, RenderOutcomeView,
};
pub use graph::{Atom, ErrorObjectKind, GraphWriter, JsString, ModelGraph, Node, Property};

pub use schema::{
    FieldKind, FieldSchema, FunctionShape, HostExceptionToken, InvocationOutcome,
    JavaScriptErrorKind, MemberKind, MemberSchema, MethodCall, MethodId, MethodSchema, ModelSchema,
    ModelSchemaView, PrototypeId, PrototypeParent, PrototypeSchema, PublicError, PublicPeleError,
    RecordId, RecordKind, RecordSchema, RenderOutcome, SharedFunction, OPAQUE_MODEL_SCHEMA,
};

pub(crate) mod binary;
pub use binary::{
    decode_binary_exchange, encode_binary_exchange, BinaryExchange, DEFAULT_BINARY_BUDGET,
};

pub(crate) use engine::create_prepared_edge_public;

pub(crate) mod captured_transport;
#[cfg(feature = "web-host-sankey")]
pub(crate) use engine::CapturedSankeyRegistration;
#[cfg(feature = "web-host-sankey")]
#[doc(hidden)]
pub use engine::HostSankeyEntrySelection;

#[cfg(feature = "web-host-sankey")]
pub(crate) mod flat_store;
