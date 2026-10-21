//! Admission-only replay of ER publication, before legacy-fallback host writes.
//! The actual publisher determines canonical table order. This sink keeps only
//! unique strings and aggregate counts; completed nodes are dropped immediately.
use super::{public_model::initial_into, ErModel, InitialWriter};
use crate::compat::{
    public::{Atom, Node, DEFAULT_BINARY_BUDGET},
    ErrorCode, PeleError,
};
use std::collections::HashMap;

const MAX_TABLE: usize = 1_000_000;
const MAX_UNITS: usize = 16_000_000;
const MAX_REFERENCES: usize = 8_000_000;
const INITIAL_METADATA: &[u8] = br#"{"type":"er"}"#;

pub(crate) fn admit(model: &ErModel) -> Result<(), PeleError> {
    initial_into(Admission::default(), model)
}

#[derive(Default)]
struct Admission {
    strings: HashMap<String, u32>,
    units: usize,
    nodes: usize,
    references: usize,
    node_bytes: usize,
    table_exceeded: bool,
}

fn atom_bytes(atom: &Atom) -> usize {
    match atom {
        Atom::Number(_) => 9,
        Atom::String(_) | Atom::Ref(_) | Atom::BuiltinFunction(_) | Atom::HostReference(_) => 5,
        Atom::Undefined | Atom::Null | Atom::Bool(_) => 1,
    }
}

impl InitialWriter for Admission {
    type Output = ();
    fn string(&mut self, value: &str) -> Result<u32, PeleError> {
        // Once the first validation category is certain, finish still determines
        // the error. Do not grow a doomed string table while traversing the rest.
        if self.table_exceeded {
            return Ok(0);
        }
        if let Some(id) = self.strings.get(value) {
            return Ok(*id);
        }
        let units = self.units.saturating_add(value.encode_utf16().count());
        if self.strings.len() == MAX_TABLE || units > MAX_UNITS {
            self.table_exceeded = true;
            return Ok(0);
        }
        let id = self.strings.len() as u32;
        self.units = units;
        self.strings.insert(value.to_owned(), id);
        Ok(id)
    }
    fn node(&mut self, node: Node) -> Result<u32, PeleError> {
        let id = self.nodes.min(MAX_TABLE) as u32;
        self.nodes = self.nodes.saturating_add(1);
        self.table_exceeded |= self.nodes > MAX_TABLE;
        let properties = node.properties();
        self.references = self.references.saturating_add(properties.len());
        // Canonical binary node tag + property count + property key/flags/value.
        let mut bytes = properties.iter().fold(5usize, |n, property| {
            n.saturating_add(5)
                .saturating_add(atom_bytes(&property.value))
        });
        match &node {
            Node::Object { .. } => {}
            Node::Instance { .. } | Node::Array { .. } => {
                bytes = bytes.saturating_add(4);
            }
            Node::Map { entries, .. } => {
                bytes = bytes.saturating_add(4);
                self.references = self
                    .references
                    .saturating_add(entries.len().saturating_mul(2));
                for (key, value) in entries {
                    bytes = bytes
                        .saturating_add(atom_bytes(key))
                        .saturating_add(atom_bytes(value));
                }
            }
            Node::Set { values, .. } => {
                bytes = bytes.saturating_add(4);
                self.references = self.references.saturating_add(values.len());
                for value in values {
                    bytes = bytes.saturating_add(atom_bytes(value));
                }
            }
            Node::Date { .. } => bytes = bytes.saturating_add(8),
            Node::Error { .. } => bytes = bytes.saturating_add(1),
        }
        self.node_bytes = self.node_bytes.saturating_add(bytes);
        Ok(id)
    }
    fn finish(self, root: Atom) -> Result<(), PeleError> {
        // This trusted publisher only creates valid references, flags, and
        // schema IDs. Match GraphWriter::finish's actual budget precedence:
        // table/UTF16 first, aggregate references second, facade bytes last.
        if self.table_exceeded {
            return Err(PeleError::new(
                "Public model graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        if self.references > MAX_REFERENCES {
            return Err(PeleError::new(
                "Public graph exceeds work budget",
                ErrorCode::Limit,
            ));
        }
        // MAGIC, version, revision, metadata length, string/node counts.
        let bytes = 32usize
            .saturating_add(atom_bytes(&root))
            .saturating_add(self.strings.len().saturating_mul(4))
            .saturating_add(self.units.saturating_mul(2))
            .saturating_add(self.node_bytes)
            .saturating_add(INITIAL_METADATA.len());
        if bytes > DEFAULT_BINARY_BUDGET {
            return Err(PeleError::new(
                "Public binary graph exceeds transport budget",
                ErrorCode::Limit,
            ));
        }
        Ok(())
    }
}
