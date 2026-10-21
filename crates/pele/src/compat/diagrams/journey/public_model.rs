//! JourneyDb's real builder/getActors prototype, Pele 458322a5 (MIT, kepano).
//! Exotic coercion/properties and active UTF-16 rendering remain explicit gaps.
use super::{Journey, JourneyModel, JourneyTask};
use crate::compat::{
    public::*, source::space, DiagramModule, DiagramType, ErrorCode, Object, PeleError,
    RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
const PROTO: PrototypeId = PrototypeId(1);
const SET_TITLE: MethodId = MethodId(0);
const SET_ACC_TITLE: MethodId = MethodId(1);
const SET_ACC_DESCR: MethodId = MethodId(2);
const ADD_SECTION: MethodId = MethodId(3);
const ADD_TASK: MethodId = MethodId(4);
const GET_ACTORS: MethodId = MethodId(5);
const fn field_schema(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        optional: false,
        undefined,
        nullable: false,
        flags: 7,
    }
}
const fn member(name: &'static str, method: MethodId) -> MemberSchema {
    MemberSchema {
        name,
        flags: 6,
        member: MemberKind::Method { method },
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/journey@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "Journey",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                field_schema("type", FieldKind::String, false),
                field_schema("title", FieldKind::String, true),
                field_schema("accTitle", FieldKind::String, true),
                field_schema("accDescr", FieldKind::String, true),
                field_schema("sections", FieldKind::Array, false),
                field_schema("tasks", FieldKind::Array, false),
                field_schema("currentSection", FieldKind::String, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "JourneyTask",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("task", FieldKind::String, false),
                field_schema("score", FieldKind::Number, false),
                field_schema("people", FieldKind::Array, false),
                field_schema("section", FieldKind::String, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "JourneyDb",
        parent: PrototypeParent::Object,
        members: &[
            member("setDiagramTitle", SET_TITLE),
            member("setAccTitle", SET_ACC_TITLE),
            member("setAccDescription", SET_ACC_DESCR),
            member("addSection", ADD_SECTION),
            member("addTask", ADD_TASK),
            member("getActors", GET_ACTORS),
        ],
    }],
    methods: &[
        MethodSchema {
            id: SET_TITLE,
            name: "setDiagramTitle",
            arity: 1,
        },
        MethodSchema {
            id: SET_ACC_TITLE,
            name: "setAccTitle",
            arity: 1,
        },
        MethodSchema {
            id: SET_ACC_DESCR,
            name: "setAccDescription",
            arity: 1,
        },
        MethodSchema {
            id: ADD_SECTION,
            name: "addSection",
            arity: 1,
        },
        MethodSchema {
            id: ADD_TASK,
            name: "addTask",
            arity: 2,
        },
        MethodSchema {
            id: GET_ACTORS,
            name: "getActors",
            arity: 0,
        },
    ],
};
pub struct JourneyPublicState {
    graph: ModelGraph,
    model: Result<JourneyModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("journey".into());
    e
}
fn limit() -> PeleError {
    let mut e = PeleError::new(
        "Journey public operation exceeds native work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("journey".into());
    e
}
fn js_error(name: JavaScriptErrorKind, message: impl Into<String>) -> PublicError {
    PublicError::JavaScript {
        name,
        message: JsString::from(message.into().as_str()),
    }
}
fn type_error(message: impl Into<String>) -> PublicError {
    js_error(JavaScriptErrorKind::TypeError, message)
}
fn value<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn root(g: &ModelGraph) -> Result<u32, PeleError> {
    if let Atom::Ref(id) = g.root {
        Ok(id)
    } else {
        Err(invalid("Journey root must be an object"))
    }
}
fn object(a: Option<&Atom>) -> Result<u32, PeleError> {
    if let Some(Atom::Ref(id)) = a {
        Ok(*id)
    } else {
        Err(invalid("Journey field must be an object"))
    }
}
fn intern(g: &mut ModelGraph, text: JsString) -> Result<u32, PublicError> {
    if let Some(i) = g.strings.iter().position(|v| v == &text) {
        return Ok(i as u32);
    }
    if g.strings.len() >= 1_000_000 || text.0.len() > 16_000_000 {
        return Err(limit().into());
    }
    let id = g.strings.len() as u32;
    g.strings.push(text);
    Ok(id)
}
fn node(g: &mut ModelGraph, node: Node) -> Result<u32, PublicError> {
    if g.nodes.len() >= 1_000_000 {
        return Err(limit().into());
    }
    let id = g.nodes.len() as u32;
    g.nodes.push(node);
    Ok(id)
}
fn assign(g: &mut ModelGraph, id: u32, key: &str, atom: Atom) -> Result<(), PublicError> {
    let index = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key));
    if let Some(i) = index {
        let p = &mut g.nodes[id as usize].properties_mut()[i];
        if p.flags & 2 == 0 {
            return Err(type_error(format!(
                "Cannot assign to read only property '{key}'"
            )));
        }
        p.value = atom;
    } else {
        let key = intern(g, JsString::from(key))?;
        g.nodes[id as usize].properties_mut().push(Property {
            key,
            value: atom,
            flags: 7,
        });
    }
    Ok(())
}
fn receiver(a: &Atom, key: &str, write: bool) -> Result<u32, PublicError> {
    match a {
        Atom::Ref(id) => Ok(*id),
        Atom::Undefined | Atom::Null => Err(type_error(format!(
            "Cannot {} properties of {} ({} '{key}')",
            if write { "set" } else { "read" },
            if matches!(a, Atom::Null) {
                "null"
            } else {
                "undefined"
            },
            if write { "setting" } else { "reading" }
        ))),
        Atom::Number(n) if write => Err(type_error(format!(
            "Cannot create property '{key}' on number '{}'",
            number_text(*n)
        ))),
        _ => Err(invalid("Journey borrowed primitive receiver coercion is not implemented").into()),
    }
}
fn units(g: &ModelGraph, a: &Atom, method: &str, param: &str) -> Result<Vec<u16>, PublicError> {
    match a {
        Atom::String(id) => Ok(g.strings[*id as usize].0.clone()),
        Atom::Null | Atom::Undefined => Err(type_error(format!(
            "Cannot read properties of {} (reading '{method}')",
            if matches!(a, Atom::Null) {
                "null"
            } else {
                "undefined"
            }
        ))),
        _ => Err(type_error(format!("{param}.{method} is not a function"))),
    }
}
fn ws(unit: u16) -> bool {
    char::from_u32(u32::from(unit)).is_some_and(space)
}
fn trim(mut units: &[u16]) -> &[u16] {
    while units.first().is_some_and(|v| ws(*v)) {
        units = &units[1..]
    }
    while units.last().is_some_and(|v| ws(*v)) {
        units = &units[..units.len() - 1]
    }
    units
}
fn push(g: &mut ModelGraph, receiver: u32, key: &str, atom: Atom) -> Result<(), PublicError> {
    let target = value(g, receiver, key).cloned().unwrap_or(Atom::Undefined);
    let id = match target {
        Atom::Ref(id) => id,
        Atom::Null | Atom::Undefined => {
            return Err(type_error(format!(
                "Cannot read properties of {} (reading 'push')",
                if matches!(target, Atom::Null) {
                    "null"
                } else {
                    "undefined"
                }
            )))
        }
        _ => return Err(type_error(format!("this.{key}.push is not a function"))),
    };
    let length = if let Node::Array { length, .. } = g.nodes[id as usize] {
        length
    } else {
        return Err(type_error(format!("this.{key}.push is not a function")));
    };
    assign(g, id, &length.to_string(), atom)?;
    let Some(next) = length.checked_add(1) else {
        return Err(js_error(
            JavaScriptErrorKind::RangeError,
            "Invalid array length",
        ));
    };
    if let Node::Array { length, .. } = &mut g.nodes[id as usize] {
        *length = next
    }
    Ok(())
}
fn array(g: &ModelGraph, id: u32, holes: bool) -> Result<Vec<Atom>, PeleError> {
    let Some(Node::Array { length, properties }) = g.nodes.get(id as usize) else {
        return Err(invalid("Journey collections must be Arrays"));
    };
    if *length > 1_000_000 {
        return Err(limit());
    }
    let mut index = HashMap::new();
    for p in properties {
        if let Some(text) = g.strings[p.key as usize].well_formed() {
            if let Ok(n) = text.parse::<u32>() {
                if n.to_string() == text && n < *length {
                    index.entry(n).or_insert_with(|| p.value.clone());
                }
            }
        }
    }
    if !holes && index.len() != *length as usize {
        return Err(invalid("Journey sparse active arrays are not implemented"));
    }
    Ok((0..*length)
        .map(|i| index.get(&i).cloned().unwrap_or(Atom::Undefined))
        .collect())
}
struct Text(usize);
impl Text {
    fn read(
        &mut self,
        g: &ModelGraph,
        a: Option<&Atom>,
        undefined: bool,
    ) -> Result<String, PeleError> {
        if undefined && matches!(a, None | Some(Atom::Undefined)) {
            return Ok(String::new());
        }
        let Some(a) = a else {
            return Err(invalid("Journey active text field is absent"));
        };
        let value = g.text(a)?;
        self.0 = self.0.checked_add(value.0.len()).ok_or_else(limit)?;
        if self.0 > 16_000_000 {
            return Err(limit());
        }
        value.well_formed().ok_or_else(|| {
            invalid("Journey rendering of lone UTF-16 surrogates is not implemented")
        })
    }
    fn metadata(&mut self, g: &ModelGraph, key: &str) -> Result<Option<String>, PeleError> {
        match g.field(key)? {
            None | Some(Atom::Undefined) => Ok(None),
            Some(Atom::Null | Atom::Bool(false)) if key != "title" => Ok(None),
            Some(Atom::Number(n)) if key != "title" && (*n == 0. || n.is_nan()) => Ok(None),
            Some(a) => self.read(g, Some(a), false).map(Some),
        }
    }
}
fn imported(g: &ModelGraph) -> Result<JourneyModel, PeleError> {
    let r = root(g)?;
    let mut text = Text(0);
    // Render never reads sections/currentSection; keep their complete graph
    // values without turning bookkeeping mutations into rendering failures.
    let sections = vec![];
    let mut tasks = vec![];
    for task in array(g, object(value(g, r, "tasks"))?, false)? {
        let id = object(Some(&task))?;
        let mut people = vec![];
        for name in array(g, object(value(g, id, "people"))?, false)? {
            people.push(text.read(g, Some(&name), false)?)
        }
        let score = match value(g, id, "score") {
            Some(Atom::Number(n)) => *n,
            _ => {
                return Err(invalid(
                    "Journey rendering of non-number scores is not implemented",
                ))
            }
        };
        tasks.push(JourneyTask {
            task: text.read(g, value(g, id, "task"), false)?,
            score,
            people,
            section: text.read(g, value(g, id, "section"), false)?,
        });
    }
    let current_section = match value(g, r, "currentSection") {
        Some(Atom::String(id)) => g.strings[*id as usize].well_formed().unwrap_or_default(),
        _ => String::new(),
    };
    Ok(JourneyModel {
        diagram_type: DiagramType::Journey,
        title: text.metadata(g, "title")?,
        acc_title: text.metadata(g, "accTitle")?,
        acc_descr: text.metadata(g, "accDescr")?,
        sections,
        tasks,
        current_section,
    })
}
fn number_text(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n == f64::INFINITY {
        "Infinity".into()
    } else if n == f64::NEG_INFINITY {
        "-Infinity".into()
    } else if n == 0. {
        "0".into()
    } else {
        ryu_js::Buffer::new().format(n).into()
    }
}
#[derive(Hash, Eq, PartialEq)]
enum Name {
    Undefined,
    Null,
    Bool(bool),
    Number(u64),
    String(JsString),
    Ref(u32),
    Function(u32),
    Host(u32),
}
fn iterable(g: &mut ModelGraph, atom: Atom, message: &str) -> Result<Vec<Atom>, PublicError> {
    match atom {
        Atom::String(id) => {
            // String iteration uses code points, preserving unmatched UTF-16 units.
            let units = g.strings[id as usize].0.clone();
            if units.len() > 1_000_000 {
                return Err(limit().into());
            }
            let mut out = Vec::new();
            let mut i = 0;
            while i < units.len() {
                let pair = (0xd800..=0xdbff).contains(&units[i])
                    && units
                        .get(i + 1)
                        .is_some_and(|u| (0xdc00..=0xdfff).contains(u));
                let end = i + if pair { 2 } else { 1 };
                out.push(Atom::String(intern(g, JsString(units[i..end].to_vec()))?));
                i = end;
            }
            Ok(out)
        }
        Atom::Ref(id) => match &g.nodes[id as usize] {
            Node::Array { .. } => array(g, id, true).map_err(Into::into),
            Node::Set { values, .. } => Ok(values.clone()),
            Node::Map { entries, .. } => {
                let entries = entries.clone();
                let mut out = Vec::new();
                for (key, value) in entries {
                    let zero = intern(g, JsString::from("0"))?;
                    let one = intern(g, JsString::from("1"))?;
                    out.push(Atom::Ref(node(
                        g,
                        Node::Array {
                            length: 2,
                            properties: vec![
                                Property {
                                    key: zero,
                                    value: key,
                                    flags: 7,
                                },
                                Property {
                                    key: one,
                                    value,
                                    flags: 7,
                                },
                            ],
                        },
                    )?));
                }
                Ok(out)
            }
            _ => Err(type_error(message)),
        },
        _ => Err(type_error(message)),
    }
}
fn get_actors(g: &mut ModelGraph, receiver: u32) -> Result<Atom, PublicError> {
    let tasks = value(g, receiver, "tasks")
        .cloned()
        .unwrap_or(Atom::Undefined);
    let tasks = iterable(g, tasks, "e is not iterable")?;
    let mut seen = HashSet::new();
    let mut names = vec![];
    for task in tasks {
        let id = match task {
            Atom::Ref(id) => id,
            Atom::Null | Atom::Undefined => {
                return Err(type_error(format!(
                    "Cannot read properties of {} (reading 'people')",
                    if matches!(task, Atom::Null) {
                        "null"
                    } else {
                        "undefined"
                    }
                )))
            }
            _ => return Err(type_error("n.people is not iterable")),
        };
        let people = value(g, id, "people").cloned().unwrap_or(Atom::Undefined);
        let people = iterable(g, people, "n.people is not iterable")?;
        for mut name in people {
            let key = match &name {
                Atom::Undefined => Name::Undefined,
                Atom::Null => Name::Null,
                Atom::Bool(v) => Name::Bool(*v),
                Atom::Number(v) => {
                    if *v == 0. {
                        name = Atom::Number(0.);
                        Name::Number(0)
                    } else {
                        Name::Number(if v.is_nan() {
                            f64::NAN.to_bits()
                        } else {
                            v.to_bits()
                        })
                    }
                }
                Atom::String(id) => Name::String(g.strings[*id as usize].clone()),
                Atom::Ref(id) => Name::Ref(*id),
                Atom::BuiltinFunction(id) => Name::Function(*id),
                Atom::HostReference(id) => Name::Host(*id),
            };
            if seen.insert(key) {
                names.push(name)
            }
            if names.len() > 1_000_000 {
                return Err(limit().into());
            }
        }
    }
    let mut sortable = Vec::with_capacity(names.len());
    for name in names {
        let text = match &name {
            Atom::String(id) => g.strings[*id as usize].0.clone(),
            Atom::Number(n) => number_text(*n).encode_utf16().collect(),
            Atom::Bool(b) => b.to_string().encode_utf16().collect(),
            Atom::Null => "null".encode_utf16().collect(),
            Atom::Undefined => vec![],
            _ => {
                return Err(invalid(
                    "Journey actor sorting of arbitrary objects is not implemented",
                )
                .into())
            }
        };
        sortable.push((name, text));
    }
    sortable.sort_by(|(a, x), (b, y)| {
        match (matches!(a, Atom::Undefined), matches!(b, Atom::Undefined)) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => x.cmp(y),
        }
    });
    let mut properties = vec![];
    for (i, (value, _)) in sortable.into_iter().enumerate() {
        let key = intern(g, JsString::from(i.to_string().as_str()))?;
        properties.push(Property {
            key,
            value,
            flags: 7,
        })
    }
    let length = properties.len() as u32;
    Ok(Atom::Ref(node(g, Node::Array { length, properties })?))
}
fn invoke(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let arg = |i| call.arguments.get(i).cloned().unwrap_or(Atom::Undefined);
    match call.method {
        SET_TITLE => {
            let r = receiver(&call.receiver, "title", true)?;
            assign(g, r, "title", arg(0))?;
        }
        SET_ACC_TITLE | SET_ACC_DESCR => {
            let raw = units(g, &arg(0), "replace", "t")?;
            let value = if call.method == SET_ACC_TITLE {
                let mut start = 0;
                while raw.get(start).is_some_and(|v| ws(*v)) {
                    start += 1
                }
                raw[start..].to_vec()
            } else {
                let mut out = vec![];
                let mut i = 0;
                while i < raw.len() {
                    let unit = raw[i];
                    out.push(unit);
                    i += 1;
                    if unit == 10 {
                        while raw.get(i).is_some_and(|v| ws(*v)) {
                            i += 1
                        }
                    }
                }
                out
            };
            let value = Atom::String(intern(g, JsString(value))?);
            let key = if call.method == SET_ACC_TITLE {
                "accTitle"
            } else {
                "accDescr"
            };
            let r = receiver(&call.receiver, key, true)?;
            assign(g, r, key, value)?;
        }
        ADD_SECTION => {
            let r = receiver(&call.receiver, "currentSection", true)?;
            let value = arg(0);
            assign(g, r, "currentSection", value.clone())?;
            push(g, r, "sections", value)?;
        }
        ADD_TASK => {
            let raw = units(g, &arg(1), "substring", "n")?;
            let pieces: Vec<_> = raw.get(1..).unwrap_or(&[]).split(|v| *v == 58).collect();
            let score =
                String::from_utf16(pieces[0]).map_or(f64::NAN, |s| super::parser::js_number(&s));
            let mut properties = vec![];
            if let Some(raw) = pieces.get(1) {
                for (i, name) in raw.split(|v| *v == 44).enumerate() {
                    if i >= 1_000_000 {
                        return Err(limit().into());
                    }
                    let key = intern(g, JsString::from(i.to_string().as_str()))?;
                    let value = Atom::String(intern(g, JsString(trim(name).to_vec()))?);
                    properties.push(Property {
                        key,
                        value,
                        flags: 7,
                    })
                }
            }
            let length = properties.len() as u32;
            let people = Atom::Ref(node(g, Node::Array { length, properties })?);
            let r = receiver(&call.receiver, "tasks", false)?;
            let section = value(g, r, "currentSection")
                .cloned()
                .unwrap_or(Atom::Undefined);
            let mut properties = vec![];
            for (key, value) in [
                ("task", arg(0)),
                ("score", Atom::Number(score)),
                ("people", people),
                ("section", section),
            ] {
                let key = intern(g, JsString::from(key))?;
                properties.push(Property {
                    key,
                    value,
                    flags: 7,
                })
            }
            let task = Atom::Ref(node(g, Node::Object { properties })?);
            push(g, r, "tasks", task)?;
        }
        GET_ACTORS => {
            let r = receiver(&call.receiver, "tasks", false)?;
            return get_actors(g, r);
        }
        _ => return Err(invalid("Unknown Journey method").into()),
    }
    Ok(Atom::Undefined)
}
fn array_node(out: &mut GraphWriter, values: impl IntoIterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(i, v)| out.property(&i.to_string(), v))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn construct_default() -> Result<JourneyPublicState, PeleError> {
    Journey.create_public(&JourneyModel::empty(None))
}

