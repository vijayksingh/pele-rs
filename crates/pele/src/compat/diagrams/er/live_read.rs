//! ER live read boundary. Geometry, capture and evaluation order stay in Rust.
//!
//! Store calls return owned values. A host implementation must release every
//! interior borrow before returning, including while reporting a host exception.
//! Document admission/history checks precede this seam; live-read counters are
//! separate and must bound callback-driven growth without reimporting a model.
use super::{
    graph::{self, Edge, Node},
    model::{ErAttribute, ErModel},
};
#[cfg(feature = "web-host-er")]
use crate::compat::ErrorCode;
use crate::compat::PeleError;
use crate::compat::{
    public::{Atom, JsString},
    source::space,
    Object, RenderResources, RenderSettings, Rendered,
};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

/// Private browser-store vocabulary, exported only through the hidden adapter.
/// No primitive may retain a Rust borrow/guard while invoking JavaScript.
/// Collection/string primitives must check bounds before allocating results.
pub trait RenderStore {
    fn root(&self) -> Atom;
    fn field(&self, receiver: &Atom, key: &str) -> Result<Atom, PeleError>;
    fn array_length(&self, array: &Atom) -> Result<usize, PeleError>;
    fn array_get(&self, array: &Atom, index: usize) -> Result<Atom, PeleError>;
    fn array_has(&self, array: &Atom, index: usize) -> Result<bool, PeleError>;
    fn map_entries(&self, map: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError>;
    fn map_get(&self, map: &Atom, key: &str) -> Result<Atom, PeleError>;
    fn map_get_value(&self, map: &Atom, key: &Atom) -> Result<Atom, PeleError>;
    fn string_units(&self, value: &Atom) -> Result<JsString, PeleError>;
    fn coerce_string(&self, value: &Atom) -> Result<String, PeleError>;
    /// Strict string read, preserving original UTF16 until this conversion.
    /// `operation` selects the actual failed string operation for non-strings:
    /// capture/trim/split/toLowerCase/includes/indexOf/replace/label/accessibility. Do not coerce
    /// a malformed cell or optional label to a different accepted value.
    fn string(&self, value: &Atom, operation: &'static str) -> Result<String, PeleError>;
    /// Generic Array.join primitive, including holes/null/undefined and the
    /// existing coercion/throw policy. No ER-specific interpretation here.
    fn join(&self, array: &Atom, separator: &str) -> Result<String, PeleError>;
    fn truthy(&self, value: &Atom) -> Result<bool, PeleError>;
    /// Charge bounded live traversal separately from initial import admission.
    fn charge(&self, work: usize, utf16_units: usize) -> Result<(), PeleError>;
}

#[derive(Clone, Copy)]
pub(super) enum CellField {
    Type,
    Name,
    Comment,
}
impl CellField {
    fn key(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Name => "name",
            Self::Comment => "comment",
        }
    }
}

/// Shared draw interface. Typed handles borrow immutable model slices; host
/// handles are owned Atoms. No complete imported model or value mirror exists.
pub(super) trait ErRead {
    type Attributes: Clone;
    type Styles: Clone;
    type Row;
    type Scalar: Clone;
    type Key: Clone + Eq + std::hash::Hash;
    fn capture(
        &self,
    ) -> Result<
        (
            Vec<Node<Self::Attributes, Self::Styles, Self::Scalar>>,
            Vec<Edge<Self::Scalar>>,
        ),
        PeleError,
    >;
    fn key<'a>(&'a self, value: &'a Self::Scalar) -> Result<Cow<'a, Self::Key>, PeleError>;
    fn scalar_text<'a>(
        &'a self,
        value: &'a Self::Scalar,
        operation: &'static str,
    ) -> Result<Cow<'a, str>, PeleError>;
    fn label<'a>(&'a self, value: &'a Self::Scalar) -> Result<Option<Cow<'a, str>>, PeleError>;
    fn truthy(&self, value: &Self::Scalar) -> Result<bool, PeleError>;
    fn scalar_direction<'a>(&'a self, value: &'a Self::Scalar) -> Result<Cow<'a, str>, PeleError>;
    fn direction(&self) -> Result<Cow<'_, str>, PeleError>;
    fn title(&self) -> Result<Option<Cow<'_, str>>, PeleError>;
    fn acc_title(&self) -> Result<Option<Cow<'_, str>>, PeleError>;
    fn acc_descr(&self) -> Result<Option<Cow<'_, str>>, PeleError>;
    fn styles<'a>(&'a self, styles: &'a Self::Styles) -> Result<Cow<'a, [String]>, PeleError>;
    fn attribute_len(&self, attributes: &Self::Attributes) -> Result<usize, PeleError>;
    fn attribute_at(
        &self,
        attributes: &Self::Attributes,
        index: usize,
    ) -> Result<Option<Self::Row>, PeleError>;
    fn cell<'a>(&'a self, row: &'a Self::Row, field: CellField) -> Result<Cow<'a, str>, PeleError>;
    fn keys(&self, row: &Self::Row) -> Result<String, PeleError>;
}

