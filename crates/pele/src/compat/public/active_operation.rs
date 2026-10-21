//! Callback-free access to an owner between suspended host operations.
use super::{ActiveModel, Atom, JsString, MethodCall, PublicError};
use crate::compat::{ErrorCode, PeleError};
#[doc(hidden)]
pub enum PublicActiveOperation {
    Validate(Atom),
    Read {
        receiver: Atom,
        key: String,
    },
    Set {
        receiver: Atom,
        key: String,
        value: Atom,
    },
    Intern(JsString),
    String(Atom),
    Object,
    Array,
    Date(f64),
    DateMillis(Atom),
    Push {
        receiver: Atom,
        value: Atom,
    },
    Builtin(MethodCall),
}
#[doc(hidden)]
pub enum PublicActiveValue {
    Atom(Atom),
    String(JsString),
}
impl PublicActiveOperation {
    pub(crate) fn mutates(&self) -> bool {
        !matches!(
            self,
            Self::Validate(_) | Self::Read { .. } | Self::String(_) | Self::DateMillis(_)
        )
    }
    /// No host call can be made by this dispatcher. Builtins require a module's
    /// explicit callback-free implementation; suspended ones use invocation jobs.
    pub(crate) fn run(self, model: &mut dyn ActiveModel) -> Result<PublicActiveValue, PublicError> {
        Ok(PublicActiveValue::Atom(match self {
            Self::Validate(v) => {
                model.validate(&v)?;
                Atom::Undefined
            }
            Self::Read { receiver, key } => model.read(&receiver, &key)?,
            Self::Set {
                receiver,
                key,
                value,
            } => {
                model.set(&receiver, &key, value)?;
                Atom::Undefined
            }
            Self::Intern(s) => model.intern(s)?,
            Self::String(v) => return model.string(&v).map(PublicActiveValue::String),
            Self::Object => model.object()?,
            Self::Array => model.array()?,
            Self::Date(n) => model.date(n)?,
            Self::DateMillis(v) => Atom::Number(model.date_millis(&v)?),
            Self::Push { receiver, value } => {
                model.push(&receiver, value)?;
                Atom::Undefined
            }
            Self::Builtin(_) => {
                return Err(PeleError::new(
                    "Owned builtin access is unavailable",
                    ErrorCode::Semantic,
                )
                .into())
            }
        }))
    }
}
/// Cumulative work belongs to the whole detached reader, not individual calls.
#[doc(hidden)]
pub struct PublicActiveBudget {
    pub(crate) remaining: usize,
}
impl Default for PublicActiveBudget {
    fn default() -> Self {
        Self {
            remaining: 1_000_000,
        }
    }
}
