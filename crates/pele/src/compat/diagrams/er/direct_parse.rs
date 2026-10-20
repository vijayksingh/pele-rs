//! Construction-only ER target. Final arena layout is exactly the typed adapter's
//! layout; declarations that would require rewriting it use the existing parser.
use super::{
    model::{DocItem, ErAttribute, ErModel, RelSpec},
    parser::{self, Target},
    public_model::{
        arr, finish_initial, publish_entity, publish_spec, EntityPublication, InitialRoots,
        InitialWriter, RELATIONSHIP_FIELDS,
    },
};
use crate::compat::{
    public::{Atom, GraphWriter, ModelGraph, Node},
    ErrorCode, ParseContext, PeleError,
};
use std::collections::HashMap;

// Conservative private eligibility, not a new public input limit. It bounds
// unpublished construction while GraphWriter retains its final exact admission.
// Larger sources/headers keep the established typed path without new errors.
const MAX_DIRECT_SOURCE_BYTES: usize = 64 * 1024;
struct Direct<W> {
    writer: W,
    header: ErModel,
    entities: Vec<(Atom, Atom)>,
    ids: HashMap<String, Atom>,
    entity_map: Option<u32>,
    relationships: Vec<Atom>,
    fallback: bool,
}
impl<W: InitialWriter> Direct<W> {
    fn fallback<T>(&mut self) -> Result<T, PeleError> {
        self.fallback = true;
        Err(PeleError::new(
            "Private ER construction fallback",
            ErrorCode::Semantic,
        ))
    }
    fn ensure_entity(
        &mut self,
        name: &str,
        alias: Option<&str>,
        attributes: &[ErAttribute],
        order: &[bool],
    ) -> Result<Atom, PeleError> {
        if let Some(id) = self.ids.get(name) {
            if alias.is_some() || !attributes.is_empty() {
                return self.fallback();
            }
            return Ok(id.clone());
        }
        if self.entity_map.is_some() {
            return self.fallback();
        }
        let id = format!("entity-{name}-{}", self.entities.len());
        let (key, node, id) = publish_entity(
            &mut self.writer,
            EntityPublication {
                name,
                id: &id,
                label: name,
                alias: alias.unwrap_or(""),
                attributes,
                classes: "default",
                styles: &[],
            },
            |index| order.get(index).copied().unwrap_or(false),
        )?;
        self.entities.push((key, node));
        self.ids.insert(name.to_owned(), id.clone());
        Ok(id)
    }
    fn seal_entities(&mut self) -> Result<u32, PeleError> {
        if let Some(id) = self.entity_map {
            return Ok(id);
        }
        let id = self.writer.node(Node::Map {
            entries: std::mem::take(&mut self.entities),
            properties: vec![],
        })?;
        self.entity_map = Some(id);
        Ok(id)
    }
    fn finish(mut self) -> Result<W::Output, PeleError> {
        let entities = self.seal_entities()?;
        let relationships = arr(&mut self.writer, self.relationships)?;
        let classes = self.writer.node(Node::Map {
            entries: vec![],
            properties: vec![],
        })?;
        let subgraphs = arr(&mut self.writer, [])?;
        let lookup = self.writer.node(Node::Map {
            entries: vec![],
            properties: vec![],
        })?;
        let claimed = self.writer.node(Node::Set {
            values: vec![],
            properties: vec![],
        })?;
        finish_initial(
            self.writer,
            &self.header,
            InitialRoots {
                entities,
                relationships,
                classes,
                subgraphs,
                lookup,
                claimed,
            },
        )
    }
}
impl<W: InitialWriter> Target for Direct<W> {
    fn header(&mut self) -> &mut ErModel {
        &mut self.header
    }
    fn entity(
        &mut self,
        name: &str,
        alias: Option<&str>,
        attributes: Vec<ErAttribute>,
        order: Vec<bool>,
    ) -> Result<(), PeleError> {
        self.ensure_entity(name, alias, &attributes, &order)
            .map(|_| ())
    }
    fn relationship(
        &mut self,
        a: &str,
        role: String,
        b: &str,
        spec: RelSpec,
    ) -> Result<(), PeleError> {
        let a = self.ensure_entity(a, None, &[], &[])?;
        let b = self.ensure_entity(b, None, &[], &[])?;
        self.seal_entities()?;
        let spec = publish_spec(&mut self.writer, &spec)?;
        // Endpoint strings were interned by their entity records. The legacy
        // adapter's string lookups are hits here; no string-order change.
        let role = Atom::String(self.writer.string(role.as_str())?);
        let relation = self
            .writer
            .record(&RELATIONSHIP_FIELDS, [a, role, b, Atom::Ref(spec)])?;
        self.relationships.push(Atom::Ref(relation));
        Ok(())
    }
    fn subgraph(&mut self, _: String, _: String, _: Vec<DocItem>) -> Result<String, PeleError> {
        self.fallback()
    }
    fn add_class(&mut self, _: &[String], _: &[String]) -> Result<(), PeleError> {
        self.fallback()
    }
    fn styles(&mut self, _: &[String], _: &[String]) -> Result<(), PeleError> {
        self.fallback()
    }
    fn set_class(&mut self, _: &[String], names: &[String]) -> Result<(), PeleError> {
        if names.is_empty() {
            Ok(())
        } else {
            self.fallback()
        }
    }
}
pub(super) fn parse(context: ParseContext<'_>) -> Option<Result<ModelGraph, PeleError>> {
    parse_into(context, GraphWriter::default())
}
#[doc(hidden)]
pub fn parse_into<W: InitialWriter>(
    context: ParseContext<'_>,
    writer: W,
) -> Option<Result<W::Output, PeleError>> {
    if context.source.len() > MAX_DIRECT_SOURCE_BYTES
        || context
            .title
            .is_some_and(|title| title.len() > MAX_DIRECT_SOURCE_BYTES)
    {
        return writer.abort().err().map(Err);
    }
    let mut target = Direct {
        writer,
        header: ErModel::new(context.title),
        entities: vec![],
        ids: HashMap::new(),
        entity_map: None,
        relationships: vec![],
        fallback: false,
    };
    if let Err(error) = parser::parse_into(context.source, &mut target) {
        let fallback = target.fallback;
        if let Err(abort) = target.writer.abort() {
            return Some(Err(abort));
        }
        return if fallback { None } else { Some(Err(error)) };
    }
    // None drops all provisional state before PublicEngine's single legacy
    // parse/create fallback. Preparation and provider selection are not repeated.
    target.finish().ok().map(Ok)
}
