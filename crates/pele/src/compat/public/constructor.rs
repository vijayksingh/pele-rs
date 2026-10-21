//! Captured native class construction, separate from parsed-model provenance.
use super::{Atom, DetachedPublicInvocation, JsString, PrototypeId};
use crate::compat::PeleError;
use std::any::TypeId;

/// Opaque builtin constructor implementation. Providers can forward discovery,
/// but only the concrete creator registered with the engine may consume it.
///
/// Hosts observe only the authenticated argument count. Ignored arguments are
/// never traversed; bodies run through owned native continuations.
#[doc(hidden)]
pub struct PublicConstructorFactory<S> {
    creator: TypeId,
    class: TypeId,
    prototype: PrototypeId,
    construct: Construction<S>,
    arguments: Arguments,
    arity: u32,
    live_fields: &'static [LiveConstructorField],
}
enum Arguments {
    Ignored,
    FirstValue,
    TwoValues,
}
enum Construction<S> {
    Ignored(fn() -> Result<S, PeleError>),
    First(fn(ConstructorArgument) -> Result<S, PeleError>),
    Body {
        default: fn() -> Result<S, PeleError>,
        initial: fn(ConstructorArgument) -> Result<S, PeleError>,
        body: fn(ConstructorArgument) -> DetachedPublicInvocation,
    },
    TwoBody {
        initial: fn([ConstructorArgument; 2]) -> Result<S, PeleError>,
        body: fn([ConstructorArgument; 2]) -> DetachedPublicInvocation,
    },
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
    /// Native graph-coordinate reply. Bindings never accept this from a wire tag;
    /// the owning document validates it before advancing a continuation.
    #[doc(hidden)]
    Graph(Atom),
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
            Self::Graph(atom) => atom,
        })
    }
}
/// Constructor arguments use the same owned scalar/foreign-value vocabulary as
/// pre-model host reads; no document string pool is required.
pub type ConstructorArgument = PublicHostValue;
/// Immutable transfer policy selected by the authentic native constructor.
/// `Value` copies known scalar values; foreign objects remain opaque.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructorArgumentTransfer {
    Retain,
    Value,
}
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
            arity: 0,
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
            arity: 0,
            live_fields,
        }
    }
    pub(crate) fn first_value_body<D: 'static>(
        prototype: PrototypeId,
        default: fn() -> Result<S, PeleError>,
        initial: fn(ConstructorArgument) -> Result<S, PeleError>,
        body: fn(ConstructorArgument) -> DetachedPublicInvocation,
        live_fields: &'static [LiveConstructorField],
    ) -> Self {
        Self {
            creator: TypeId::of::<D>(),
            class: TypeId::of::<D>(),
            prototype,
            arguments: Arguments::FirstValue,
            arity: 0,
            construct: Construction::Body {
                default,
                initial,
                body,
            },
            live_fields,
        }
    }
    pub(crate) fn two_value_body<D: 'static>(
        prototype: PrototypeId,
        initial: fn([ConstructorArgument; 2]) -> Result<S, PeleError>,
        body: fn([ConstructorArgument; 2]) -> DetachedPublicInvocation,
        live_fields: &'static [LiveConstructorField],
    ) -> Self {
        Self {
            creator: TypeId::of::<D>(),
            class: TypeId::of::<D>(),
            prototype,
            arguments: Arguments::TwoValues,
            arity: 2,
            construct: Construction::TwoBody { initial, body },
            live_fields,
        }
    }
    pub(super) fn argument_transfers(&self) -> &'static [ConstructorArgumentTransfer] {
        use ConstructorArgumentTransfer::{Retain, Value};
        match self.arguments {
            Arguments::Ignored => &[],
            Arguments::FirstValue => &[Retain],
            Arguments::TwoValues => &[Value, Value],
        }
    }
    pub(super) fn arity(&self) -> u32 {
        self.arity
    }
    pub(super) fn begin_arguments(
        &self,
        arguments: [ConstructorArgument; 2],
    ) -> Result<(S, Option<DetachedPublicInvocation>), PeleError> {
        if let Construction::TwoBody { initial, body } = self.construct {
            let state = initial(arguments.clone())?;
            let program = body(arguments)
                .for_creator_type(self.creator)
                .ok_or_else(|| {
                    PeleError::new(
                        "Constructor body has foreign creator",
                        crate::compat::ErrorCode::Semantic,
                    )
                })?;
            Ok((state, Some(program)))
        } else {
            let [first, _] = arguments;
            self.begin(first)
        }
    }
    pub(super) fn begin(
        &self,
        argument: ConstructorArgument,
    ) -> Result<(S, Option<DetachedPublicInvocation>), PeleError> {
        match self.construct {
            Construction::Body { initial, body, .. } => {
                let state = initial(argument.clone())?;
                let program = body(argument)
                    .for_creator_type(self.creator)
                    .ok_or_else(|| {
                        PeleError::new(
                            "Constructor body has foreign creator",
                            crate::compat::ErrorCode::Semantic,
                        )
                    })?;
                Ok((state, Some(program)))
            }
            _ => self.construct(argument).map(|state| (state, None)),
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
            Construction::Body { default, .. }
                if matches!(argument, ConstructorArgument::Undefined) =>
            {
                default()
            }
            Construction::Body { .. } | Construction::TwoBody { .. } => Err(PeleError::new(
                "Constructor requires host evaluation",
                crate::compat::ErrorCode::Semantic,
            )),
        }
    }
}