impl PublicDiagramModule for Journey {
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<crate::compat::public::PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTO).then(|| {
            crate::compat::public::PublicConstructorFactory::ignoring_arguments::<Self>(
                PROTO,
                construct_default,
            )
        })
    }

    type PublicState = JourneyPublicState;
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::journey())
    }
    fn detached_public_render(
        &self,
        state: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::journey(
            super::JourneyDetachedInput::new(state.model.as_ref().err()),
        ))
    }

    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(JourneyPublicState { graph, model })
    }
    fn create_public(&self, m: &JourneyModel) -> Result<JourneyPublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut sections = vec![];
        for s in &m.sections {
            sections.push(Atom::String(out.string(s.as_str())))
        }
        let sections = array_node(&mut out, sections);
        let mut tasks = vec![];
        for task in &m.tasks {
            let name = out.string(task.task.as_str());
            let section = out.string(task.section.as_str());
            let mut people = vec![];
            for p in &task.people {
                people.push(Atom::String(out.string(p.as_str())))
            }
            let people = array_node(&mut out, people);
            let properties = vec![
                out.property("task", Atom::String(name)),
                out.property("score", Atom::Number(task.score)),
                out.property("people", Atom::Ref(people)),
                out.property("section", Atom::String(section)),
            ];
            tasks.push(Atom::Ref(out.node(Node::Object { properties })))
        }
        let tasks = array_node(&mut out, tasks);
        let kind = out.string("journey");
        let mut properties = vec![out.property("type", Atom::String(kind))];
        for (key, value) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let value = value
                .as_deref()
                .map_or(Atom::Undefined, |v| Atom::String(out.string(v)));
            properties.push(out.property(key, value))
        }
        let current = out.string(m.current_section.as_str());
        properties.extend([
            out.property("sections", Atom::Ref(sections)),
            out.property("tasks", Atom::Ref(tasks)),
            out.property("currentSection", Atom::String(current)),
        ]);
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let graph = out.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(JourneyPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &JourneyPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut JourneyPublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut JourneyPublicState,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke(&mut s.graph, call);
        s.model = imported(&s.graph);
        result
    }
    fn render_public(
        &self,
        s: &mut JourneyPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
