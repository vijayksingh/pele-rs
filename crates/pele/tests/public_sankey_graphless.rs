#![cfg(feature = "web-host-sankey")]
//! An external column consumer exercises the hidden integration boundary.
use pele::compat::{
    diagrams::sankey::Sankey, host_sankey::*, public::*, text::MetricsResources, *,
};
use std::cell::Cell;

#[derive(Clone)]
struct Columns {
    root: Atom,
    strings: Vec<JsString>,
    kinds: Vec<AdmissionKind>,
    properties: Vec<Vec<Property>>,
    maps: Vec<Vec<(Atom, Atom)>>,
    sets: Vec<Vec<Atom>>,
}
impl Default for Columns {
    fn default() -> Self {
        Self {
            root: Atom::Undefined,
            strings: vec![],
            kinds: vec![],
            properties: vec![],
            maps: vec![],
            sets: vec![],
        }
    }
}
fn failure() -> PeleError {
    PeleError::new("Invalid column consumer access", ErrorCode::Semantic)
}
impl Columns {
    fn add(&mut self, node: Node) -> u32 {
        let id = self.kinds.len() as u32;
        let (kind, properties, map, set) = match node {
            Node::Object { properties } => (AdmissionKind::Object, properties, vec![], vec![]),
            Node::Instance {
                prototype,
                properties,
            } => (
                AdmissionKind::Instance(prototype),
                properties,
                vec![],
                vec![],
            ),
            Node::Array { length, properties } => {
                (AdmissionKind::Array(length), properties, vec![], vec![])
            }
            Node::Map {
                entries,
                properties,
            } => (
                AdmissionKind::Map(entries.len()),
                properties,
                entries,
                vec![],
            ),
            Node::Set { values, properties } => {
                (AdmissionKind::Set(values.len()), properties, vec![], values)
            }
            Node::Date {
                milliseconds,
                properties,
            } => (
                AdmissionKind::Date(milliseconds.to_bits()),
                properties,
                vec![],
                vec![],
            ),
            Node::Error {
                error_type,
                properties,
            } => (AdmissionKind::Error(error_type), properties, vec![], vec![]),
        };
        self.kinds.push(kind);
        self.properties.push(properties);
        self.maps.push(map);
        self.sets.push(set);
        id
    }
    fn from_graph(graph: ModelGraph) -> Self {
        let mut result = Self {
            root: graph.root,
            strings: graph.strings,
            ..Self::default()
        };
        for node in graph.nodes {
            result.add(node);
        }
        result
    }
    // ModelGraph is only a differential oracle at assertions, never owner state.
    fn graph(&self) -> ModelGraph {
        let nodes = self
            .kinds
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                let properties = self.properties[i].clone();
                match kind {
                    AdmissionKind::Object => Node::Object { properties },
                    AdmissionKind::Instance(prototype) => Node::Instance {
                        prototype: *prototype,
                        properties,
                    },
                    AdmissionKind::Array(length) => Node::Array {
                        length: *length,
                        properties,
                    },
                    AdmissionKind::Map(_) => Node::Map {
                        entries: self.maps[i].clone(),
                        properties,
                    },
                    AdmissionKind::Set(_) => Node::Set {
                        values: self.sets[i].clone(),
                        properties,
                    },
                    AdmissionKind::Date(bits) => Node::Date {
                        milliseconds: f64::from_bits(*bits),
                        properties,
                    },
                    AdmissionKind::Error(error_type) => Node::Error {
                        error_type: *error_type,
                        properties,
                    },
                    _ => panic!("observed encoding marker"),
                }
            })
            .collect();
        ModelGraph {
            version: 1,
            root: self.root.clone(),
            strings: self.strings.clone(),
            nodes,
        }
    }
    fn string_id(&mut self, text: &str) -> u32 {
        if let Some(i) = self.strings.iter().position(|s| s.is(text)) {
            return i as u32;
        }
        let i = self.strings.len() as u32;
        self.strings.push(text.into());
        i
    }
    fn equal(&self, a: &Atom, b: &Atom) -> bool {
        match (a, b) {
            (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
            (Atom::Bool(a), Atom::Bool(b)) => a == b,
            (Atom::Number(a), Atom::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
            (Atom::String(a), Atom::String(b)) => {
                self.strings[*a as usize] == self.strings[*b as usize]
            }
            (Atom::Ref(a), Atom::Ref(b))
            | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
            | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
            _ => false,
        }
    }
    fn field(&self, id: u32, key: &str) -> Option<Atom> {
        self.properties[id as usize]
            .iter()
            .find(|p| self.strings[p.key as usize].is(key))
            .map(|p| p.value.clone())
    }
    fn root_id(&self) -> u32 {
        if let Atom::Ref(id) = self.root {
            id
        } else {
            panic!("root")
        }
    }
}
impl InitialWriter for Columns {
    type Output = Self;
    fn string(&mut self, text: &str) -> Result<u32, PeleError> {
        Ok(self.string_id(text))
    }
    fn node(&mut self, node: Node) -> Result<u32, PeleError> {
        Ok(self.add(node))
    }
    fn finish(mut self, root: Atom) -> Result<Self, PeleError> {
        self.root = root;
        Ok(self)
    }
}
impl AdmissionRead for Columns {
    fn extent(&mut self) -> Result<Extent, PeleError> {
        Ok(Extent {
            version: 1,
            nodes: self.kinds.len(),
            strings: self.strings.len(),
        })
    }
    fn with_root<R>(
        &mut self,
        visit: impl FnOnce(&Atom) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        visit(&self.root)
    }
    fn visit_string_lengths(
        &mut self,
        mut visit: impl FnMut(usize) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        for s in &self.strings {
            visit(s.0.len())?;
        }
        Ok(())
    }
    fn header(&mut self, id: usize) -> Result<AdmissionHeader, PeleError> {
        Ok(AdmissionHeader {
            properties: self.properties[id].len(),
            kind: self.kinds[id],
        })
    }
    fn with_properties<R>(
        &mut self,
        id: usize,
        start: usize,
        visit: impl FnOnce(&[Property]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let p = &self.properties[id];
        visit(&p[start..p.len().min(start + 3)])
    }
    fn with_map_entries<R>(
        &mut self,
        id: usize,
        start: usize,
        visit: impl FnOnce(&[(Atom, Atom)]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let p = &self.maps[id];
        visit(&p[start..p.len().min(start + 2)])
    }
    fn with_set_values<R>(
        &mut self,
        id: usize,
        start: usize,
        visit: impl FnOnce(&[Atom]) -> Result<R, PeleError>,
    ) -> Result<R, PeleError> {
        let p = &self.sets[id];
        visit(&p[start..p.len().min(start + 3)])
    }
}
impl CapturedRead for Columns {
    type Value = Atom;
    fn root(&self) -> Atom {
        self.root.clone()
    }
    fn header(&mut self, id: u32) -> Result<Option<NodeHeader>, PeleError> {
        Ok(self.kinds.get(id as usize).map(|kind| NodeHeader {
            kind: match kind {
                AdmissionKind::Object => NodeKind::Object,
                AdmissionKind::Instance(_) => NodeKind::Instance,
                AdmissionKind::Array(_) => NodeKind::Array,
                AdmissionKind::Map(_) => NodeKind::Map,
                AdmissionKind::Set(_) => NodeKind::Set,
                AdmissionKind::Date(_) => NodeKind::Date,
                AdmissionKind::Error(_) => NodeKind::Error,
                _ => panic!("marker"),
            },
            property_count: self.properties[id as usize].len() as u32,
            array_length: if let AdmissionKind::Array(n) = kind {
                Some(*n)
            } else {
                None
            },
        }))
    }
    fn own_fields(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [Option<Atom>],
    ) -> Result<(), PeleError> {
        for (name, slot) in names.iter().zip(out) {
            *slot = self.field(id, name);
        }
        Ok(())
    }
    fn property_prefixes(
        &mut self,
        id: u32,
        start: u32,
        out: &mut [PropertyPrefix<Atom>],
    ) -> Result<usize, PeleError> {
        for value in out.iter_mut() {
            *value = PropertyPrefix::default();
        }
        let props = self.properties.get(id as usize).ok_or_else(failure)?;
        let end = props.len().min(start as usize + out.len());
        for (p, slot) in props[start as usize..end].iter().zip(out.iter_mut()) {
            let key = &self.strings[p.key as usize].0;
            slot.key_units = key.len() as u32;
            for (a, b) in slot.prefix.iter_mut().zip(key) {
                *a = *b;
            }
            slot.value = Some(p.value.clone());
        }
        Ok(end - start as usize)
    }
    fn string_length(&mut self, id: u32) -> Result<u32, PeleError> {
        Ok(self.strings.get(id as usize).ok_or_else(failure)?.0.len() as u32)
    }
    fn with_string_chunks(
        &mut self,
        id: u32,
        mut visit: impl FnMut(&[u16]) -> Result<(), PeleError>,
    ) -> Result<(), PeleError> {
        for part in self
            .strings
            .get(id as usize)
            .ok_or_else(failure)?
            .0
            .chunks(STRING_CHUNK)
        {
            visit(part)?;
        }
        Ok(())
    }
}
impl CapturedWrite for Columns {
    fn counts(&mut self) -> Result<GraphCounts, PeleError> {
        Ok(GraphCounts {
            nodes: self.kinds.len(),
            strings: self.strings.len(),
            units: self.strings.iter().map(|s| s.0.len()).sum(),
            work: self.properties.iter().map(Vec::len).sum::<usize>()
                + 2 * self.maps.iter().map(Vec::len).sum::<usize>()
                + self.sets.iter().map(Vec::len).sum::<usize>(),
        })
    }
    fn find_strings(&mut self, names: &[&str], out: &mut [Option<u32>]) -> Result<(), PeleError> {
        for (name, slot) in names.iter().zip(out) {
            *slot = self
                .strings
                .iter()
                .position(|s| s.is(name))
                .map(|i| i as u32);
        }
        Ok(())
    }
    fn intern(&mut self, text: &str) -> Result<u32, PeleError> {
        Ok(self.string_id(text))
    }
    fn append_node(&mut self, node: Node) -> Result<u32, PeleError> {
        Ok(self.add(node))
    }
    fn own_descriptors(
        &mut self,
        id: u32,
        names: &[&str],
        out: &mut [OwnQuery],
    ) -> Result<(), PeleError> {
        for (name, slot) in names.iter().zip(out) {
            *slot = OwnQuery::default();
            for (i, p) in self.properties[id as usize].iter().enumerate() {
                if self.strings[p.key as usize].is(name) {
                    slot.first_slot.get_or_insert(i as u32);
                    slot.any_nonwritable |= p.flags & 2 == 0;
                }
            }
        }
        Ok(())
    }
    fn map_find(&mut self, id: u32, key: &Atom) -> Result<Option<MapMatch>, PeleError> {
        Ok(self.maps[id as usize]
            .iter()
            .enumerate()
            .find(|(_, (k, _))| self.equal(k, key))
            .map(|(i, (_, value))| MapMatch {
                slot: i as u32,
                value: value.clone(),
            }))
    }
    fn replace_map_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        self.maps[id as usize][slot as usize].1 = value;
        Ok(())
    }
    fn append_map_entry(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PeleError> {
        self.maps[id as usize].push((key, value));
        self.kinds[id as usize] = AdmissionKind::Map(self.maps[id as usize].len());
        Ok(())
    }
    fn replace_property_value(&mut self, id: u32, slot: u32, value: Atom) -> Result<(), PeleError> {
        self.properties[id as usize][slot as usize].value = value;
        Ok(())
    }
    fn append_property(&mut self, id: u32, property: Property) -> Result<(), PeleError> {
        self.properties[id as usize].push(property);
        Ok(())
    }
    fn replace_matching_property_values(
        &mut self,
        id: u32,
        key: &str,
        value: Atom,
    ) -> Result<(), PeleError> {
        for p in &mut self.properties[id as usize] {
            if self.strings[p.key as usize].is(key) {
                p.value = value.clone();
            }
        }
        Ok(())
    }
    fn set_array_length(&mut self, id: u32, length: u32) -> Result<(), PeleError> {
        self.kinds[id as usize] = AdmissionKind::Array(length);
        Ok(())
    }
}

fn captured(engine: &PublicEngine, source: &str) -> (Document, Columns) {
    match engine
        .parse_host_sankey_with_limits(
            source,
            &ParseOptions::default(),
            ParseLimits::default(),
            |_| Ok(Columns::default()),
        )
        .unwrap()
    {
        Parsed::Captured { document, storage } => (document, storage),
        _ => panic!("builtin selection"),
    }
}
fn bytes(g: &ModelGraph) -> Vec<u8> {
    encode_binary_exchange(0, g, &[], DEFAULT_BINARY_BUDGET).unwrap()
}
fn same_state(columns: &Columns, arena: &PublicDocument) {
    assert_eq!(bytes(&columns.graph()), bytes(&arena.snapshot().unwrap()));
}
fn call(doc: &Document, root: Atom, name: &str, args: Vec<Atom>) -> MethodCall {
    MethodCall {
        method: doc
            .schema()
            .model
            .methods
            .iter()
            .find(|m| m.name == name)
            .unwrap()
            .id,
        receiver: root,
        arguments: args,
    }
}

#[test]
fn source_columns_are_ready_for_first_helper_and_render_without_graph_reconstruction() {
    let engine = PublicEngine::with_builtins();
    let source = "sankey\nA,B,5\nA,C,2\nB,C,0";
    let (mut doc, mut columns) = captured(&engine, source);
    let mut arena = engine.parse(source, &ParseOptions::default()).unwrap();
    same_state(&columns, &arena);
    doc.snapshot_admission(&mut columns).unwrap();
    let id = columns.string_id("D");
    let observed = columns.graph();
    arena.apply(0, observed).unwrap();
    doc.apply(0, &mut columns).unwrap();
    let method = call(
        &doc,
        columns.root.clone(),
        "findOrCreateNode",
        vec![Atom::String(id)],
    );
    let a = doc.invoke(1, &mut columns, method.clone()).unwrap();
    let b = arena.invoke(1, method, &mut MetricsResources).unwrap();
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
    same_state(&columns, &arena);
    let a = doc
        .render(
            2,
            &mut columns,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = arena
        .render_operation(2, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(a.revision, b.revision);
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
    same_state(&columns, &arena);
    let a = doc
        .render_raw(
            3,
            &mut columns,
            &Object::default(),
            &RenderSettings::default(),
            &mut MetricsResources,
        )
        .unwrap();
    let b = arena
        .render_raw_operation(
            3,
            &Object::default(),
            &RenderSettings::default(),
            &mut MetricsResources,
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
}

#[test]
fn prepared_raw_partial_errors_and_repair_use_same_eager_projection() {
    let engine = PublicEngine::with_builtins();
    let config = Object::default();
    let source = "sankey\nA,B,5";
    let context = || ParseContext {
        source,
        config: &config,
        title: Some("prepared"),
        line_offset: 0,
        max_edges: 500,
    };
    let (mut doc, mut columns) = match engine
        .parse_prepared_host_sankey_as(
            &DiagramType::Sankey.into(),
            context(),
            EdgeLimit::Undefined,
            |_| Ok(Columns::default()),
        )
        .unwrap()
    {
        Parsed::Captured { document, storage } => (document, storage),
        _ => panic!("prepared"),
    };
    let mut arena = engine
        .parse_prepared_as(&DiagramType::Sankey.into(), context(), EdgeLimit::Undefined)
        .unwrap();
    same_state(&columns, &arena);
    let nodes = if let Atom::Ref(id) = columns.field(columns.root_id(), "nodes").unwrap() {
        id
    } else {
        panic!("nodes")
    };
    let length = if let AdmissionKind::Array(n) = columns.kinds[nodes as usize] {
        n
    } else {
        panic!("array")
    };
    let key = columns.string_id("length");
    columns.properties[nodes as usize].push(Property {
        key,
        flags: 0,
        value: Atom::Number(length as f64),
    });
    let text = columns.string_id("missing");
    arena.apply(0, columns.graph()).unwrap();
    doc.apply(0, &mut columns).unwrap();
    let method = call(
        &doc,
        columns.root.clone(),
        "findOrCreateNode",
        vec![Atom::String(text)],
    );
    let a = doc.invoke(1, &mut columns, method.clone()).unwrap();
    let b = arena.invoke(1, method, &mut MetricsResources).unwrap();
    assert!(a.result.is_err());
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
    same_state(&columns, &arena);
    // Raw adoption imports immediately and can retain a latent invalid root.
    let selection = engine
        .select_host_sankey_entry(&DiagramType::Sankey.into())
        .unwrap()
        .unwrap();
    let mut raw = Columns::from_graph(ModelGraph {
        version: 1,
        root: Atom::Null,
        strings: vec![],
        nodes: vec![],
    });
    let mut raw_doc = selection.bind(Object::default(), &mut raw).unwrap();
    let mut raw_arena = PublicDocument::from_graph(Sankey, raw.graph(), Object::default()).unwrap();
    let a = raw_doc
        .render(
            0,
            &mut raw,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = raw_arena
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert!(a.result.is_err());
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
    let (_, good) = captured(&engine, source);
    raw = good;
    raw_doc.apply(1, &mut raw).unwrap();
    raw_arena.apply(1, raw.graph()).unwrap();
    let a = raw_doc
        .render(
            2,
            &mut raw,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = raw_arena
        .render_operation(2, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert!(a.result.is_ok());
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
}

#[test]
fn canonical_native_frame_count_matches_binary_and_keeps_ingress_order() {
    let engine = PublicEngine::with_builtins();
    let (_, mut columns) = captured(&engine, "sankey\nA,B,5");
    let key = columns.string_id("extra");
    columns.add(Node::Date {
        milliseconds: f64::from_bits(0x7ff8000000000042),
        properties: vec![],
    });
    columns.add(Node::Error {
        error_type: ErrorObjectKind::Type,
        properties: vec![],
    });
    columns.add(Node::Set {
        values: vec![
            Atom::Number(-0.0),
            Atom::Undefined,
            Atom::HostReference(u32::MAX),
        ],
        properties: vec![Property {
            key,
            flags: 7,
            value: Atom::Null,
        }],
    });
    let graph = columns.graph();
    for metadata in [0, 17, 61] {
        let expected =
            encode_binary_exchange(0, &graph, &vec![b'x'; metadata], DEFAULT_BINARY_BUDGET)
                .unwrap()
                .len();
        assert_eq!(
            canonical_frame_bytes(&mut columns, metadata, DEFAULT_BINARY_BUDGET).unwrap(),
            expected
        );
        admit_observed(&mut columns, metadata, DEFAULT_BINARY_BUDGET).unwrap();
        admit_outbound(&mut columns, metadata, DEFAULT_BINARY_BUDGET).unwrap();
    }
    let base = canonical_frame_bytes(&mut columns, 0, DEFAULT_BINARY_BUDGET).unwrap();
    admit_outbound(
        &mut columns,
        DEFAULT_BINARY_BUDGET - base,
        DEFAULT_BINARY_BUDGET,
    )
    .unwrap();
    let err = admit_outbound(
        &mut columns,
        DEFAULT_BINARY_BUDGET - base + 1,
        DEFAULT_BINARY_BUDGET,
    )
    .unwrap_err();
    assert_eq!(
        err,
        PeleError::new(
            "Public binary graph exceeds transport budget",
            ErrorCode::Limit
        )
    );
    columns.root = Atom::Ref(u32::MAX);
    assert_eq!(
        admit_observed(&mut columns, DEFAULT_BINARY_BUDGET, DEFAULT_BINARY_BUDGET).unwrap_err(),
        err
    );
    assert_eq!(
        admit_outbound(&mut columns, DEFAULT_BINARY_BUDGET, DEFAULT_BINARY_BUDGET).unwrap_err(),
        PeleError::new("Invalid public object reference", ErrorCode::Semantic)
    );
}

#[test]
fn parser_failure_precedes_sink_factory_and_other_families_keep_arena() {
    let engine = PublicEngine::with_builtins();
    let calls = Cell::new(0);
    let source = "sankey\nA,B,5\nnot csv";
    let old = engine
        .parse(source, &ParseOptions::default())
        .err()
        .unwrap();
    let new = engine
        .parse_host_sankey_with_limits(
            source,
            &ParseOptions::default(),
            ParseLimits::default(),
            |_| {
                calls.set(calls.get() + 1);
                Ok(Columns::default())
            },
        )
        .err()
        .unwrap();
    assert_eq!(new, old);
    assert_eq!(calls.get(), 0);
    let result = engine
        .parse_host_sankey_with_limits(
            "pie\n\"A\":1",
            &ParseOptions::default(),
            ParseLimits::default(),
            |_| {
                calls.set(calls.get() + 1);
                Ok(Columns::default())
            },
        )
        .unwrap();
    assert!(matches!(result, Parsed::Arena(_)));
    assert_eq!(calls.get(), 0);
}

struct Delegated;
impl DiagramModule for Delegated {
    type Model = pele::compat::diagrams::sankey::SankeyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Sankey.descriptor()
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        let mut m = Sankey.parse(c)?;
        if let Some(node) = m.nodes.first_mut() {
            node.index = 41;
        }
        if let Some(link) = m.links.first_mut() {
            link.source.id.push('!');
        }
        Ok(m)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        s: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render(m, c, s, r)
    }
}
impl PublicDiagramModule for Delegated {
    type PublicState = pele::compat::diagrams::sankey::SankeyPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Sankey.model_schema()
    }
    fn create_public(&self, m: &Self::Model) -> Result<Self::PublicState, PeleError> {
        Sankey.create_public(m)
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Sankey.export_public(s)
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        Sankey.apply_public(s, g)
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render_public(s, c, o, r)
    }
}
#[test]
fn captured_builtin_survives_replacement_and_delegating_models_do_not_opt_in() {
    let mut engine = PublicEngine::with_builtins();
    let id = DiagramType::Sankey.into();
    let selection = engine.select_host_sankey_entry(&id).unwrap().unwrap();
    let (mut retained, mut columns) = captured(&engine, "sankey\nA,B,5");
    let mut legacy = engine
        .parse("sankey\nA,B,5", &ParseOptions::default())
        .unwrap();
    engine.register(Delegated);
    assert!(engine.select_host_sankey_entry(&id).unwrap().is_none());
    let mut adopted = selection.bind(Object::default(), &mut columns).unwrap();
    let a = retained
        .render(
            0,
            &mut columns,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = legacy
        .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    let c = adopted
        .render(
            0,
            &mut columns,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&c.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
    let expected = engine
        .parse("sankey\nA,B,5", &ParseOptions::default())
        .unwrap();
    let actual = engine
        .parse_host_sankey_with_limits(
            "sankey\nA,B,5",
            &ParseOptions::default(),
            ParseLimits::default(),
            |_| -> Result<Columns, PeleError> { panic!("delegated source must not publish host") },
        )
        .unwrap();
    let Parsed::Arena(actual) = actual else {
        panic!("delegation opted in")
    };
    assert_eq!(
        bytes(&actual.snapshot().unwrap()),
        bytes(&expected.snapshot().unwrap())
    );
}

#[test]
fn dispatched_output_errors_and_late_transport_failure_keep_native_revision() {
    let engine = PublicEngine::with_builtins();
    let (mut doc, mut columns) = captured(&engine, "sankey\nA,B,5");
    let mut arena = engine
        .parse("sankey\nA,B,5", &ParseOptions::default())
        .unwrap();
    let mut options = RenderOptions::default();
    options.render.output_limit = Some(1);
    let a = doc
        .render(0, &mut columns, &options, &mut MetricsResources, None, true)
        .unwrap();
    let b = arena
        .render_operation(0, &options, &mut MetricsResources)
        .unwrap();
    assert!(a.result.is_err());
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
    assert_eq!(doc.revision(), 1);
    // A later transport refusal must not revert an already dispatched operation.
    assert!(admit_outbound(&mut columns, DEFAULT_BINARY_BUDGET, DEFAULT_BINARY_BUDGET).is_err());
    assert_eq!(
        doc.render(
            0,
            &mut columns,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true
        )
        .err()
        .unwrap(),
        PeleError::new("Stale public model revision", ErrorCode::Semantic)
    );
    let a = doc
        .render(
            1,
            &mut columns,
            &RenderOptions::default(),
            &mut MetricsResources,
            None,
            true,
        )
        .unwrap();
    let b = arena
        .render_operation(1, &RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert!(a.result.is_ok());
    assert_eq!(
        serde_json::to_value(&a.result).unwrap(),
        serde_json::to_value(&b.result).unwrap()
    );
}

struct MalformedSink(Columns);
impl InitialWriter for MalformedSink {
    type Output = Columns;
    fn string(&mut self, s: &str) -> Result<u32, PeleError> {
        InitialWriter::string(&mut self.0, s)
    }
    fn node(&mut self, n: Node) -> Result<u32, PeleError> {
        InitialWriter::node(&mut self.0, n)
    }
    fn finish(self, root: Atom) -> Result<Columns, PeleError> {
        let mut c = self.0.finish(root)?;
        c.add(Node::Set {
            values: vec![Atom::Ref(u32::MAX)],
            properties: vec![],
        });
        Ok(c)
    }
}
#[test]
fn completed_sink_is_independently_admitted_and_invalid_scalar_markers_count_first() {
    let engine = PublicEngine::with_builtins();
    let error = engine
        .parse_host_sankey_with_limits(
            "sankey\nA,B,5",
            &ParseOptions::default(),
            ParseLimits::default(),
            |_| Ok(MalformedSink(Columns::default())),
        )
        .err()
        .unwrap();
    assert_eq!(
        error,
        PeleError::new("Invalid public object reference", ErrorCode::Semantic)
    );
    for kind in [AdmissionKind::InvalidArray, AdmissionKind::InvalidError] {
        let mut columns = Columns::default();
        columns.add(Node::Object { properties: vec![] });
        columns.kinds[0] = kind;
        let n = canonical_frame_bytes(&mut columns, 0, DEFAULT_BINARY_BUDGET).unwrap();
        assert_eq!(
            n,
            if matches!(kind, AdmissionKind::InvalidArray) {
                42
            } else {
                39
            }
        );
        assert_eq!(
            admit_observed(&mut columns, DEFAULT_BINARY_BUDGET, DEFAULT_BINARY_BUDGET).unwrap_err(),
            PeleError::new(
                "Public binary graph exceeds transport budget",
                ErrorCode::Limit
            )
        );
        assert_eq!(
            admit_observed(&mut columns, 0, DEFAULT_BINARY_BUDGET).unwrap_err(),
            PeleError::new("Invalid public binary graph transport", ErrorCode::Semantic)
        );
    }
}

#[test]
fn frozen_source_corpus_has_identical_complete_columns_or_parse_errors() {
    use std::io::Read;
    let file = std::fs::File::open(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/pele/sankey.json.gz"
    ))
    .unwrap();
    let mut bytes_json = Vec::new();
    flate2::read::GzDecoder::new(file)
        .read_to_end(&mut bytes_json)
        .unwrap();
    let fixture: serde_json::Value = serde_json::from_slice(&bytes_json).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 11_616);
    let mut engine = PublicEngine::empty();
    engine.register(Sankey);
    for case in cases {
        let options: ParseOptions = serde_json::from_value(case["options"].clone()).unwrap();
        let source = case["source"].as_str().unwrap();
        let old = engine.parse(source, &options);
        let new =
            engine.parse_host_sankey_with_limits(source, &options, ParseLimits::default(), |_| {
                Ok(Columns::default())
            });
        match (old, new) {
            (Err(a), Err(b)) => assert_eq!(a, b, "{}", case["id"]),
            (
                Ok(a),
                Ok(Parsed::Captured {
                    document,
                    mut storage,
                }),
            ) => {
                document.snapshot_admission(&mut storage).unwrap();
                assert!(
                    bytes(&a.snapshot().unwrap()) == bytes(&storage.graph()),
                    "{}",
                    case["id"]
                );
            }
            _ => panic!("source outcome differs: {}", case["id"]),
        }
    }
}

#[cfg(feature = "web-host-er")]
mod combined_source_routing {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    type Trace = Rc<RefCell<Vec<&'static str>>>;
    struct Sink {
        columns: Columns,
        trace: Trace,
    }
    impl InitialWriter for Sink {
        type Output = Columns;
        fn string(&mut self, text: &str) -> Result<u32, PeleError> {
            InitialWriter::string(&mut self.columns, text)
        }
        fn node(&mut self, node: Node) -> Result<u32, PeleError> {
            InitialWriter::node(&mut self.columns, node)
        }
        fn finish(self, root: Atom) -> Result<Columns, PeleError> {
            self.trace.borrow_mut().push("finish");
            self.columns.finish(root)
        }
        fn abort(self) -> Result<(), PeleError> {
            self.trace.borrow_mut().push("abort");
            Ok(())
        }
    }
    fn sink(trace: &Trace, family: &'static str) -> Sink {
        trace.borrow_mut().push(family);
        Sink {
            columns: Columns::default(),
            trace: trace.clone(),
        }
    }
    #[test]
    fn combined_source_preserves_er_direct_and_fallback_factories_and_exact_tables() {
        use pele::compat::host_er;
        let engine = PublicEngine::with_builtins();
        for (source, expected_trace) in [
            ("erDiagram\nA ||--o{ B : label", vec!["er", "finish"]),
            (
                "erDiagram\nA\nstyle A fill:red",
                vec!["er", "abort", "er", "finish"],
            ),
        ] {
            let old_trace = Trace::default();
            let new_trace = Trace::default();
            let old = engine
                .parse_host_er_with_limits(
                    source,
                    &ParseOptions::default(),
                    ParseLimits::default(),
                    |_| Ok(sink(&old_trace, "er")),
                )
                .unwrap();
            let new = engine
                .parse_host_storage_with_limits(
                    source,
                    &ParseOptions::default(),
                    ParseLimits::default(),
                    |_| Ok(sink(&new_trace, "er")),
                    |_| -> Result<Sink, PeleError> { panic!("ER must not enter Sankey factory") },
                )
                .unwrap();
            let host_er::Parsed::Host {
                policy: a,
                output: a_graph,
            } = old
            else {
                panic!("old ER route")
            };
            let CombinedParsed::Er {
                policy: b,
                output: b_graph,
            } = new
            else {
                panic!("combined ER route")
            };
            assert_eq!(*old_trace.borrow(), expected_trace);
            assert_eq!(*new_trace.borrow(), expected_trace);
            assert_eq!(a.config(), b.config());
            assert_eq!(bytes(&a_graph.graph()), bytes(&b_graph.graph()));
        }
    }
    #[test]
    fn combined_source_keeps_family_specific_parser_factory_and_limit_error_order() {
        let engine = PublicEngine::with_builtins();
        let trace = Trace::default();
        let sentinel = PeleError::new("factory sentinel", ErrorCode::Semantic);
        // Existing ER creates its first unpublished owner before its grammar;
        // Sankey parses successfully before calling its schema/publication factory.
        for source in ["erDiagram\nA { int }", "sankey\nnot csv", "pie\nnot a row"] {
            let old = if source.starts_with("er") {
                engine
                    .parse_host_er_with_limits(
                        source,
                        &ParseOptions::default(),
                        ParseLimits::default(),
                        |_| -> Result<Sink, PeleError> { Err(sentinel.clone()) },
                    )
                    .err()
                    .unwrap()
            } else {
                engine
                    .parse_host_sankey_with_limits(
                        source,
                        &ParseOptions::default(),
                        ParseLimits::default(),
                        |_| -> Result<Sink, PeleError> { Err(sentinel.clone()) },
                    )
                    .err()
                    .unwrap()
            };
            trace.borrow_mut().clear();
            let new = engine
                .parse_host_storage_with_limits(
                    source,
                    &ParseOptions::default(),
                    ParseLimits::default(),
                    |_| -> Result<Sink, PeleError> {
                        trace.borrow_mut().push("er");
                        Err(sentinel.clone())
                    },
                    |_| -> Result<Sink, PeleError> {
                        trace.borrow_mut().push("sankey");
                        Err(sentinel.clone())
                    },
                )
                .err()
                .unwrap();
            assert_eq!(new, old);
            if source.starts_with("er") {
                assert_eq!(*trace.borrow(), vec!["er"]);
            } else {
                assert!(trace.borrow().is_empty());
            }
        }
        for source in ["erDiagram\nA", "sankey\nA,B,5", "pie\n\"A\":1"] {
            let limits = ParseLimits {
                source: Some(SourceLimit::Null),
                max_edges: None,
            };
            let old = engine
                .parse_with_limits(source, &ParseOptions::default(), limits)
                .err()
                .unwrap();
            let new = engine
                .parse_host_storage_with_limits(
                    source,
                    &ParseOptions::default(),
                    limits,
                    |_| -> Result<Sink, PeleError> {
                        panic!("source admission must precede ER factory")
                    },
                    |_| -> Result<Sink, PeleError> {
                        panic!("source admission must precede Sankey factory")
                    },
                )
                .err()
                .unwrap();
            assert_eq!(new, old);
        }
    }
    #[test]
    fn combined_source_keeps_ready_sankey_and_captured_provider_routing() {
        let mut engine = PublicEngine::with_builtins();
        let source = "---\ntitle: captured title\n---\nsankey\nA,B,5";
        let trace = Trace::default();
        let combined = engine
            .parse_host_storage_with_limits(
                source,
                &ParseOptions::default(),
                ParseLimits::default(),
                |_| -> Result<Sink, PeleError> { panic!("Sankey must not construct ER owner") },
                |_| Ok(sink(&trace, "sankey")),
            )
            .unwrap();
        let CombinedParsed::Sankey {
            mut document,
            mut storage,
        } = combined
        else {
            panic!("Sankey route")
        };
        assert_eq!(*trace.borrow(), vec!["sankey", "finish"]);
        let mut arena = engine.parse(source, &ParseOptions::default()).unwrap();
        same_state(&storage, &arena);
        // Replacement changes subsequent source selection; the already returned
        // document retains its concrete captured builtin and effective config.
        engine.register(Delegated);
        let actual = document
            .render(
                0,
                &mut storage,
                &RenderOptions::default(),
                &mut MetricsResources,
                None,
                true,
            )
            .unwrap();
        let expected = arena
            .render_operation(0, &RenderOptions::default(), &mut MetricsResources)
            .unwrap();
        assert_eq!(
            serde_json::to_value(actual.result).unwrap(),
            serde_json::to_value(expected.result).unwrap()
        );
        for source in ["sankey\nA,B,5", "pie\n\"A\":1"] {
            let actual = engine
                .parse_host_storage_with_limits(
                    source,
                    &ParseOptions::default(),
                    ParseLimits::default(),
                    |_| -> Result<Sink, PeleError> { panic!("unselected ER factory") },
                    |_| -> Result<Sink, PeleError> { panic!("delegated codec must not opt in") },
                )
                .unwrap();
            let CombinedParsed::Arena(actual) = actual else {
                panic!("arena route")
            };
            let expected = engine.parse(source, &ParseOptions::default()).unwrap();
            assert_eq!(
                bytes(&actual.snapshot().unwrap()),
                bytes(&expected.snapshot().unwrap())
            );
        }
    }
}
