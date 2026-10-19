//! Small native descriptor tables for public model records and real prototypes.
//! Descriptors declare exchange behavior, never infer fields from serde output.
use super::{Atom, JsString, ModelGraph};
use crate::compat::{ErrorCode, PeleError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
macro_rules! identifier {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u32);
    };
}
identifier!(RecordId);
identifier!(PrototypeId);
identifier!(MethodId);
// Opaque identity in the calling host session; never a native pointer.
identifier!(HostExceptionToken);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordKind {
    Object,
    Array,
    Map,
    Set,
    Date,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "record", rename_all = "camelCase")]
pub enum FieldKind {
    Any,
    Bool,
    Number,
    String,
    Object,
    Array,
    Map,
    Set,
    Date,
    Function,
    Record(RecordId),
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct FieldSchema {
    pub name: &'static str,
    pub kind: FieldKind,
    /// Absence, explicit undefined and null are separate observations.
    pub optional: bool,
    pub undefined: bool,
    pub nullable: bool,
    /// Enumerable=1, writable=2, configurable=4.
    pub flags: u8,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum MemberKind {
    Method {
        method: MethodId,
    },
    Accessor {
        get: Option<MethodId>,
        set: Option<MethodId>,
    },
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct MemberSchema {
    pub name: &'static str,
    pub flags: u8,
    #[serde(flatten)]
    pub member: MemberKind,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct MethodSchema {
    pub id: MethodId,
    /// Declared callable name. Minifier-generated names and function source
    /// are outside this descriptor contract.
    pub name: &'static str,
    /// JavaScript function length, not an argument-count restriction.
    pub arity: u32,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag = "kind", content = "prototype", rename_all = "camelCase")]
pub enum PrototypeParent {
    Object,
    Null,
    Prototype(PrototypeId),
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct PrototypeSchema {
    pub id: PrototypeId,
    pub name: &'static str,
    pub parent: PrototypeParent,
    pub members: &'static [MemberSchema],
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct RecordSchema {
    pub id: RecordId,
    pub name: &'static str,
    pub kind: RecordKind,
    pub prototype: Option<PrototypeId>,
    pub fields: &'static [FieldSchema],
    /// Real own callable/accessor members, separate from prototype members.
    pub members: &'static [MemberSchema],
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ModelSchema {
    pub id: &'static str,
    pub version: u32,
    pub root: Option<RecordId>,
    pub records: &'static [RecordSchema],
    pub prototypes: &'static [PrototypeSchema],
    pub methods: &'static [MethodSchema],
}
/// A real callable imported by more than one model shares host identity even
/// though its native method IDs remain local to each document schema.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SharedFunction {
    pub method: MethodId,
    pub identity: &'static str,
}
/// Function construction is distinct from invocation. Most class methods are
/// nonconstructible; imported ordinary functions can retain their real shape.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct FunctionShape {
    pub method: MethodId,
    pub constructible: bool,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ModelSchemaView {
    #[serde(flatten)]
    pub model: &'static ModelSchema,
    #[serde(rename = "sharedFunctions")]
    pub shared_functions: &'static [SharedFunction],
    #[serde(rename = "functionShapes")]
    pub function_shapes: &'static [FunctionShape],
}
impl ModelSchemaView {
    pub fn validate(&self) -> Result<(), PeleError> {
        self.model.validate()?;
        if self.function_shapes.len() > self.model.methods.len() {
            return Err(invalid("Too many public function shapes"));
        }
        let methods: HashMap<_, _> = self.model.methods.iter().map(|m| (m.id, m)).collect();
        let mut shapes = HashMap::new();
        for shape in self.function_shapes {
            if !methods.contains_key(&shape.method)
                || shapes.insert(shape.method, shape.constructible).is_some()
            {
                return Err(invalid("Invalid public function shape"));
            }
        }
        if self.shared_functions.len() > self.model.methods.len() {
            return Err(invalid("Too many shared public function identities"));
        }
        let mut shared_methods = HashSet::new();
        let mut identities = HashMap::new();
        let mut units = 0usize;
        for entry in self.shared_functions {
            let method = methods
                .get(&entry.method)
                .ok_or_else(|| invalid("Unknown shared public method"))?;
            units = units.saturating_add(entry.identity.len());
            if units > 16_000_000 {
                return Err(limit());
            }
            if entry.identity.is_empty()
                || entry.identity.len() > 4096
                || !shared_methods.insert(entry.method)
            {
                return Err(invalid("Invalid shared public function identity"));
            }
            let descriptor = (
                method.name,
                method.arity,
                shapes.get(&entry.method).copied().unwrap_or(false),
            );
            if identities
                .insert(entry.identity, descriptor)
                .is_some_and(|previous| previous != descriptor)
            {
                return Err(invalid("Conflicting shared public function descriptor"));
            }
        }
        Ok(())
    }
}
/// Existing codecs remain usable without advertising unimplemented methods.
/// This descriptor makes no claim about their family prototype or fields.
pub static OPAQUE_MODEL_SCHEMA: ModelSchema = ModelSchema {
    id: "pele.opaque",
    version: 1,
    root: None,
    records: &[],
    prototypes: &[],
    methods: &[],
};
fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn limit() -> PeleError {
    PeleError::new("Public model schema exceeds work budget", ErrorCode::Limit)
}
impl ModelSchema {
    pub fn method(&self, id: MethodId) -> Option<&MethodSchema> {
        self.methods.iter().find(|m| m.id == id)
    }
    pub fn prototype(&self, id: PrototypeId) -> Option<&PrototypeSchema> {
        self.prototypes.iter().find(|p| p.id == id)
    }
    pub fn record(&self, id: RecordId) -> Option<&RecordSchema> {
        self.records.iter().find(|r| r.id == id)
    }
    pub fn validate(&self) -> Result<(), PeleError> {
        if self.id.is_empty() || self.version == 0 {
            return Err(invalid("Invalid public model schema identity"));
        }
        let count = self
            .records
            .len()
            .saturating_add(self.prototypes.len())
            .saturating_add(self.methods.len());
        if count > 100_000 {
            return Err(limit());
        }
        let mut work = count;
        let mut units = self.id.len();
        let mut records = HashSet::new();
        let mut prototypes = HashMap::new();
        let mut methods = HashSet::new();
        for r in self.records {
            if !records.insert(r.id) {
                return Err(invalid("Duplicate public record ID"));
            }
        }
        for p in self.prototypes {
            if prototypes.insert(p.id, p).is_some() {
                return Err(invalid("Duplicate public prototype ID"));
            }
        }
        for m in self.methods {
            if !methods.insert(m.id) {
                return Err(invalid("Duplicate public method ID"));
            }
            units = units.saturating_add(m.name.len());
        }
        if self.root.is_some_and(|id| !records.contains(&id)) {
            return Err(invalid("Unknown public root record"));
        }
        let mut members =
            |list: &[MemberSchema], names: &mut HashSet<&'static str>| -> Result<(), PeleError> {
                work = work.saturating_add(list.len());
                for m in list {
                    units = units.saturating_add(m.name.len());
                    if m.flags & !7 != 0 || !names.insert(m.name) {
                        return Err(invalid("Invalid public member descriptor"));
                    }
                    match m.member {
                        MemberKind::Method { method } if !methods.contains(&method) => {
                            return Err(invalid("Unknown public member method"))
                        }
                        MemberKind::Accessor { get, set } => {
                            if m.flags & 2 != 0 || (get.is_none() && set.is_none()) {
                                return Err(invalid("Invalid public accessor descriptor"));
                            }
                            for id in [get, set].into_iter().flatten() {
                                if !methods.contains(&id) {
                                    return Err(invalid("Unknown public accessor method"));
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(())
            };
        for r in self.records {
            if r.prototype.is_some_and(|id| !prototypes.contains_key(&id)) {
                return Err(invalid("Unknown public record prototype"));
            }
            let mut names = HashSet::new();
            // Account field work outside the closure's mutable borrows below.
            for f in r.fields {
                if f.flags & !7 != 0 || !names.insert(f.name) {
                    return Err(invalid("Invalid public field descriptor"));
                }
                if let FieldKind::Record(id) = f.kind {
                    if !records.contains(&id) {
                        return Err(invalid("Unknown public field record"));
                    }
                }
            }
            members(r.members, &mut names)?;
        }
        for p in self.prototypes {
            if let PrototypeParent::Prototype(id) = p.parent {
                if !prototypes.contains_key(&id) {
                    return Err(invalid("Unknown public parent prototype"));
                }
            }
            members(p.members, &mut HashSet::new())?;
        }
        for r in self.records {
            work = work.saturating_add(r.fields.len());
            units = units.saturating_add(r.name.len());
            for f in r.fields {
                units = units.saturating_add(f.name.len())
            }
        }
        for p in self.prototypes {
            units = units.saturating_add(p.name.len())
        }
        if work > 1_000_000 || units > 16_000_000 {
            return Err(limit());
        }
        // Prototype chains are shallow in Pele, but extension schemas may be deep.
        let mut colors = HashMap::new();
        for p in self.prototypes {
            let mut path = vec![];
            let mut at = Some(p.id);
            while let Some(id) = at {
                match colors.get(&id) {
                    Some(2) => break,
                    Some(1) => return Err(invalid("Public prototype chain contains a cycle")),
                    _ => {}
                }
                colors.insert(id, 1);
                path.push(id);
                at = match prototypes[&id].parent {
                    PrototypeParent::Prototype(id) => Some(id),
                    _ => None,
                };
            }
            for id in path {
                colors.insert(id, 2);
            }
        }
        Ok(())
    }
}
/// Private validation state for one captured immutable registration. Never used
/// by public validators accepting arbitrary (possibly stack-owned) schemas.
/// Construction validates before allocating the retained membership indexes.
pub(super) struct CompiledSchema {
    methods: HashSet<MethodId>,
    prototypes: HashSet<PrototypeId>,
}
impl CompiledSchema {
    pub(super) fn new(schema: &'static ModelSchema) -> Result<Self, PeleError> {
        schema.validate()?;
        Ok(Self {
            methods: schema.methods.iter().map(|method| method.id).collect(),
            prototypes: schema
                .prototypes
                .iter()
                .map(|prototype| prototype.id)
                .collect(),
        })
    }
    pub(super) fn methods(&self) -> &HashSet<MethodId> {
        &self.methods
    }
    pub(super) fn prototypes(&self) -> &HashSet<PrototypeId> {
        &self.prototypes
    }
}

/// IDs refer to the document's current graph, including detached retained nodes.
/// Borrowed receivers are passed unchanged; the typed module owns receiver rules.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MethodCall {
    pub method: MethodId,
    pub receiver: Atom,
    pub arguments: Vec<Atom>,
}
impl MethodCall {
    pub fn validate(&self, g: &ModelGraph, schema: &ModelSchema) -> Result<(), PeleError> {
        g.validate_schema(schema)?;
        let methods: HashSet<_> = schema.methods.iter().map(|m| m.id).collect();
        self.validate_after_graph(g, &methods)
    }
    pub(super) fn validate_compiled(
        &self,
        g: &ModelGraph,
        schema: &CompiledSchema,
    ) -> Result<(), PeleError> {
        g.validate_compiled(schema)?;
        self.validate_after_graph(g, schema.methods())
    }
    fn validate_after_graph(
        &self,
        g: &ModelGraph,
        methods: &HashSet<MethodId>,
    ) -> Result<(), PeleError> {
        if !methods.contains(&self.method) {
            return Err(invalid("Public model method is unavailable"));
        }
        if self.arguments.len() > 1_000_000 {
            return Err(PeleError::new(
                "Public method arguments exceed work budget",
                ErrorCode::Limit,
            ));
        }
        g.validate_atom(&self.receiver)?;
        for a in &self.arguments {
            g.validate_atom(a)?;
            if let Atom::BuiltinFunction(id) = a {
                if !methods.contains(&MethodId(*id)) {
                    return Err(invalid("Unknown public builtin function"));
                }
            }
        }
        if let Atom::BuiltinFunction(id) = self.receiver {
            if !methods.contains(&MethodId(id)) {
                return Err(invalid("Unknown public builtin function"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JavaScriptErrorKind {
    Error,
    TypeError,
    RangeError,
    ReferenceError,
    SyntaxError,
    EvalError,
    URIError,
}
/// JavaScript messages keep exact UTF-16; host exceptions keep identity tokens.
/// RenderResources still returns PeleError: adapters must provide a token-aware
/// host callback bridge before claiming arbitrary callback exception identity.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum PublicError {
    Pele(PeleError),
    PeleUtf16(PublicPeleError),
    JavaScript {
        name: JavaScriptErrorKind,
        message: JsString,
    },
    HostException(HostExceptionToken),
}
/// Lossless Pele error transport for messages/snippets outside UTF-8 scalars.
/// The serialized and observable name is always `PeleError`.
#[derive(Clone, Debug)]
pub struct PublicPeleError {
    pub message: JsString,
    pub code: ErrorCode,
    pub diagram_type: Option<JsString>,
    pub line: usize,
    pub column: usize,
    pub snippet: JsString,
}
impl PublicPeleError {
    pub fn name(&self) -> &'static str {
        "PeleError"
    }
    pub fn well_formed(&self) -> Option<PeleError> {
        Some(PeleError {
            name: self.name().into(),
            message: self.message.well_formed()?,
            code: self.code,
            diagram_type: match &self.diagram_type {
                Some(t) => Some(t.well_formed()?),
                None => None,
            },
            line: self.line,
            column: self.column,
            snippet: self.snippet.well_formed()?,
        })
    }
}
impl From<PeleError> for PublicPeleError {
    fn from(e: PeleError) -> Self {
        Self {
            message: JsString::from(e.message.as_str()),
            code: e.code,
            diagram_type: e.diagram_type.map(|s| JsString::from(s.as_str())),
            line: e.line,
            column: e.column,
            snippet: JsString::from(e.snippet.as_str()),
        }
    }
}
impl Serialize for PublicPeleError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut out = s.serialize_struct("PublicPeleError", 7)?;
        out.serialize_field("name", self.name())?;
        out.serialize_field("message", &self.message)?;
        out.serialize_field("code", &self.code)?;
        out.serialize_field("type", &self.diagram_type)?;
        out.serialize_field("line", &self.line)?;
        out.serialize_field("column", &self.column)?;
        out.serialize_field("snippet", &self.snippet)?;
        out.end()
    }
}
impl<'de> Deserialize<'de> for PublicPeleError {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            name: String,
            message: JsString,
            code: ErrorCode,
            #[serde(rename = "type")]
            diagram_type: Option<JsString>,
            line: usize,
            column: usize,
            snippet: JsString,
        }
        let v = Wire::deserialize(d)?;
        if v.name != "PeleError" {
            return Err(serde::de::Error::custom("Invalid lossless Pele error name"));
        }
        Ok(Self {
            message: v.message,
            code: v.code,
            diagram_type: v.diagram_type,
            line: v.line,
            column: v.column,
            snippet: v.snippet,
        })
    }
}
impl From<PublicPeleError> for PublicError {
    fn from(e: PublicPeleError) -> Self {
        Self::PeleUtf16(e)
    }
}
impl From<PeleError> for PublicError {
    fn from(e: PeleError) -> Self {
        Self::Pele(e)
    }
}
/// Both successful and throwing calls publish the graph and the new revision.
/// Outer invoke errors reject dispatch before mutation (or flag a broken codec
/// that could not export its state); operation errors live in `result`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvocationOutcome {
    pub revision: u64,
    pub graph: ModelGraph,
    pub result: Result<Atom, PublicError>,
}

/// Rendering can schedule tasks or update caches before returning or throwing.
/// The graph and revision publish those edits; export failure remains a protocol error.
#[derive(Clone, Debug)]
pub struct RenderOutcome {
    pub revision: u64,
    pub graph: ModelGraph,
    pub result: Result<crate::compat::RenderResult, PublicError>,
}
impl Serialize for RenderOutcome {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Outcome<'a> {
            revision: u64,
            graph: &'a ModelGraph,
            result: Result<crate::compat::RenderResultTransport<'a>, &'a PublicError>,
        }
        Outcome {
            revision: self.revision,
            graph: &self.graph,
            result: self
                .result
                .as_ref()
                .map(crate::compat::render_result_transport),
        }
        .serialize(serializer)
    }
}