pub(super) struct TypedRead<'a>(pub &'a ErModel);
impl<'m> ErRead for TypedRead<'m> {
    type Attributes = &'m [ErAttribute];
    type Styles = &'m [String];
    type Row = &'m ErAttribute;
    type Scalar = String;
    type Key = String;
    fn key<'a>(&'a self, value: &'a String) -> Result<Cow<'a, String>, PeleError> {
        Ok(Cow::Borrowed(value))
    }
    fn scalar_text<'a>(
        &'a self,
        value: &'a String,
        _operation: &'static str,
    ) -> Result<Cow<'a, str>, PeleError> {
        Ok(Cow::Borrowed(value))
    }
    fn label<'a>(&'a self, value: &'a String) -> Result<Option<Cow<'a, str>>, PeleError> {
        Ok(Some(Cow::Borrowed(value)))
    }
    fn truthy(&self, value: &String) -> Result<bool, PeleError> {
        Ok(!value.is_empty())
    }
    fn scalar_direction<'a>(&'a self, value: &'a String) -> Result<Cow<'a, str>, PeleError> {
        Ok(Cow::Borrowed(value))
    }
    fn capture(
        &self,
    ) -> Result<
        (
            Vec<Node<Self::Attributes, Self::Styles, Self::Scalar>>,
            Vec<Edge<Self::Scalar>>,
        ),
        PeleError,
    > {
        Ok(graph::build(self.0))
    }
    fn direction(&self) -> Result<Cow<'_, str>, PeleError> {
        Ok(Cow::Borrowed(&self.0.direction))
    }
    fn title(&self) -> Result<Option<Cow<'_, str>>, PeleError> {
        Ok(self.0.title.as_deref().map(Cow::Borrowed))
    }
    fn acc_title(&self) -> Result<Option<Cow<'_, str>>, PeleError> {
        Ok(self.0.acc_title.as_deref().map(Cow::Borrowed))
    }
    fn acc_descr(&self) -> Result<Option<Cow<'_, str>>, PeleError> {
        Ok(self.0.acc_descr.as_deref().map(Cow::Borrowed))
    }
    fn styles<'a>(&'a self, styles: &'a Self::Styles) -> Result<Cow<'a, [String]>, PeleError> {
        Ok(Cow::Borrowed(*styles))
    }
    fn attribute_len(&self, attributes: &Self::Attributes) -> Result<usize, PeleError> {
        Ok(attributes.len())
    }
    fn attribute_at(
        &self,
        attributes: &Self::Attributes,
        index: usize,
    ) -> Result<Option<Self::Row>, PeleError> {
        Ok((*attributes).get(index))
    }
    fn cell<'a>(&'a self, row: &'a Self::Row, field: CellField) -> Result<Cow<'a, str>, PeleError> {
        Ok(Cow::Borrowed(match field {
            CellField::Type => &row.r#type,
            CellField::Name => &row.name,
            CellField::Comment => &row.comment,
        }))
    }
    fn keys(&self, row: &Self::Row) -> Result<String, PeleError> {
        Ok(row.keys.join(", "))
    }
}

