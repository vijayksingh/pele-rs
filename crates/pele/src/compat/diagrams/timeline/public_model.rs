//! Mutable TimelineDb records and native builders, Pele 458322a5 (MIT, kepano).
use super::{Timeline, TimelineModel, TimelinePeriod};
use crate::compat::{
    public::*, source::space, DiagramModule, DiagramType, ErrorCode, Object, PeleError,
    RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
const MAX_ITEMS: usize = 1_000_000;
const MAX_WORK: usize = 8_000_000;
const MAX_TEXT: usize = 16_000_000;
const PROTOTYPE: PrototypeId = PrototypeId(1);
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
const fn method(id: u32, name: &'static str, arity: u32) -> MethodSchema {
    MethodSchema {
        id: MethodId(id),
        name,
        arity,
    }
}
const fn member(id: u32, name: &'static str) -> MemberSchema {
    MemberSchema {
        name,
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(id),
        },
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/timeline@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "TimelineModel",
            kind: RecordKind::Object,
            prototype: Some(PROTOTYPE),
            fields: &[
                field_schema("type", FieldKind::String, false),
                field_schema("title", FieldKind::String, true),
                field_schema("accTitle", FieldKind::String, true),
                field_schema("accDescr", FieldKind::String, true),
                field_schema("direction", FieldKind::String, false),
                field_schema("sections", FieldKind::Array, false),
                field_schema("periods", FieldKind::Array, false),
                field_schema("currentSection", FieldKind::String, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "TimelinePeriod",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("text", FieldKind::String, false),
                field_schema("events", FieldKind::Array, false),
                field_schema("section", FieldKind::String, false),
                field_schema("sectionIndex", FieldKind::Number, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTOTYPE,
        name: "TimelineDb",
        parent: PrototypeParent::Object,
        members: &[
            member(1, "getCommonDb"),
            member(2, "setDiagramTitle"),
            member(3, "setAccTitle"),
            member(4, "setAccDescription"),
            member(5, "setDirection"),
            member(6, "addSection"),
            member(7, "addTask"),
            member(8, "addEvent"),
        ],
    }],
    methods: &[
        method(1, "getCommonDb", 0),
        method(2, "setDiagramTitle", 1),
        method(3, "setAccTitle", 1),
        method(4, "setAccDescription", 1),
        method(5, "setDirection", 1),
        method(6, "addSection", 1),
        method(7, "addTask", 3),
        method(8, "addEvent", 1),
    ],
};
pub struct TimelinePublicState {
    graph: ModelGraph,
    imported: Result<TimelineModel, PeleError>,
}
fn invalid(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("timeline".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Timeline public model expansion exceeds the work budget");
    e.code = ErrorCode::Limit;
    e
}
fn type_error(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn reference(a: Option<&Atom>) -> Result<u32, PeleError> {
    if let Some(Atom::Ref(id)) = a {
        Ok(*id)
    } else {
        Err(invalid("Timeline records and collections must be objects"))
    }
}
fn method_object(a: Option<&Atom>) -> Result<u32, PublicError> {
    reference(a)
        .map_err(|_| type_error("Timeline method requires an object receiver or collection"))
}
fn truthy(g: &ModelGraph, a: Option<&Atom>) -> bool {
    match a {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => false,
        Some(Atom::Number(v)) => *v != 0. && !v.is_nan(),
        Some(Atom::String(i)) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
fn text(g: &ModelGraph, a: &Atom) -> Result<String, PeleError> {
    g.text(a)?
        .well_formed()
        .ok_or_else(|| invalid("Timeline rendering of lone UTF-16 surrogates is not implemented"))
}
fn metadata(g: &ModelGraph, a: Option<&Atom>) -> Result<Option<String>, PeleError> {
    match a {
        None | Some(Atom::Undefined) => Ok(None),
        Some(v) => text(g, v).map(Some),
    }
}
fn array<'a>(g: &'a ModelGraph, a: Option<&Atom>) -> Result<Vec<&'a Atom>, PeleError> {
    let id = reference(a)?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        return Err(invalid("Timeline collections must be Arrays"));
    };
    if *length as usize > MAX_ITEMS {
        return Err(limit());
    }
    let mut indexes = HashMap::new();
    for p in properties {
        let key = &g.strings[p.key as usize].0;
        if key.is_empty() || key.len() > 10 || (key.len() > 1 && key[0] == 48) {
            continue;
        }
        let mut n = 0u64;
        let mut canonical = true;
        for &u in key {
            if !(48..=57).contains(&u) {
                canonical = false;
                break;
            }
            n = n * 10 + u64::from(u - 48);
        }
        if canonical && n < u64::from(*length) {
            indexes.entry(n as u32).or_insert(&p.value);
        }
    }
    if indexes.len() != *length as usize {
        return Err(invalid(
            "Timeline collection contains a missing array record",
        ));
    }
    Ok((0..*length).map(|i| indexes[&i]).collect())
}
fn count_text(g: &ModelGraph, a: Option<&Atom>, units: &mut usize) -> Result<(), PeleError> {
    if let Some(Atom::String(i)) = a {
        *units = units
            .checked_add(g.strings[*i as usize].0.len())
            .ok_or_else(limit)?;
        if *units > MAX_TEXT {
            return Err(limit());
        }
    }
    Ok(())
}
fn imported(g: &ModelGraph) -> Result<TimelineModel, PeleError> {
    let root = reference(Some(&g.root))?;
    let sections = array(g, field(g, root, "sections"))?;
    let periods = array(g, field(g, root, "periods"))?;
    let mut units = 0usize;
    for key in ["title", "accTitle", "accDescr"] {
        count_text(g, field(g, root, key), &mut units)?
    }
    for s in &sections {
        count_text(g, Some(s), &mut units)?
    }
    let mut work = 0usize;
    for p in &periods {
        let p = reference(Some(p))?;
        count_text(g, field(g, p, "text"), &mut units)?;
        let events = array(g, field(g, p, "events"))?;
        work = work.checked_add(events.len()).ok_or_else(limit)?;
        if work > MAX_WORK {
            return Err(limit());
        }
        for e in events {
            count_text(g, Some(e), &mut units)?
        }
    }
    let sections = sections
        .into_iter()
        .map(|a| text(g, a))
        .collect::<Result<Vec<_>, _>>()?;
    let periods = periods
        .into_iter()
        .map(|p| {
            let p = reference(Some(p))?;
            let section_index = match field(g, p, "sectionIndex") {
                Some(Atom::Number(v))
                    if v.is_finite() && *v >= 0. && v.fract() == 0. && *v < MAX_ITEMS as f64 =>
                {
                    *v as i64
                }
                _ => -1,
            };
            Ok(TimelinePeriod {
                text: text(g, field(g, p, "text").unwrap_or(&Atom::Undefined))?,
                events: array(g, field(g, p, "events"))?
                    .into_iter()
                    .map(|a| {
                        if matches!(a, Atom::Undefined) {
                            Ok(String::new())
                        } else {
                            text(g, a)
                        }
                    })
                    .collect::<Result<Vec<_>, PeleError>>()?,
                // Section names and currentSection are builder bookkeeping, never read
                // by the renderer. The authoritative graph retains their exact atoms.
                section: String::new(),
                section_index,
            })
        })
        .collect::<Result<Vec<_>, PeleError>>()?;
    Ok(TimelineModel{diagram_type:DiagramType::Timeline,title:metadata(g,field(g,root,"title"))?,acc_title:metadata(g,field(g,root,"accTitle"))?,acc_descr:metadata(g,field(g,root,"accDescr"))?,direction:if matches!(field(g,root,"direction"),Some(Atom::String(i))if g.strings[*i as usize].is("TD")){"TD"}else{"LR"}.into(),sections,periods,current_section:String::new()})
}
fn array_node(out: &mut GraphWriter, values: impl Iterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .enumerate()
        .map(|(i, v)| out.property(&i.to_string(), v))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn string(out: &mut GraphWriter, s: &str) -> Atom {
    Atom::String(out.string(s))
}
fn graph_work(g: &ModelGraph) -> usize {
    g.nodes
        .iter()
        .map(|n| {
            n.properties().len()
                + match n {
                    Node::Map { entries, .. } => entries.len() * 2,
                    Node::Set { values, .. } => values.len(),
                    _ => 0,
                }
        })
        .sum()
}
fn intern(g: &mut ModelGraph, s: JsString) -> Result<u32, PublicError> {
    if let Some(i) = g.strings.iter().position(|v| *v == s) {
        return Ok(i as u32);
    }
    if g.strings.len() >= MAX_ITEMS
        || g.strings
            .iter()
            .map(|s| s.0.len())
            .sum::<usize>()
            .saturating_add(s.0.len())
            > MAX_TEXT
    {
        return Err(limit().into());
    }
    let id = g.strings.len() as u32;
    g.strings.push(s);
    Ok(id)
}
fn assign(g: &mut ModelGraph, id: u32, key: &str, v: Atom) -> Result<(), PublicError> {
    if let Some(i) = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key))
    {
        if g.nodes[id as usize].properties()[i].flags & 2 == 0 {
            return Err(type_error("Cannot assign to read only Timeline property"));
        }
        g.nodes[id as usize].properties_mut()[i].value = v;
        return Ok(());
    }
    if graph_work(g) >= MAX_WORK {
        return Err(limit().into());
    }
    let key = intern(g, key.into())?;
    g.nodes[id as usize].properties_mut().push(Property {
        key,
        value: v,
        flags: 7,
    });
    Ok(())
}
fn new_record(
    g: &mut ModelGraph,
    fields: Vec<(&str, Atom)>,
    length: Option<u32>,
) -> Result<u32, PublicError> {
    if g.nodes.len() >= MAX_ITEMS || graph_work(g).saturating_add(fields.len()) > MAX_WORK {
        return Err(limit().into());
    }
    let mut properties = Vec::with_capacity(fields.len());
    for (key, value) in fields {
        properties.push(Property {
            key: intern(g, key.into())?,
            value,
            flags: 7,
        });
    }
    let id = g.nodes.len() as u32;
    g.nodes.push(if let Some(length) = length {
        Node::Array { length, properties }
    } else {
        Node::Object { properties }
    });
    Ok(id)
}
fn push(g: &mut ModelGraph, id: u32, a: Atom) -> Result<(), PublicError> {
    if let Some(value) = field(g, id, "push") {
        return Err(
            if matches!(value, Atom::HostReference(_) | Atom::BuiltinFunction(_)) {
                invalid("Timeline overridden collection push dispatch is not implemented").into()
            } else {
                type_error("Timeline collection push is not a function")
            },
        );
    }
    let Node::Array { length, .. } = g.nodes[id as usize] else {
        return Err(type_error("Timeline collection push is not a function"));
    };
    if length as usize >= MAX_ITEMS {
        return Err(limit().into());
    }
    assign(g, id, &length.to_string(), a)?;
    let Node::Array { length: len, .. } = &mut g.nodes[id as usize] else {
        unreachable!()
    };
    *len = length + 1;
    Ok(())
}
fn argument(call: &MethodCall, i: usize) -> &Atom {
    call.arguments.get(i).unwrap_or(&Atom::Undefined)
}
fn invoke(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let method = call.method.0;
    if method == 1 {
        return Ok(call.receiver);
    }
    let root = method_object(Some(&call.receiver))?;
    match method {
        2 | 5 => {
            assign(
                g,
                root,
                if method == 2 { "title" } else { "direction" },
                argument(&call, 0).clone(),
            )?;
        }
        3 | 4 => {
            let Atom::String(i) = argument(&call, 0) else {
                return Err(type_error(
                    "Timeline accessibility argument must be a string",
                ));
            };
            let raw = &g.strings[*i as usize].0;
            let whitespace = |u: u16| char::from_u32(u as u32).is_some_and(space);
            let units = if method == 3 {
                raw.iter().copied().skip_while(|u| whitespace(*u)).collect()
            } else {
                let mut out = Vec::new();
                let mut after = false;
                for &u in raw {
                    if after && whitespace(u) {
                        continue;
                    }
                    out.push(u);
                    after = u == 10;
                }
                out
            };
            let i = intern(g, JsString(units))?;
            assign(
                g,
                root,
                if method == 3 { "accTitle" } else { "accDescr" },
                Atom::String(i),
            )?;
        }
        6 => {
            let text = argument(&call, 0).clone();
            assign(g, root, "currentSection", text.clone())?;
            let sections = method_object(field(g, root, "sections"))?;
            push(g, sections, text)?;
        }
        7 => {
            // _length is intentionally unread. Field insertion order and foreign
            // references are those of the actual four-field object literal.
            let text = argument(&call, 0).clone();
            let event = argument(&call, 2).clone();
            let has_event = truthy(g, Some(&event));
            let events = new_record(
                g,
                if has_event {
                    vec![("0", event)]
                } else {
                    vec![]
                },
                Some(u32::from(has_event)),
            )?;
            let section = field(g, root, "currentSection")
                .cloned()
                .unwrap_or(Atom::Undefined);
            let sections = method_object(field(g, root, "sections"))?;
            let length = match &g.nodes[sections as usize] {
                Node::Array { length, .. } => *length as f64,
                _ => return Err(type_error("Timeline sections must be an Array")),
            };
            let period = new_record(
                g,
                vec![
                    ("text", text),
                    ("events", Atom::Ref(events)),
                    ("section", section),
                    ("sectionIndex", Atom::Number(length - 1.)),
                ],
                None,
            )?;
            let periods = method_object(field(g, root, "periods"))?;
            push(g, periods, Atom::Ref(period))?;
        }
        8 => {
            let periods = method_object(field(g, root, "periods"))?;
            let Node::Array { length, .. } = g.nodes[periods as usize] else {
                return Err(type_error("Timeline periods must be an Array"));
            };
            let period = if length == 0 {
                None
            } else {
                field(g, periods, &(length - 1).to_string())
            };
            if !truthy(g, period) {
                return Err(invalid("An event needs a time period before it.").into());
            }
            let period = method_object(period)?;
            let events = method_object(field(g, period, "events"))?;
            push(g, events, argument(&call, 0).clone())?;
        }
        _ => return Err(invalid("Timeline public method is unavailable").into()),
    }
    Ok(Atom::Undefined)
}
impl PublicDiagramModule for Timeline {
    type PublicState = TimelinePublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let imported = imported(&graph);
        Ok(TimelinePublicState { graph, imported })
    }
    fn create_public(&self, m: &TimelineModel) -> Result<Self::PublicState, PeleError> {
        let mut out = GraphWriter::default();
        let values: Vec<_> = m.sections.iter().map(|s| string(&mut out, s)).collect();
        let sections = array_node(&mut out, values.into_iter());
        let mut periods = Vec::new();
        for p in &m.periods {
            let values: Vec<_> = p.events.iter().map(|s| string(&mut out, s)).collect();
            let events = array_node(&mut out, values.into_iter());
            let text = string(&mut out, &p.text);
            let section = string(&mut out, &p.section);
            let properties = vec![
                out.property("text", text),
                out.property("events", Atom::Ref(events)),
                out.property("section", section),
                out.property("sectionIndex", Atom::Number(p.section_index as f64)),
            ];
            periods.push(Atom::Ref(out.node(Node::Object { properties })));
        }
        let periods = array_node(&mut out, periods.into_iter());
        let kind = string(&mut out, "timeline");
        let mut properties = vec![out.property("type", kind)];
        for (key, v) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let a = v
                .as_deref()
                .map_or(Atom::Undefined, |s| string(&mut out, s));
            properties.push(out.property(key, a));
        }
        let direction = string(&mut out, &m.direction);
        let current = string(&mut out, &m.current_section);
        properties.extend([
            out.property("direction", direction),
            out.property("sections", Atom::Ref(sections)),
            out.property("periods", Atom::Ref(periods)),
            out.property("currentSection", current),
        ]);
        let root = out.node(Node::Instance {
            prototype: PROTOTYPE,
            properties,
        });
        let graph = out.finish(Atom::Ref(root))?;
        let imported = imported(&graph);
        Ok(TimelinePublicState { graph, imported })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.imported = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let out = invoke(&mut s.graph, call);
        s.imported = imported(&s.graph);
        out
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.imported.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
