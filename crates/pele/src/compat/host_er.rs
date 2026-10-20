//! Hidden, opt-in WASM ER storage boundary. No JavaScript values enter core.
pub use super::diagrams::er::{InitialWriter, MethodStore, RenderStore};
pub enum Parsed<T> {
    Host { policy: Policy, output: T },
    Arena(super::public::PublicDocument),
}
use super::{
    diagrams::er,
    public::{Atom, JsString, PublicError},
    ErrorCode, ModuleDescriptor, Object, OutputLimit, PeleError, RenderOptions, RenderResources,
    RenderResult, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};

pub fn number_string(value: f64) -> String {
    super::svg::js_num(value)
}
pub struct Policy {
    descriptor: ModuleDescriptor,
    config: Object,
}
impl Policy {
    pub(crate) fn captured(descriptor: ModuleDescriptor, config: Object) -> Self {
        Self { descriptor, config }
    }
    pub fn config(&self) -> &Object {
        &self.config
    }
    pub fn render(
        &self,
        store: &impl RenderStore,
        options: &RenderOptions,
        limit: Option<OutputLimit>,
        resources: &mut dyn RenderResources,
    ) -> Result<RenderResult, PeleError> {
        super::engine::render_registered_policy(
            &self.descriptor,
            &self.config,
            options,
            resources,
            limit,
            true,
            |settings, session| er::render_store(store, &self.config, settings, session),
        )
    }
    pub fn render_raw(
        &self,
        store: &impl RenderStore,
        config: &Object,
        settings: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        super::engine::render_raw_registered(&self.descriptor, settings, resources, |session| {
            er::render_store(store, config, settings, session)
        })
    }
}
// Validation-only equivalent of the existing typed importer. Implementations
// inspect own data and immutable UTF16; they do not cache an imported ErModel.
pub trait AdmissionStore {
    fn root(&self) -> Atom;
    fn own(&self, value: &Atom, key: &str) -> Result<Option<Atom>, PeleError>;
    fn utf16(&self, value: &Atom) -> Result<JsString, PeleError>;
    fn array(&self, value: &Atom) -> Result<(usize, usize), PeleError>;
    fn index(&self, value: &Atom, index: usize) -> Result<Atom, PeleError>;
    fn map_len(&self, value: &Atom) -> Result<usize, PeleError>;
    fn map(&self, value: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError>;
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("er".into());
    e
}
fn limit() -> PeleError {
    PeleError::new(
        "ER native graph operation exceeds work budget",
        ErrorCode::Limit,
    )
}
struct Admission<'a, S> {
    store: &'a S,
    work: usize,
    bytes: usize,
}
impl<S: AdmissionStore> Admission<'_, S> {
    fn required(&self, a: &Atom, key: &str) -> Result<Atom, PeleError> {
        self.store
            .own(a, key)?
            .ok_or_else(|| invalid(&format!("ER drawable field {key} is missing")))
    }
    fn text(&mut self, a: &Atom) -> Result<String, PeleError> {
        let units = self.store.utf16(a)?;
        self.bytes = self.bytes.saturating_add(units.0.len().saturating_mul(3));
        if self.bytes > 128 * 1024 * 1024 {
            return Err(limit());
        }
        units
            .well_formed()
            .ok_or_else(|| invalid("ER rendering of lone UTF-16 surrogates is not implemented"))
    }
    fn field(&mut self, a: &Atom, key: &str) -> Result<String, PeleError> {
        self.text(&self.required(a, key)?)
    }
    fn optional(&mut self, a: &Atom, key: &str) -> Result<(), PeleError> {
        match self.store.own(a, key)? {
            None | Some(Atom::Undefined | Atom::Null) => Ok(()),
            Some(value) => self.text(&value).map(|_| ()),
        }
    }
    fn array(&mut self, a: &Atom) -> Result<Vec<Atom>, PeleError> {
        let (length, properties) = self.store.array(a)?;
        self.work = self.work.saturating_add(length).saturating_add(properties);
        if self.work > 1_000_000 {
            return Err(limit());
        }
        (0..length).map(|i| self.store.index(a, i)).collect()
    }
    fn texts(&mut self, a: &Atom) -> Result<Vec<String>, PeleError> {
        self.array(a)?.iter().map(|v| self.text(v)).collect()
    }
    fn map(&mut self, a: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
        let length = self.store.map_len(a)?;
        self.work = self.work.saturating_add(length);
        if self.work > 1_000_000 {
            return Err(limit());
        }
        self.store.map(a)
    }
}
pub fn validate_render(store: &impl AdmissionStore) -> Result<(), PeleError> {
    let mut i = Admission {
        store,
        work: 0,
        bytes: 0,
    };
    let root = store.root();
    i.field(&root, "direction")?;
    for key in ["title", "accTitle", "accDescr"] {
        i.optional(&root, key)?;
    }
    let mut used = HashSet::new();
    let mut groups = HashSet::new();
    let mut class_occurrences = vec![];
    for a in i.array(&i.required(&root, "subgraphs")?)? {
        let classes = i.texts(&i.required(&a, "classes")?)?;
        used.extend(classes.iter().cloned());
        groups.insert(i.field(&a, "id")?);
        i.texts(&i.required(&a, "nodes")?)?;
        i.field(&a, "title")?;
        i.texts(&i.required(&a, "cssStyles")?)?;
        i.optional(&a, "dir")?;
        class_occurrences.extend(classes);
    }
    for (key, a) in i.map(&i.required(&root, "entities")?)? {
        let name = i.text(&key)?;
        if groups.contains(&name) {
            i.field(&a, "id")?;
            continue;
        }
        let classes = i.field(&a, "cssClasses")?;
        used.extend(classes.split(' ').map(str::to_owned));
        for attr in i.array(&i.required(&a, "attributes")?)? {
            i.field(&attr, "type")?;
            i.field(&attr, "name")?;
            i.texts(&i.required(&attr, "keys")?)?;
            i.field(&attr, "comment")?;
        }
        i.field(&a, "id")?;
        i.field(&a, "label")?;
        i.field(&a, "alias")?;
        i.texts(&i.required(&a, "cssStyles")?)?;
        class_occurrences.extend(classes.split(' ').map(str::to_owned));
    }
    let mut class_sizes = HashMap::new();
    let mut class_count = 0usize;
    for (key, a) in i.map(&i.required(&root, "classes")?)? {
        if !matches!(key, Atom::String(_)) {
            continue;
        }
        let units = store.utf16(&key)?;
        let Some(key) = units.well_formed() else {
            continue;
        };
        if !used.contains(&key) {
            continue;
        }
        let styles = i.texts(&i.required(&a, "styles")?)?;
        let text = i.texts(&i.required(&a, "textStyles")?)?;
        let bytes = styles.iter().chain(&text).fold(0usize, |n, s| {
            n.saturating_add(s.len())
                .saturating_add(std::mem::size_of::<String>())
        });
        class_sizes.insert(key, (styles.len() + text.len(), bytes));
        class_count += 1;
    }
    for a in i.array(&i.required(&root, "relationships")?)? {
        let spec = i.required(&a, "relSpec")?;
        for key in ["cardA", "cardB"] {
            if !matches!(
                i.field(&spec, key)?.to_ascii_uppercase().as_str(),
                "ZERO_OR_ONE" | "ZERO_OR_MORE" | "ONE_OR_MORE" | "ONLY_ONE" | "MD_PARENT"
            ) {
                return Err(invalid(
                    "ER unknown cardinality marker is not representable in the typed renderer",
                ));
            }
        }
        i.required(&spec, "relType")?;
        i.field(&a, "entityA")?;
        i.field(&a, "roleA")?;
        i.field(&a, "entityB")?;
    }
    for class in class_occurrences {
        i.work = i.work.saturating_add(class_count);
        if let Some((count, bytes)) = class_sizes.get(&class) {
            i.work = i.work.saturating_add(*count);
            i.bytes = i.bytes.saturating_add(*bytes);
        }
        if i.work > 1_000_000 || i.bytes > 128 * 1024 * 1024 {
            return Err(limit());
        }
    }
    Ok(())
}
pub fn invoke(
    store: &mut impl MethodStore,
    method: u32,
    receiver: Atom,
    arguments: Vec<Atom>,
) -> Result<Atom, PublicError> {
    if !(1..=10).contains(&method) {
        return Err(
            PeleError::new("Public model method is unavailable", ErrorCode::Semantic).into(),
        );
    }
    if arguments.len() > 1_000_000 {
        return Err(PeleError::new(
            "Public method arguments exceed work budget",
            ErrorCode::Limit,
        )
        .into());
    }
    store.invoke(super::public::MethodCall {
        method: super::public::MethodId(method),
        receiver,
        arguments,
    })
}