#[derive(Clone)]
pub(super) enum HostScalar {
    Captured(Atom),
    Text(String),
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub(super) enum RenderKey {
    Undefined,
    Null,
    Bool(bool),
    Number(u64),
    String(JsString),
    Ref(u32),
    Function(u32),
    Host(u32),
}
struct HostRead<'a, S: RenderStore + ?Sized> {
    store: &'a S,
    root: Atom,
}
impl<S: RenderStore + ?Sized> HostRead<'_, S> {
    fn atom_key(&self, value: &Atom) -> Result<RenderKey, PeleError> {
        Ok(match value {
            Atom::Undefined => RenderKey::Undefined,
            Atom::Null => RenderKey::Null,
            Atom::Bool(v) => RenderKey::Bool(*v),
            Atom::Number(v) => RenderKey::Number(if v.is_nan() {
                f64::NAN.to_bits()
            } else if *v == 0. {
                0
            } else {
                v.to_bits()
            }),
            Atom::String(_) => {
                let value = self.store.string_units(value)?;
                self.store.charge(0, value.0.len())?;
                RenderKey::String(value)
            }
            Atom::Ref(id) => RenderKey::Ref(*id),
            Atom::BuiltinFunction(id) => RenderKey::Function(*id),
            Atom::HostReference(id) => RenderKey::Host(*id),
        })
    }
    fn atom_direction(&self, value: &Atom) -> Result<&'static str, PeleError> {
        if !matches!(value, Atom::String(_)) {
            return Ok("");
        }
        let units = self.store.string_units(value)?;
        self.store.charge(0, units.0.len())?;
        Ok(if units.is("LR") {
            "LR"
        } else if units.is("RL") {
            "RL"
        } else if units.is("BT") {
            "BT"
        } else {
            "TB"
        })
    }
    fn text(&self, value: &Atom, operation: &'static str) -> Result<String, PeleError> {
        let text = self.store.string(value, operation)?;
        self.store.charge(0, text.encode_utf16().count())?;
        Ok(text)
    }
    fn field_text(
        &self,
        value: &Atom,
        key: &str,
        operation: &'static str,
    ) -> Result<String, PeleError> {
        self.text(&self.store.field(value, key)?, operation)
    }
    fn text_array(
        &self,
        array: &Atom,
        operation: &'static str,
        trim: bool,
    ) -> Result<Vec<String>, PeleError> {
        let length = self.store.array_length(array)?;
        self.store.charge(length, 0)?;
        let mut out = Vec::with_capacity(length);
        for n in 0..length {
            let text = self.text(&self.store.array_get(array, n)?, operation)?;
            out.push(if trim {
                text.trim_matches(space).to_owned()
            } else {
                text
            });
        }
        Ok(out)
    }
    fn compile_class(&self, class: &Atom, out: &mut Vec<String>) -> Result<(), PeleError> {
        if matches!(class, Atom::Undefined) {
            return Ok(());
        }
        for key in ["styles", "textStyles"] {
            out.extend(self.text_array(&self.store.field(class, key)?, "trim", true)?);
        }
        Ok(())
    }
    fn compiled(&self, names: &[String]) -> Result<Vec<String>, PeleError> {
        let mut out = Vec::new();
        for name in names {
            self.store.charge(1, 0)?;
            let classes = self.store.field(&self.root, "classes")?;
            self.compile_class(&self.store.map_get(&classes, name)?, &mut out)?;
        }
        Ok(out)
    }
    fn compiled_group(&self, names: &Atom) -> Result<Vec<String>, PeleError> {
        let count = self.store.array_length(names)?;
        self.store.charge(count, 0)?;
        let mut out = Vec::new();
        for n in 0..count {
            let name = self.store.array_get(names, n)?;
            let classes = self.store.field(&self.root, "classes")?;
            self.compile_class(&self.store.map_get_value(&classes, &name)?, &mut out)?;
        }
        Ok(out)
    }
    fn optional_root(
        &self,
        key: &str,
        truthy: bool,
        operation: &'static str,
    ) -> Result<Option<Cow<'_, str>>, PeleError> {
        let value = self.store.field(&self.root, key)?;
        if (truthy && !self.store.truthy(&value)?) || (!truthy && matches!(value, Atom::Undefined))
        {
            Ok(None)
        } else {
            self.text(&value, operation).map(|v| Some(Cow::Owned(v)))
        }
    }
}

