//! EventModeling's actual plain objects and Arc-backed native render cache.
//! Pele 458322a5 (MIT kepano). No prototype/builder methods are returned.
//! Sparse active arrays, unknown entity kinds, exotic coercion and active lone
//! UTF16 remain explicit gaps. Existing UTF16 hard-wrap output gap is retained.
use super::{
    EmBox, EmLane, EmNote, EmRelation, EmSpecification, EntityKind, EventModeling,
    EventModelingModel, LaneGroup,
};
use crate::compat::{
    public::*, DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use std::{collections::HashMap, sync::Arc};
const fn f(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        undefined,
        optional: false,
        nullable: false,
        flags: 7,
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/eventmodeling@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "EventModeling",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("type", FieldKind::String, false),
                f("title", FieldKind::String, true),
                f("accTitle", FieldKind::String, true),
                f("accDescr", FieldKind::String, true),
                f("boxes", FieldKind::Array, false),
                f("lanes", FieldKind::Array, false),
                f("relations", FieldKind::Array, false),
                f("notes", FieldKind::Array, false),
                f("specifications", FieldKind::Array, false),
                f("entities", FieldKind::Array, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "EventModelingLane",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("index", FieldKind::Number, false),
                f("label", FieldKind::String, false),
                f("group", FieldKind::String, false),
                f("namespace", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "EventModelingBox",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("id", FieldKind::String, false),
                f("kind", FieldKind::String, false),
                f("entity", FieldKind::String, false),
                f("name", FieldKind::String, false),
                f("namespace", FieldKind::String, true),
                f("reset", FieldKind::Bool, false),
                f("lane", FieldKind::Record(RecordId(1)), false),
                f("data", FieldKind::String, true),
                f("dataType", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "EventModelingRelation",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("source", FieldKind::Record(RecordId(2)), false),
                f("target", FieldKind::Record(RecordId(2)), false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "EventModelingNote",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("frame", FieldKind::String, false),
                f("text", FieldKind::String, false),
                f("dataType", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(5),
            name: "EventModelingSpecification",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("frame", FieldKind::String, false),
                f("given", FieldKind::Array, false),
                f("when", FieldKind::Array, false),
                f("then", FieldKind::Array, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[],
    methods: &[],
};
pub struct EventModelingPublicState {
    graph: ModelGraph,
    model: Result<EventModelingModel, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("eventmodeling".into());
    e
}
fn limit(s: &str) -> PeleError {
    let mut e = invalid(s);
    e.code = ErrorCode::Limit;
    e
}
fn reference(a: Option<&Atom>) -> Result<u32, PeleError> {
    match a {
        Some(Atom::Ref(i)) => Ok(*i),
        _ => Err(invalid(
            "EventModeling active record or collection requires an object",
        )),
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn truthy(g: &ModelGraph, a: Option<&Atom>) -> bool {
    match a {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => false,
        Some(Atom::Number(n)) => *n != 0. && !n.is_nan(),
        Some(Atom::String(i)) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
struct Budget {
    values: usize,
    scans: usize,
    text: usize,
}
impl Budget {
    fn values(&mut self, n: usize) -> Result<(), PeleError> {
        self.values = self.values.saturating_add(n);
        if self.values > 1_000_000 {
            Err(limit(
                "EventModeling imported collections exceed native 1000000 work budget",
            ))
        } else {
            Ok(())
        }
    }
    fn scan(&mut self, g: &ModelGraph, id: u32, reads: usize) -> Result<(), PeleError> {
        self.scans = self.scans.saturating_add(
            g.nodes[id as usize]
                .properties()
                .len()
                .saturating_mul(reads),
        );
        if self.scans > 32_000_000 {
            Err(limit(
                "EventModeling property reads exceed native 32000000 work budget",
            ))
        } else {
            Ok(())
        }
    }
    fn units(&mut self, n: usize) -> Result<(), PeleError> {
        self.text = self.text.saturating_add(n);
        if self.text > 16_000_000 {
            Err(limit(
                "EventModeling imported text exceeds native 16000000 code-unit work budget",
            ))
        } else {
            Ok(())
        }
    }
    fn text(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<String, PeleError> {
        let Some(Atom::String(i)) = a else {
            return Err(invalid(
                "EventModeling active text coercion for this runtime kind is not implemented",
            ));
        };
        let s = &g.strings[*i as usize];
        self.units(s.0.len())?;
        s.well_formed().ok_or_else(|| {
            invalid("EventModeling rendering of lone UTF-16 text is not implemented")
        })
    }
    fn metadata(
        &mut self,
        g: &ModelGraph,
        r: u32,
        key: &str,
        accessible: bool,
    ) -> Result<Option<String>, PeleError> {
        let a = field(g, r, key);
        if matches!(a, None | Some(Atom::Undefined)) || (accessible && !truthy(g, a)) {
            Ok(None)
        } else {
            self.text(g, a).map(Some)
        }
    }
    fn array(
        &mut self,
        g: &ModelGraph,
        a: Option<&Atom>,
        sparse: bool,
    ) -> Result<Vec<Atom>, PeleError> {
        let id = reference(a)?;
        self.scan(g, id, 1)?;
        let Node::Array { length, properties } = &g.nodes[id as usize] else {
            return Err(invalid(
                "EventModeling active indexed collections must be Arrays",
            ));
        };
        self.values(*length as usize)?;
        let mut indexed = HashMap::new();
        for p in properties {
            let s = &g.strings[p.key as usize];
            if s.0.is_empty() || s.0.len() > 10 {
                continue;
            }
            if let Some(k) = s.well_formed() {
                if let Ok(i) = k.parse::<u32>() {
                    if i < *length && i.to_string() == k {
                        indexed.insert(i, p.value.clone());
                    }
                }
            }
        }
        if !sparse && indexed.len() != *length as usize {
            return Err(invalid(
                "EventModeling sparse active arrays are not implemented",
            ));
        }
        Ok((0..*length)
            .map(|i| indexed.remove(&i).unwrap_or(Atom::Undefined))
            .collect())
    }
    fn iterable(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<Vec<Atom>, PeleError> {
        let id = reference(a)?;
        if let Node::Set { values, .. } = &g.nodes[id as usize] {
            self.values(values.len())?;
            return Ok(values.clone());
        }
        self.array(g, a, false)
    }
    fn joined(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<Vec<String>, PeleError> {
        let id = reference(a)?;
        if field(g, id, "join").is_some() {
            return Err(invalid(
                "EventModeling replaced Array.join is not implemented",
            ));
        }
        let values = self.array(g, a, true)?;
        let mut rows = Vec::with_capacity(values.len());
        for a in values {
            let text = match &a {
                Atom::Null | Atom::Undefined => String::new(),
                Atom::String(_) => {
                    rows.push(self.text(g, Some(&a))?);
                    continue;
                }
                Atom::Bool(v) => v.to_string(),
                Atom::Number(n) => {
                    if n.is_nan() {
                        "NaN".into()
                    } else if *n == f64::INFINITY {
                        "Infinity".into()
                    } else if *n == f64::NEG_INFINITY {
                        "-Infinity".into()
                    } else if *n == 0. {
                        "0".into()
                    } else {
                        ryu_js::Buffer::new().format(*n).into()
                    }
                }
                _ => {
                    return Err(invalid(
                        "EventModeling joined object coercion is not implemented",
                    ))
                }
            };
            self.units(text.len())?;
            rows.push(text);
        }
        Ok(rows)
    }
}
fn parse_kind(g: &ModelGraph, a: Option<&Atom>) -> Result<EntityKind, PeleError> {
    let Some(Atom::String(id)) = a else {
        return Err(invalid(
            "EventModeling unknown runtime entity kind is not implemented",
        ));
    };
    let s = &g.strings[*id as usize];
    if s.is("ui") {
        Ok(EntityKind::Ui)
    } else if s.is("processor") {
        Ok(EntityKind::Processor)
    } else if s.is("command") {
        Ok(EntityKind::Command)
    } else if s.is("readmodel") {
        Ok(EntityKind::Readmodel)
    } else if s.is("event") {
        Ok(EntityKind::Event)
    } else {
        Err(invalid(
            "EventModeling unknown runtime entity kind is not implemented",
        ))
    }
}
fn imported(g: &ModelGraph) -> Result<EventModelingModel, PeleError> {
    let mut b = Budget {
        values: 0,
        scans: 0,
        text: 0,
    };
    let r = reference(Some(&g.root))?;
    b.scan(g, r, 12)?;
    let title = b.metadata(g, r, "title", false)?;
    let acc_title = b.metadata(g, r, "accTitle", true)?;
    let acc_descr = b.metadata(g, r, "accDescr", true)?;
    let raw_lanes = b.iterable(g, field(g, r, "lanes"))?;
    let mut lane_map = HashMap::new();
    let mut lanes = vec![];
    for a in raw_lanes {
        let id = reference(Some(&a))?;
        if lane_map.contains_key(&id) {
            continue;
        }
        b.scan(g, id, 1)?;
        let label = b.text(g, field(g, id, "label"))?;
        let lane = Arc::new(EmLane {
            index: 0,
            label,
            group: LaneGroup::Trigger,
            namespace: None,
        });
        lane_map.insert(id, lane.clone());
        lanes.push(lane);
    }
    let raw_boxes = b.array(g, field(g, r, "boxes"), false)?;
    let mut box_map = HashMap::new();
    let mut box_units = HashMap::new();
    let mut boxes = vec![];
    for a in raw_boxes {
        let id = reference(Some(&a))?;
        if let Some(c) = box_map.get(&id) {
            b.units(box_units[&id])?;
            boxes.push(Arc::clone(c));
            continue;
        }
        b.scan(g, id, 7)?;
        let start = b.text;
        let cid = b.text(g, field(g, id, "id"))?;
        let name = b.text(g, field(g, id, "name"))?;
        let kind = parse_kind(g, field(g, id, "kind"))?;
        let lane = reference(field(g, id, "lane"))?;
        let lane = lane_map.get(&lane).cloned().ok_or_else(|| {
            invalid("EventModeling box lane must be present in the active lanes collection")
        })?;
        b.units(lane.label.encode_utf16().count())?;
        let data = if truthy(g, field(g, id, "data")) {
            Some(Arc::<str>::from(b.text(g, field(g, id, "data"))?))
        } else {
            None
        };
        let units = b.text - start;
        let c = Arc::new(EmBox {
            id: cid,
            kind,
            entity: String::new(),
            name,
            namespace: None,
            reset: truthy(g, field(g, id, "reset")),
            lane,
            data,
            data_type: None,
        });
        box_units.insert(id, units);
        box_map.insert(id, c.clone());
        boxes.push(c);
    }
    let mut relations = vec![];
    for a in b.iterable(g, field(g, r, "relations"))? {
        let id = reference(Some(&a))?;
        b.scan(g, id, 2)?;
        let (Some(Atom::Ref(s)), Some(Atom::Ref(t))) =
            (field(g, id, "source"), field(g, id, "target"))
        else {
            continue;
        };
        if let (Some(s), Some(t)) = (box_map.get(s), box_map.get(t)) {
            relations.push(EmRelation {
                source: Arc::clone(s),
                target: Arc::clone(t),
            })
        }
    }
    let mut notes = vec![];
    for a in b.iterable(g, field(g, r, "notes"))? {
        let id = reference(Some(&a))?;
        b.scan(g, id, 2)?;
        notes.push(EmNote {
            frame: b.text(g, field(g, id, "frame"))?,
            text: b.text(g, field(g, id, "text"))?,
            data_type: None,
        })
    }
    let mut specifications = vec![];
    for a in b.iterable(g, field(g, r, "specifications"))? {
        let id = reference(Some(&a))?;
        b.scan(g, id, 4)?;
        specifications.push(EmSpecification {
            frame: b.text(g, field(g, id, "frame"))?,
            given: b.joined(g, field(g, id, "given"))?,
            when: b.joined(g, field(g, id, "when"))?,
            then: b.joined(g, field(g, id, "then"))?,
        })
    }
    let work = relations
        .len()
        .saturating_mul(relations.len())
        .saturating_add(
            boxes
                .len()
                .saturating_mul(notes.len().saturating_add(specifications.len())),
        );
    if work > 128_000_000 {
        return Err(limit(
            "EventModeling layout exceeds native 128000000 work budget",
        ));
    }
    Ok(EventModelingModel {
        diagram_type: DiagramType::Eventmodeling,
        title,
        acc_title,
        acc_descr,
        boxes,
        lanes,
        relations,
        notes,
        specifications,
        entities: vec![],
    })
}
fn kind_text(k: EntityKind) -> &'static str {
    match k {
        EntityKind::Ui => "ui",
        EntityKind::Processor => "processor",
        EntityKind::Command => "command",
        EntityKind::Readmodel => "readmodel",
        EntityKind::Event => "event",
    }
}
fn group_text(k: LaneGroup) -> &'static str {
    match k {
        LaneGroup::Trigger => "trigger",
        LaneGroup::State => "state",
        LaneGroup::Stream => "stream",
    }
}
fn array(out: &mut GraphWriter, values: Vec<Atom>) -> u32 {
    let properties = values
        .into_iter()
        .enumerate()
        .map(|(i, a)| out.property(&i.to_string(), a))
        .collect::<Vec<_>>();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn text_array(out: &mut GraphWriter, v: &[String]) -> u32 {
    let values = v
        .iter()
        .map(|s| Atom::String(out.string(s.as_str())))
        .collect();
    array(out, values)
}
fn optional(out: &mut GraphWriter, s: Option<&str>) -> Atom {
    s.map_or(Atom::Undefined, |s| Atom::String(out.string(s)))
}
struct Export {
    out: GraphWriter,
    lanes: HashMap<usize, u32>,
    boxes: HashMap<usize, u32>,
}
impl Export {
    fn lane(&mut self, l: &Arc<EmLane>) -> u32 {
        let key = Arc::as_ptr(l) as usize;
        if let Some(id) = self.lanes.get(&key) {
            return *id;
        }
        let label = self.out.string(l.label.as_str());
        let group = self.out.string(group_text(l.group));
        let namespace = optional(&mut self.out, l.namespace.as_deref());
        let properties = vec![
            self.out.property("index", Atom::Number(l.index as f64)),
            self.out.property("label", Atom::String(label)),
            self.out.property("group", Atom::String(group)),
            self.out.property("namespace", namespace),
        ];
        let id = self.out.node(Node::Object { properties });
        self.lanes.insert(key, id);
        id
    }
    fn em_box(&mut self, c: &Arc<EmBox>) -> u32 {
        let key = Arc::as_ptr(c) as usize;
        if let Some(id) = self.boxes.get(&key) {
            return *id;
        }
        let lane = self.lane(&c.lane);
        let id = self.out.string(c.id.as_str());
        let kind = self.out.string(kind_text(c.kind));
        let entity = self.out.string(c.entity.as_str());
        let name = self.out.string(c.name.as_str());
        let namespace = optional(&mut self.out, c.namespace.as_deref());
        let data = optional(&mut self.out, c.data.as_deref());
        let data_type = optional(&mut self.out, c.data_type.as_deref());
        let properties = vec![
            self.out.property("id", Atom::String(id)),
            self.out.property("kind", Atom::String(kind)),
            self.out.property("entity", Atom::String(entity)),
            self.out.property("name", Atom::String(name)),
            self.out.property("namespace", namespace),
            self.out.property("reset", Atom::Bool(c.reset)),
            self.out.property("lane", Atom::Ref(lane)),
            self.out.property("data", data),
            self.out.property("dataType", data_type),
        ];
        let id = self.out.node(Node::Object { properties });
        self.boxes.insert(key, id);
        id
    }
}
impl PublicDiagramModule for EventModeling {
    type PublicState = EventModelingPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(EventModelingPublicState { graph, model })
    }
    fn create_public(&self, m: &EventModelingModel) -> Result<EventModelingPublicState, PeleError> {
        let mut e = Export {
            out: GraphWriter::default(),
            lanes: HashMap::new(),
            boxes: HashMap::new(),
        };
        let boxes = m.boxes.iter().map(|c| Atom::Ref(e.em_box(c))).collect();
        let boxes = array(&mut e.out, boxes);
        let lanes = m.lanes.iter().map(|c| Atom::Ref(e.lane(c))).collect();
        let lanes = array(&mut e.out, lanes);
        let mut relations = vec![];
        for c in &m.relations {
            let source = e.em_box(&c.source);
            let target = e.em_box(&c.target);
            let properties = vec![
                e.out.property("source", Atom::Ref(source)),
                e.out.property("target", Atom::Ref(target)),
            ];
            relations.push(Atom::Ref(e.out.node(Node::Object { properties })));
        }
        let relations = array(&mut e.out, relations);
        let mut notes = vec![];
        for c in &m.notes {
            let frame = e.out.string(c.frame.as_str());
            let text = e.out.string(c.text.as_str());
            let data_type = optional(&mut e.out, c.data_type.as_deref());
            let properties = vec![
                e.out.property("frame", Atom::String(frame)),
                e.out.property("text", Atom::String(text)),
                e.out.property("dataType", data_type),
            ];
            notes.push(Atom::Ref(e.out.node(Node::Object { properties })));
        }
        let notes = array(&mut e.out, notes);
        let mut specs = vec![];
        for c in &m.specifications {
            let frame = e.out.string(c.frame.as_str());
            let given = text_array(&mut e.out, &c.given);
            let when = text_array(&mut e.out, &c.when);
            let then = text_array(&mut e.out, &c.then);
            let properties = vec![
                e.out.property("frame", Atom::String(frame)),
                e.out.property("given", Atom::Ref(given)),
                e.out.property("when", Atom::Ref(when)),
                e.out.property("then", Atom::Ref(then)),
            ];
            specs.push(Atom::Ref(e.out.node(Node::Object { properties })));
        }
        let specifications = array(&mut e.out, specs);
        let entities = text_array(&mut e.out, &m.entities);
        let kind = e.out.string("eventmodeling");
        let mut properties = vec![e.out.property("type", Atom::String(kind))];
        for (k, v) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let a = optional(&mut e.out, v.as_deref());
            properties.push(e.out.property(k, a));
        }
        for (k, id) in [
            ("boxes", boxes),
            ("lanes", lanes),
            ("relations", relations),
            ("notes", notes),
            ("specifications", specifications),
            ("entities", entities),
        ] {
            properties.push(e.out.property(k, Atom::Ref(id)));
        }
        let root = e.out.node(Node::Object { properties });
        let graph = e.out.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(EventModelingPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &EventModelingPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(
        &self,
        s: &mut EventModelingPublicState,
        g: ModelGraph,
    ) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn render_public(
        &self,
        s: &mut EventModelingPublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, r)
    }
}
