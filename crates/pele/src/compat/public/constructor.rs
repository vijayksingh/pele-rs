//! Captured native class construction, separate from parsed-model provenance.
use super::{Atom, JsString, PrototypeId};
use crate::compat::PeleError;
use std::any::TypeId;

/// Opaque builtin constructor implementation. Providers can forward discovery,
/// but only the concrete creator registered with the engine may consume it.
///
/// Ignored-argument factories never observe arguments. First-value factories
/// retain an explicit owned scalar or owner-local opaque identity without reads.
#[doc(hidden)]
pub struct PublicConstructorFactory<S> {
    creator: TypeId,
    class: TypeId,
    prototype: PrototypeId,
    construct: Construction<S>,
    arguments: Arguments,
    live_fields: &'static [LiveConstructorField],
}
enum Arguments {
    Ignored,
    FirstValue,
}
enum Construction<S> {
    Ignored(fn() -> Result<S, PeleError>),
    First(fn(ConstructorArgument) -> Result<S, PeleError>),
}
/// Explicit lossless first argument. Foreign identities are resolved by the
/// owning host; construction never enumerates or coerces an opaque value.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub enum PublicHostValue {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(JsString),
    Opaque(u32),
}
impl PublicHostValue {
    /// Import one owned scalar. Only strings require the owning graph's interner.
    /// Undefined is not defaulted here; default arguments are constructor policy.
    pub fn into_atom<E>(self, intern: impl FnOnce(JsString) -> Result<Atom, E>) -> Result<Atom, E> {
        Ok(match self {
            Self::Undefined => Atom::Undefined,
            Self::Null => Atom::Null,
            Self::Bool(value) => Atom::Bool(value),
            Self::Number(value) => Atom::Number(value),
            Self::String(value) => return intern(value),
            Self::Opaque(id) => Atom::HostReference(id),
        })
    }
}
/// Constructor arguments use the same owned scalar/foreign-value vocabulary as
/// pre-model host reads; no document string pool is required.
pub type ConstructorArgument = PublicHostValue;
#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct LiveConstructorField {
    pub key: &'static str,
    pub methods: &'static [super::MethodId],
}
impl<S> PublicConstructorFactory<S> {
    pub(crate) fn ignoring_arguments<D: 'static>(
        prototype: PrototypeId,
        construct: fn() -> Result<S, PeleError>,
    ) -> Self {
        Self {
            creator: TypeId::of::<D>(),
            class: TypeId::of::<D>(),
            prototype,
            construct: Construction::Ignored(construct),
            arguments: Arguments::Ignored,
            live_fields: &[],
        }
    }
    pub(crate) fn first_value<D: 'static>(
        prototype: PrototypeId,
        construct: fn(ConstructorArgument) -> Result<S, PeleError>,
        live_fields: &'static [LiveConstructorField],
    ) -> Self {
        Self {
            creator: TypeId::of::<D>(),
            class: TypeId::of::<D>(),
            prototype,
            construct: Construction::First(construct),
            arguments: Arguments::FirstValue,
            live_fields,
        }
    }
    pub(crate) fn sharing_class<D: 'static>(mut self) -> Self {
        self.class = TypeId::of::<D>();
        self
    }
    pub(super) fn class_id(&self) -> TypeId {
        self.class
    }
    pub(super) fn for_creator<D: 'static>(self, prototype: PrototypeId) -> Option<Self> {
        (self.creator == TypeId::of::<D>() && self.prototype == prototype).then_some(self)
    }
    pub(super) fn live_fields(&self) -> &'static [LiveConstructorField] {
        self.live_fields
    }
    pub(super) fn ignores_arguments(&self) -> bool {
        matches!(self.arguments, Arguments::Ignored)
    }
    pub(super) fn construct(&self, argument: ConstructorArgument) -> Result<S, PeleError> {
        match self.construct {
            Construction::Ignored(f) => f(),
            Construction::First(f) => f(argument),
        }
    }
}
