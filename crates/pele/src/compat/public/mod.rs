//! Mutable host documents augment the immutable typed Rust API.
//! Public codecs are opt-in; an unimplemented codec never silently ignores edits.
mod engine;
mod graph;
pub(crate) mod captured_read;
pub(crate) mod captured_write;
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

mod binary;
pub use binary::{
    decode_binary_exchange, encode_binary_exchange, BinaryExchange, DEFAULT_BINARY_BUDGET,
};

pub(crate) use engine::create_prepared_edge_public;
