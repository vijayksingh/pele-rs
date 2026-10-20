//! Captured native class construction, separate from parsed-model provenance.
use super::PrototypeId;
use crate::compat::PeleError;
use std::any::TypeId;

/// Opaque builtin constructor implementation. Providers can forward discovery,
/// but only the concrete creator registered with the engine may consume it.
///
/// The current Sankey constructor ignores every argument. Hosts must not observe
/// or serialize ignored arguments before invoking the captured factory.
#[doc(hidden)]
pub struct PublicConstructorFactory<S> {
    creator: TypeId,
    prototype: PrototypeId,
    construct: fn() -> Result<S, PeleError>,
    arguments: Arguments,
}
enum Arguments {
    Ignored,
}
impl<S> PublicConstructorFactory<S> {
    pub(crate) fn ignoring_arguments<D: 'static>(
        prototype: PrototypeId,
        construct: fn() -> Result<S, PeleError>,
    ) -> Self {
        Self {
            creator: TypeId::of::<D>(),
            prototype,
            construct,
            arguments: Arguments::Ignored,
        }
    }
    pub(super) fn for_creator<D: 'static>(self, prototype: PrototypeId) -> Option<Self> {
        (self.creator == TypeId::of::<D>() && self.prototype == prototype).then_some(self)
    }
    pub(super) fn ignores_arguments(&self) -> bool {
        matches!(self.arguments, Arguments::Ignored)
    }
    pub(super) fn construct(&self) -> Result<S, PeleError> {
        (self.construct)()
    }
}