impl<S: RenderStore + ?Sized> ErRead for HostRead<'_, S> {
    type Attributes = Atom;
    type Styles = Atom;
    type Row = Atom;
    type Scalar = HostScalar;
    type Key = RenderKey;
    fn key<'a>(&'a self, value: &'a HostScalar) -> Result<Cow<'a, RenderKey>, PeleError> {
        Ok(Cow::Owned(match value {
            HostScalar::Captured(value) => self.atom_key(value)?,
            HostScalar::Text(value) => RenderKey::String(JsString::from(value.as_str())),
        }))
    }
    fn scalar_text<'a>(
        &'a self,
        value: &'a HostScalar,
        operation: &'static str,
    ) -> Result<Cow<'a, str>, PeleError> {
        match value {
            HostScalar::Captured(value) => self.text(value, operation).map(Cow::Owned),
            HostScalar::Text(value) => Ok(Cow::Borrowed(value)),
        }
    }
    fn label<'a>(&'a self, value: &'a HostScalar) -> Result<Option<Cow<'a, str>>, PeleError> {
        if matches!(value, HostScalar::Captured(Atom::Undefined)) {
            Ok(None)
        } else {
            self.scalar_text(value, "label").map(Some)
        }
    }
    fn truthy(&self, value: &HostScalar) -> Result<bool, PeleError> {
        match value {
            HostScalar::Captured(value) => self.store.truthy(value),
            HostScalar::Text(value) => Ok(!value.is_empty()),
        }
    }
    fn scalar_direction<'a>(&'a self, value: &'a HostScalar) -> Result<Cow<'a, str>, PeleError> {
        match value {
            HostScalar::Captured(value) => self.atom_direction(value).map(Cow::Borrowed),
            HostScalar::Text(value) => Ok(Cow::Borrowed(value)),
        }
    }
    fn capture(
        &self,
    ) -> Result<(Vec<Node<Atom, Atom, HostScalar>>, Vec<Edge<HostScalar>>), PeleError> {
        // ft captures values, not validated Strings. Defer label/alias/id text
        // operations until their actual use; a later pass may capture values
        // changed by callbacks even though initial document admission passed.
        let s = self.store;
        let groups = s.field(&self.root, "subgraphs")?;
        let mut parent = HashMap::new();
        let mut ids = HashSet::new();
        let count = s.array_length(&groups)?;
        s.charge(count, 0)?;
        for n in (0..count).rev() {
            let group = s.array_get(&groups, n)?;
            let id = s.field(&group, "id")?;
            ids.insert(self.atom_key(&id)?);
            let children = s.field(&group, "nodes")?;
            let length = s.array_length(&children)?;
            s.charge(length, 0)?;
            for child in 0..length {
                let child = s.array_get(&children, child)?;
                parent.insert(self.atom_key(&child)?, s.field(&group, "id")?);
            }
        }
        let mut nodes = Vec::new();
        let count = s.array_length(&groups)?;
        s.charge(count, 0)?;
        for n in (0..count).rev() {
            let group = s.array_get(&groups, n)?;
            let id = s.field(&group, "id")?;
            let label = s.field(&group, "title")?;
            let parent_id = parent
                .get(&self.atom_key(&s.field(&group, "id")?)?)
                .cloned();
            let css_styles = s.field(&group, "cssStyles")?;
            let css_compiled_styles = self.compiled_group(&s.field(&group, "classes")?)?;
            let css_classes = s.join(&s.field(&group, "classes")?, " ")?;
            let dir = Some(HostScalar::Captured(s.field(&group, "dir")?));
            nodes.push(Node {
                id: HostScalar::Captured(id),
                label: HostScalar::Captured(label),
                alias: HostScalar::Text(String::new()),
                attributes: None,
                is_group: true,
                parent_id: parent_id
                    .filter(|v| !matches!(v, Atom::Undefined))
                    .map(HostScalar::Captured),
                css_styles,
                css_compiled_styles,
                css_classes: HostScalar::Text(css_classes),
                dir,
            });
        }
        let mut moved = HashMap::new();
        let entities = s.map_entries(&s.field(&self.root, "entities")?)?;
        s.charge(entities.len(), 0)?;
        for (key, entity) in entities {
            if ids.contains(&self.atom_key(&key)?) {
                moved.insert(self.atom_key(&s.field(&entity, "id")?)?, key);
                continue;
            }
            let id = s.field(&entity, "id")?;
            let label = s.field(&entity, "label")?;
            let alias = s.field(&entity, "alias")?;
            let attributes = Some(s.field(&entity, "attributes")?);
            let parent_id = parent.get(&self.atom_key(&key)?).cloned();
            let css_styles = s.field(&entity, "cssStyles")?;
            // split is a real capture-time operation; it must still reject here.
            let names = self.field_text(&entity, "cssClasses", "split")?;
            let names = names.split(' ').map(str::to_owned).collect::<Vec<_>>();
            let css_compiled_styles = self.compiled(&names)?;
            let css_classes = s.field(&entity, "cssClasses")?;
            nodes.push(Node {
                id: HostScalar::Captured(id),
                label: HostScalar::Captured(label),
                alias: HostScalar::Captured(alias),
                attributes,
                is_group: false,
                parent_id: parent_id
                    .filter(|v| !matches!(v, Atom::Undefined))
                    .map(HostScalar::Captured),
                css_styles,
                css_compiled_styles,
                css_classes: HostScalar::Captured(css_classes),
                dir: None,
            });
        }
        let relationships = s.field(&self.root, "relationships")?;
        let count = s.array_length(&relationships)?;
        s.charge(count, 0)?;
        let mut edges = Vec::with_capacity(count);
        for n in 0..count {
            if !s.array_has(&relationships, n)? {
                continue;
            }
            let relationship = s.array_get(&relationships, n)?;
            // Template interpolation coerces endpoints now; lookup keys and
            // endpoints remain original values, not their formatted spelling.
            let a_text = s.coerce_string(&s.field(&relationship, "entityA")?)?;
            let b_text = s.coerce_string(&s.field(&relationship, "entityB")?)?;
            s.charge(
                0,
                a_text
                    .encode_utf16()
                    .count()
                    .saturating_add(b_text.encode_utf16().count()),
            )?;
            let id = format!("id_{a_text}_{b_text}_{n}");
            let a = s.field(&relationship, "entityA")?;
            let start = match moved.get(&self.atom_key(&a)?) {
                Some(value) if !matches!(value, Atom::Null | Atom::Undefined) => value.clone(),
                _ => s.field(&relationship, "entityA")?,
            };
            let b = s.field(&relationship, "entityB")?;
            let end = match moved.get(&self.atom_key(&b)?) {
                Some(value) if !matches!(value, Atom::Null | Atom::Undefined) => value.clone(),
                _ => s.field(&relationship, "entityB")?,
            };
            let label = s.field(&relationship, "roleA")?;
            let card_b =
                self.field_text(&s.field(&relationship, "relSpec")?, "cardB", "toLowerCase")?;
            let card_a =
                self.field_text(&s.field(&relationship, "relSpec")?, "cardA", "toLowerCase")?;
            let rel_type = s.field(&s.field(&relationship, "relSpec")?, "relType")?;
            let dashed = !matches!(&rel_type, Atom::String(_))
                || !s.string_units(&rel_type)?.is("IDENTIFYING");
            edges.push(Edge {
                id,
                start: HostScalar::Captured(start),
                end: HostScalar::Captured(end),
                label: HostScalar::Captured(label),
                arrow_start: Cow::Owned(card_b.to_lowercase()),
                arrow_end: Cow::Owned(card_a.to_lowercase()),
                dashed,
            });
        }
        Ok((nodes, edges))
    }
    fn direction(&self) -> Result<Cow<'_, str>, PeleError> {
        self.atom_direction(&self.store.field(&self.root, "direction")?)
            .map(Cow::Borrowed)
    }
    fn title(&self) -> Result<Option<Cow<'_, str>>, PeleError> {
        self.optional_root("title", false, "label")
    }
    fn acc_title(&self) -> Result<Option<Cow<'_, str>>, PeleError> {
        let title = self.optional_root("accTitle", true, "accessibility")?;
        // The pinned root decodes accTitle before reading accDescr. The shared
        // SVG writer accepts both strings at once, so validate the fallible
        // entity case here without altering spelling or decoding twice in the
        // emitted value. Plain metadata incurs no extra decode allocation.
        if let Some(value) = &title {
            if value.contains('&') || value.contains("¶ß") {
                crate::compat::source::decode_entities(value)?;
            }
        }
        Ok(title)
    }
    fn acc_descr(&self) -> Result<Option<Cow<'_, str>>, PeleError> {
        self.optional_root("accDescr", true, "accessibility")
    }
    fn styles<'a>(&'a self, styles: &'a Atom) -> Result<Cow<'a, [String]>, PeleError> {
        self.text_array(styles, "indexOf", false).map(Cow::Owned)
    }
    fn attribute_len(&self, attributes: &Atom) -> Result<usize, PeleError> {
        self.store.array_length(attributes)
    }
    fn attribute_at(&self, attributes: &Atom, index: usize) -> Result<Option<Atom>, PeleError> {
        // Array iterator length/index are fetched anew after the previous row's
        // callbacks; the returned row identity itself remains captured.
        if index >= self.store.array_length(attributes)? {
            return Ok(None);
        }
        self.store.charge(1, 0)?;
        self.store.array_get(attributes, index).map(Some)
    }
    fn cell<'a>(&'a self, row: &'a Atom, field: CellField) -> Result<Cow<'a, str>, PeleError> {
        self.field_text(row, field.key(), "includes")
            .map(Cow::Owned)
    }
    fn keys(&self, row: &Atom) -> Result<String, PeleError> {
        let keys = self.store.field(row, "keys")?;
        let text = self.store.join(&keys, ", ")?;
        self.store.charge(0, text.encode_utf16().count())?;
        Ok(text)
    }
}

/// Browser-only hidden entry. Existing native/source presentation stays on the
/// typed adapter; the ordinary web facade exposes no presentation-capture API.
#[cfg(feature = "web-host-er")]
pub fn render_store<S: RenderStore + ?Sized>(
    store: &S,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    if resources.presentation_budget().is_some() {
        return Err(PeleError::new(
            "ER host rendering does not expose native presentation capture",
            ErrorCode::Semantic,
        ));
    }
    render_owned_store(store, config, options, resources)
}
/// Internal native entry. Uses the existing renderer's presentation policy;
/// the old browser export above still refuses native presentation capture.
pub(super) fn render_owned_store<S: RenderStore + ?Sized>(
    store: &S,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PeleError> {
    let input = HostRead {
        store,
        root: store.root(),
    };
    super::render::render_read(&input, config, options, resources)
}
