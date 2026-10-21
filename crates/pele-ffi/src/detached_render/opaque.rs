//! Lossless values for an optional owner ledger; no arena interning on transfer.
use super::*;
use crate::{host_values, PeleHostRequestV20, PeleHostValueV17};
use pele::compat::public::{PublicHostResult, PublicHostValue};
impl Reader {
    pub(super) fn opaque_operation(
        &self,
        operation: u32,
        values: &[&Value],
        key: &str,
        message: &str,
        policy: PublicHostResult,
    ) -> Result<Option<Value>, PublicError> {
        if values.len() > 1_000_002 {
            return Err(
                PeleError::new("Opaque host access exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let total = values.iter().fold(
            key.encode_utf16()
                .count()
                .saturating_add(message.encode_utf16().count()),
            |n, v| {
                n.saturating_add(match v {
                    Value::String(s) => s.0.len(),
                    _ => 0,
                })
            },
        );
        if total > 16_000_000 {
            return Err(
                PeleError::new("Opaque host access exceeds work budget", ErrorCode::Limit).into(),
            );
        }
        let packets: Vec<PeleHostValueV17> = values
            .iter()
            .map(|value| {
                Ok(match value {
                    Value::String(s) => host_values::packet(4, 0, &s.0),
                    Value::Atom(atom) => match atom {
                        Atom::Undefined => host_values::packet(0, 0, &[]),
                        Atom::Null => host_values::packet(1, 0, &[]),
                        Atom::Bool(b) => host_values::packet(2, u64::from(*b), &[]),
                        Atom::Number(n) => host_values::packet(3, n.to_bits(), &[]),
                        Atom::HostReference(id) => host_values::packet(5, u64::from(*id), &[]),
                        Atom::Ref(id) => host_values::packet(6, u64::from(*id), &[]),
                        Atom::BuiltinFunction(id) => host_values::packet(7, u64::from(*id), &[]),
                        Atom::String(_) => {
                            return Err(invalid("Unowned live string transfer").into())
                        }
                    },
                    _ => {
                        return Err(invalid(
                            "Opaque host access requires scalar or graph-coordinate arguments",
                        )
                        .into())
                    }
                })
            })
            .collect::<Result<_, PublicError>>()?;
        let key: Vec<u16> = key.encode_utf16().collect();
        let error: Vec<u16> = message.encode_utf16().collect();
        let request = PeleHostRequestV20 {
            operation,
            result: match policy {
                PublicHostResult::Value => 0,
                PublicHostResult::Retain => 1,
                PublicHostResult::Discard => 2,
            },
            values: packets.as_ptr(),
            count: packets.len(),
            key: key.as_ptr(),
            key_count: key.len(),
            error: error.as_ptr(),
            error_count: error.len(),
        };
        host_values::dispatch(self.owner, &request).map(|value| {
            value.map(|v| match v {
                PublicHostValue::Undefined => Value::Atom(Atom::Undefined),
                PublicHostValue::Null => Value::Atom(Atom::Null),
                PublicHostValue::Bool(v) => Value::Atom(Atom::Bool(v)),
                PublicHostValue::Number(v) => Value::Atom(Atom::Number(v)),
                PublicHostValue::String(s) => Value::String(Arc::new(s)),
                PublicHostValue::Opaque(id) => Value::Atom(Atom::HostReference(id)),
                PublicHostValue::Graph(atom) => Value::Atom(atom),
            })
        })
    }
}
