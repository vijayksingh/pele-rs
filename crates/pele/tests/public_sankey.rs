//! Mutable parsed Sankey feature: real collection/object aliases, retained
//! provenance, typed rendering, edit/error recovery and lossless flat graphs.
//! Array length: Node::Array.length stores value; a nondefault own length
//! Property stores descriptor flags0 (readonly). Default writable flags2 is implicit.
//! Pinned db.ts SHA256 4128d8cb55bf40930a0f1fa31094b171d4d304d4b53cfc3973cd8abe3aa8b91d;
//! render.ts SHA256 1108b7bf8d0b9e61cd2e054cf1089cf527ac2c486652ecdf2c2ea713142d933b.
use pele::compat::{
    diagrams::sankey::{Sankey, SankeyLink, SankeyModel, SankeyNode},
    public::{
        Atom, InvocationOutcome, JavaScriptErrorKind, JsString, MethodCall, ModelGraph,
        ModelSchema, Node, Property, PublicDiagramModule, PublicDocument, PublicEngine,
        PublicError,
    },
    text::MetricsResources,
    DiagramModule, DiagramType, Engine, ErrorCode, ModuleDescriptor, Object, ParseContext,
    PeleError, RenderOptions, RenderResources, RenderSettings, Rendered, TextRequest,
};
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
const SOURCE: &str = "sankey-beta\nA,B,2\nA,C,3";
fn reference(value: &Atom) -> u32 {
    if let Atom::Ref(id) = value {
        *id
    } else {
        panic!("object reference expected")
    }
}
fn string(g: &mut ModelGraph, text: &str) -> Atom {
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(text));
    Atom::String(id)
}
fn property(g: &mut ModelGraph, id: u32, key: &str, value: Atom) {
    let existing = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key));
    if let Some(at) = existing {
        properties_mut(&mut g.nodes[id as usize])[at].value = value;
    } else {
        let key_id = g.strings.len() as u32;
        g.strings.push(JsString::from(key));
        properties_mut(&mut g.nodes[id as usize]).push(Property {
            key: key_id,
            value,
            flags: 7,
        });
    }
}
fn properties_mut(node: &mut Node) -> &mut Vec<Property> {
    match node {
        Node::Object { properties }
        | Node::Instance { properties, .. }
        | Node::Array { properties, .. }
        | Node::Map { properties, .. }
        | Node::Set { properties, .. }
        | Node::Date { properties, .. }
        | Node::Error { properties, .. } => properties,
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    &g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .unwrap()
        .value
}
fn root(g: &ModelGraph) -> u32 {
    reference(&g.root)
}
fn array(g: &ModelGraph, id: u32) -> Vec<Atom> {
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        panic!()
    };
    (0..*length)
        .map(|i| {
            properties
                .iter()
                .find(|p| g.strings[p.key as usize].is(&i.to_string()))
                .unwrap()
                .value
                .clone()
        })
        .collect()
}
fn push_object(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> u32 {
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties: vec![] });
    for (k, v) in fields {
        property(g, id, k, v)
    }
    id
}
fn push_array(g: &mut ModelGraph, values: Vec<Atom>) -> u32 {
    let id = g.nodes.len() as u32;
    let length = values.len() as u32;
    g.nodes.push(Node::Array {
        length,
        properties: vec![],
    });
    for (i, v) in values.into_iter().enumerate() {
        property(g, id, &i.to_string(), v)
    }
    id
}
fn fingerprint(g: &ModelGraph) -> String {
    serde_json::to_string(g).unwrap()
}
fn typed(
    nodes: &[(&str, usize)],
    links: &[(usize, usize, f64)],
    title: Option<&str>,
) -> SankeyModel {
    SankeyModel {
        diagram_type: DiagramType::Sankey,
        nodes: nodes
            .iter()
            .map(|(id, index)| SankeyNode {
                id: (*id).into(),
                index: *index,
            })
            .collect(),
        links: links
            .iter()
            .map(|(source, target, value)| SankeyLink {
                source: SankeyNode {
                    id: "unused endpoint label".into(),
                    index: *source,
                },
                target: SankeyNode {
                    id: "unused endpoint label".into(),
                    index: *target,
                },
                value: *value,
            })
            .collect(),
        title: title.map(str::to_owned),
        acc_title: None,
        acc_descr: None,
    }
}
fn direct(model: &SankeyModel) -> Rendered {
    Sankey
        .render(
            model,
            &Object::default(),
            &RenderSettings::default(),
            &mut MetricsResources,
        )
        .unwrap()
}
struct Counted(Arc<AtomicUsize>);
impl DiagramModule for Counted {
    type Model = SankeyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Sankey.descriptor()
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Sankey.parse(c)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render(m, c, o, h)
    }
}
impl PublicDiagramModule for Counted {
    type PublicState = <Sankey as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Sankey.model_schema()
    }
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        call: MethodCall,
        c: &Object,
        h: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        Sankey.invoke_public(s, call, c, h)
    }
    fn create_public(&self, m: &SankeyModel) -> Result<Self::PublicState, PeleError> {
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
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render_public(s, c, o, h)
    }
}
#[test]
fn mutable_sankey_aliases_order_lookup_and_object_replacement() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut engine = PublicEngine::empty();
    engine.register(Counted(count.clone()));
    let mut doc = engine.parse(SOURCE, &Default::default()).unwrap();
    let mut g = doc.snapshot().unwrap();
    let root = root(&g);
    assert_eq!(
        g.root_properties()
            .unwrap()
            .iter()
            .map(|p| g.strings[p.key as usize].well_formed().unwrap())
            .collect::<Vec<_>>(),
        ["type", "nodes", "links", "title", "accTitle", "accDescr", "lookup"]
    );
    assert!(g.root_properties().unwrap().iter().all(|p| p.flags == 7));
    assert!(matches!(g.field("title").unwrap(), Some(Atom::Undefined)));
    let nodes_id = reference(g.field("nodes").unwrap().unwrap());
    let links_id = reference(g.field("links").unwrap().unwrap());
    let lookup_id = reference(g.field("lookup").unwrap().unwrap());
    let node_values = array(&g, nodes_id);
    let ids: Vec<_> = node_values.iter().map(reference).collect();
    let link_values = array(&g, links_id);
    let l0 = reference(&link_values[0]);
    let l1 = reference(&link_values[1]);
    assert_eq!(reference(field(&g, l0, "source")), ids[0]);
    assert_eq!(reference(field(&g, l1, "source")), ids[0]);
    assert_eq!(reference(field(&g, l0, "target")), ids[1]);
    let Node::Map { entries, .. } = &g.nodes[lookup_id as usize] else {
        panic!()
    };
    assert_eq!(
        entries
            .iter()
            .map(|(_, v)| reference(v))
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        Engine::with_builtins()
            .render_source(SOURCE, &Default::default(), &mut MetricsResources)
            .unwrap()
    );
    let renamed = string(&mut g, "Edited A");
    property(&mut g, ids[0], "id", renamed);
    property(&mut g, l0, "value", Atom::Number(4.));
    property(&mut g, l1, "value", Atom::Number(2.));
    let after = string(&mut g, "After");
    property(&mut g, root, "title", after);
    let fake = string(&mut g, "pie");
    property(&mut g, root, "type", fake);
    // Moving records leaves their index fields and every link/lookup alias intact.
    property(&mut g, nodes_id, "0", node_values[2].clone());
    property(&mut g, nodes_id, "1", node_values[0].clone());
    property(&mut g, nodes_id, "2", node_values[1].clone());
    doc.apply(0, g.clone()).unwrap();
    let result = doc
        .render(&Default::default(), &mut MetricsResources)
        .unwrap();
    let expected = Engine::with_builtins()
        .render_source(
            "---\ntitle: After\n---\nsankey-beta\nC,Edited A,4\nC,B,2",
            &Default::default(),
            &mut MetricsResources,
        )
        .unwrap();
    assert_eq!(result, expected);
    assert_eq!(doc.diagram_type().as_str(), "sankey");
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&g));
    let replacement_label = string(&mut g, "  Replacement  ");
    let replacement = push_object(
        &mut g,
        vec![("id", replacement_label), ("index", Atom::Number(9999.))],
    );
    property(&mut g, nodes_id, "0", Atom::Ref(replacement));
    // A replaced array record does not replace either old endpoint or lookup value.
    assert_eq!(reference(field(&g, l1, "target")), ids[2]);
    let map_id = g.nodes.len() as u32;
    g.nodes.push(Node::Map {
        entries: vec![],
        properties: vec![],
    });
    property(&mut g, root, "lookup", Atom::Ref(map_id));
    let stale_key = string(&mut g, "stale");
    let Node::Map { entries, .. } = &mut g.nodes[map_id as usize] else {
        panic!()
    };
    entries.push((stale_key, Atom::Ref(ids[2])));
    entries.push((Atom::Number(f64::NAN), Atom::Ref(ids[0])));
    doc.apply(1, g.clone()).unwrap();
    let expected = direct(&typed(
        &[("  Replacement  ", 9999), ("Edited A", 0), ("B", 1)],
        &[(0, 1, 4.), (0, 2, 2.)],
        Some("After"),
    ));
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap()
            .rendered,
        expected
    );
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&g));
    let Node::Map { entries, .. } = &mut g.nodes[map_id as usize] else {
        panic!()
    };
    entries.remove(0);
    entries.clear();
    let key_ids: Vec<_> = g
        .strings
        .iter()
        .enumerate()
        .filter(|(_, s)| s.is("lookup"))
        .map(|(i, _)| i as u32)
        .collect();
    properties_mut(&mut g.nodes[root as usize]).retain(|p| !key_ids.contains(&p.key));
    doc.apply(2, g.clone()).unwrap();
    assert!(doc.snapshot().unwrap().field("lookup").unwrap().is_none());
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap()
            .rendered,
        expected
    );
    // Endpoint objects can be detached and need only the field the renderer reads.
    let detached = push_object(&mut g, vec![("index", Atom::Number(2.))]);
    let link = push_object(
        &mut g,
        vec![
            ("source", Atom::Ref(detached)),
            ("target", Atom::Ref(ids[0])),
            ("value", Atom::Number(7.)),
        ],
    );
    let replacement_links = push_array(&mut g, vec![Atom::Ref(link)]);
    property(&mut g, root, "links", Atom::Ref(replacement_links));
    doc.apply(3, g.clone()).unwrap();
    let expected = direct(&typed(
        &[("  Replacement  ", 9999), ("Edited A", 0), ("B", 1)],
        &[(2, 0, 7.)],
        Some("After"),
    ));
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap()
            .rendered,
        expected
    );
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&g));
    assert_eq!(count.load(Ordering::SeqCst), 1);
    engine.register(Sankey);
    drop(engine);
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap()
            .rendered,
        expected
    );
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn mutable_sankey_render_errors_preserve_edits_and_recover_without_parsing() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut engine = PublicEngine::empty();
    engine.register(Counted(count.clone()));
    let mut doc = engine
        .parse("sankey-beta\nA,B,2", &Default::default())
        .unwrap();
    let mut g = doc.snapshot().unwrap();
    let root = root(&g);
    let nodes = reference(g.field("nodes").unwrap().unwrap());
    let link = reference(&array(&g, reference(g.field("links").unwrap().unwrap()))[0]);
    let source = field(&g, link, "source").clone();
    let target = field(&g, link, "target").clone();
    property(&mut g, link, "target", source);
    doc.apply(0, g.clone()).unwrap();
    let error = doc
        .render(&Default::default(), &mut MetricsResources)
        .unwrap_err();
    let expected = Engine::with_builtins()
        .render_source(
            "sankey-beta\nA,A,2",
            &Default::default(),
            &mut MetricsResources,
        )
        .unwrap_err();
    assert_eq!(error, expected);
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&g));
    property(&mut g, link, "target", target.clone());
    property(&mut g, reference(&target), "index", Atom::Number(1e300));
    doc.apply(1, g.clone()).unwrap();
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap_err()
            .code,
        ErrorCode::Semantic
    );
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&g));
    property(&mut g, reference(&target), "index", Atom::Number(1.));
    doc.apply(2, g.clone()).unwrap();
    let expected = doc
        .render(&Default::default(), &mut MetricsResources)
        .unwrap();
    struct Reject;
    impl RenderResources for Reject {
        fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
            Err(PeleError::new(
                "host measurement rejected",
                ErrorCode::Semantic,
            ))
        }
    }
    assert_eq!(
        doc.render(&Default::default(), &mut Reject)
            .unwrap_err()
            .message,
        "host measurement rejected"
    );
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&g));
    let mut options = RenderOptions::default();
    options.render.output_limit = Some(1);
    assert_eq!(
        doc.render(&options, &mut MetricsResources)
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        expected
    );
    let fake = string(&mut g, "wrong kind");
    property(&mut g, root, "nodes", fake);
    doc.apply(3, g.clone()).unwrap();
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap_err()
            .code,
        ErrorCode::Semantic
    );
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&g));
    property(&mut g, root, "nodes", Atom::Ref(nodes));
    doc.apply(4, g).unwrap();
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        expected
    );
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[test]
fn mutable_sankey_retains_unknown_deep_cycles_utf16_and_ieee_fields() {
    let model = typed(&[("A", 0), ("B", 1)], &[(0, 1, 2.)], None);
    let mut doc = PublicDocument::from_module(Sankey, &model, Object::default()).unwrap();
    let mut g = doc.snapshot().unwrap();
    let root = root(&g);
    let mut tail = Atom::Ref(root);
    for _ in 0..6000 {
        tail = Atom::Ref(push_object(&mut g, vec![("next", tail)]));
    }
    property(&mut g, root, "cycle", tail);
    let string_id = g.strings.len() as u32;
    g.strings.push(JsString(vec![0xd800, 0x0041, 0xdc00]));
    property(&mut g, root, "utf16", Atom::String(string_id));
    for (key, value) in [
        ("nan", f64::from_bits(0x7ff8000000001234)),
        ("positiveInfinity", f64::INFINITY),
        ("negativeInfinity", f64::NEG_INFINITY),
        ("negativeZero", -0.),
    ] {
        property(&mut g, root, key, Atom::Number(value));
    }
    let nodes_id = reference(g.field("nodes").unwrap().unwrap());
    let node = reference(&array(&g, nodes_id)[0]);
    property(&mut g, node, "owner", Atom::Ref(root));
    let link = reference(&array(&g, reference(g.field("links").unwrap().unwrap()))[0]);
    property(&mut g, link, "self", Atom::Ref(link));
    let lookup_id = reference(g.field("lookup").unwrap().unwrap());
    property(&mut g, lookup_id, "owner", Atom::Ref(root));
    properties_mut(&mut g.nodes[root as usize]).push(Property {
        key: string_id,
        value: Atom::Ref(lookup_id),
        flags: 5,
    });
    let encoded = fingerprint(&g);
    let decoded: ModelGraph = serde_json::from_str(&encoded).unwrap();
    doc.apply(0, decoded).unwrap();
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), encoded);
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap()
            .rendered,
        direct(&model)
    );
    assert_eq!(fingerprint(&doc.snapshot().unwrap()), encoded);
    for (number, value) in [
        (f64::NAN, 0.),
        (f64::INFINITY, 0.),
        (-5., 0.),
        (-0., 0.),
        (4.5, 4.5),
    ] {
        let mut edit = doc.snapshot().unwrap();
        property(&mut edit, link, "value", Atom::Number(number));
        let revision = doc.revision();
        doc.apply(revision, edit.clone()).unwrap();
        assert_eq!(
            doc.render(&Default::default(), &mut MetricsResources)
                .unwrap()
                .rendered,
            direct(&typed(&[("A", 0), ("B", 1)], &[(0, 1, value)], None))
        );
        assert_eq!(fingerprint(&doc.snapshot().unwrap()), fingerprint(&edit));
    }
}

fn invoke(
    doc: &mut PublicDocument,
    name: &str,
    receiver: Option<Atom>,
    arguments: Vec<Atom>,
) -> InvocationOutcome {
    let id = doc
        .model_schema()
        .methods
        .iter()
        .find(|m| m.name == name)
        .unwrap()
        .id;
    let receiver = receiver.unwrap_or_else(|| doc.snapshot().unwrap().root);
    doc.invoke(
        doc.revision(),
        MethodCall {
            method: id,
            receiver,
            arguments,
        },
        &mut MetricsResources,
    )
    .unwrap()
}
fn found(out: InvocationOutcome) -> (u32, ModelGraph) {
    let id = reference(out.result.as_ref().unwrap());
    (id, out.graph)
}
fn source_result(source: &str) -> pele::compat::RenderResult {
    Engine::with_builtins()
        .render_source(source, &Default::default(), &mut MetricsResources)
        .unwrap()
}
fn same_source(doc: &mut PublicDocument, source: &str) {
    assert_eq!(
        doc.render(&Default::default(), &mut MetricsResources)
            .unwrap(),
        source_result(source)
    )
}
fn apply_graph(doc: &mut PublicDocument, g: ModelGraph) {
    doc.apply(doc.revision(), g).unwrap()
}
fn remove(g: &mut ModelGraph, id: u32, key: &str) {
    let keys = g.strings.clone();
    g.nodes[id as usize]
        .properties_mut()
        .retain(|p| !keys[p.key as usize].is(key))
}
#[test]
fn sankey_native_methods_keep_real_prototype_aliases_live_cache_and_borrowed_receivers() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut engine = PublicEngine::empty();
    engine.register(Counted(count.clone()));
    let mut doc = engine.parse(SOURCE, &Default::default()).unwrap();
    let schema = doc.model_schema();
    assert_eq!(schema.id, "pele/sankey@1");
    assert_eq!(schema.prototypes[0].name, "SankeyDb");
    assert_eq!(
        schema
            .methods
            .iter()
            .map(|m| (m.name, m.arity))
            .collect::<Vec<_>>(),
        [("findOrCreateNode", 1), ("addLink", 3)]
    );
    assert!(schema.prototypes[0].members.iter().all(|m| m.flags == 6));
    let mut g = doc.snapshot().unwrap();
    let root = root(&g);
    assert!(matches!(g.nodes[root as usize], Node::Instance { .. }));
    assert_eq!(g.root_properties().unwrap().len(), 7);
    let nodes = reference(g.field("nodes").unwrap().unwrap());
    let links = reference(g.field("links").unwrap().unwrap());
    let lookup = reference(g.field("lookup").unwrap().unwrap());
    let ids = array(&g, nodes);
    let a = reference(&ids[0]);
    let b = reference(&ids[1]);
    let c = reference(&ids[2]);
    let key_a = field(&g, a, "id").clone();
    let d = string(&mut g, "D");
    apply_graph(&mut doc, g);
    let before = fingerprint(&doc.snapshot().unwrap());
    let (old, g) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        None,
        vec![key_a.clone()],
    ));
    assert_eq!(old, a);
    assert_eq!(before, fingerprint(&g));
    let (d_id, g) = found(invoke(&mut doc, "findOrCreateNode", None, vec![d.clone()]));
    assert_eq!(array(&g, nodes).len(), 4);
    assert_eq!(reference(&array(&g, nodes)[3]), d_id);
    assert!(matches!(field(&g,d_id,"index"),Atom::Number(n)if *n==3.));
    assert_eq!(
        g.nodes[d_id as usize]
            .properties()
            .iter()
            .map(|p| g.strings[p.key as usize].well_formed().unwrap())
            .collect::<Vec<_>>(),
        ["id", "index"]
    );
    assert!(g.nodes[d_id as usize]
        .properties()
        .iter()
        .all(|p| p.flags == 7));
    let Node::Map { entries, .. } = &g.nodes[lookup as usize] else {
        panic!()
    };
    assert_eq!(reference(&entries[3].1), d_id);
    let (out_id, _) = found(invoke(&mut doc, "findOrCreateNode", None, vec![d]));
    assert_eq!(out_id, d_id);
    let out = invoke(
        &mut doc,
        "addLink",
        None,
        vec![Atom::Ref(b), Atom::Ref(d_id), Atom::Number(4.)],
    );
    assert!(matches!(out.result, Ok(Atom::Undefined)));
    let link = reference(&array(&out.graph, links)[2]);
    assert_eq!(reference(field(&out.graph, link, "source")), b);
    assert_eq!(reference(field(&out.graph, link, "target")), d_id);
    assert_eq!(
        out.graph.nodes[link as usize]
            .properties()
            .iter()
            .map(|p| out.graph.strings[p.key as usize].well_formed().unwrap())
            .collect::<Vec<_>>(),
        ["source", "target", "value"]
    );
    same_source(&mut doc, "sankey-beta\nA,B,2\nA,C,3\nB,D,4");
    // Changing a node ID does not rebuild lookup; the old key still returns it.
    let mut g = doc.snapshot().unwrap();
    let renamed = string(&mut g, "Renamed");
    property(&mut g, a, "id", renamed);
    let holder = push_object(
        &mut g,
        vec![
            ("nodes", Atom::Ref(nodes)),
            ("links", Atom::Ref(links)),
            ("lookup", Atom::Ref(lookup)),
        ],
    );
    let e = string(&mut g, "E");
    apply_graph(&mut doc, g);
    let (old, _) = found(invoke(&mut doc, "findOrCreateNode", None, vec![key_a]));
    assert_eq!(old, a);
    let (e_id, g) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        Some(Atom::Ref(holder)),
        vec![e],
    ));
    assert_eq!(reference(&array(&g, nodes)[4]), e_id);
    let out = invoke(
        &mut doc,
        "addLink",
        Some(Atom::Ref(holder)),
        vec![Atom::Ref(c), Atom::Ref(e_id), Atom::Number(5.)],
    );
    assert!(matches!(out.result, Ok(Atom::Undefined)));
    same_source(
        &mut doc,
        "sankey-beta\nRenamed,B,2\nRenamed,C,3\nB,D,4\nC,E,5",
    );
    // Readonly root references do not freeze the mutable Array/Map contents.
    let mut g = doc.snapshot().unwrap();
    for p in g.nodes[root as usize].properties_mut() {
        if ["nodes", "links", "lookup"]
            .iter()
            .any(|k| g.strings[p.key as usize].is(k))
        {
            p.flags = 1
        }
    }
    let f = string(&mut g, "F");
    let fake = string(&mut g, "application");
    property(&mut g, root, "type", fake);
    apply_graph(&mut doc, g);
    let (f_id, _) = found(invoke(&mut doc, "findOrCreateNode", None, vec![f]));
    assert!(matches!(
        invoke(
            &mut doc,
            "addLink",
            None,
            vec![Atom::Ref(e_id), Atom::Ref(f_id), Atom::Number(6.)]
        )
        .result,
        Ok(Atom::Undefined)
    ));
    engine.register(Sankey);
    drop(engine);
    same_source(
        &mut doc,
        "sankey-beta\nRenamed,B,2\nRenamed,C,3\nB,D,4\nC,E,5\nE,F,6",
    );
    assert_eq!(doc.diagram_type().as_str(), "sankey");
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
#[test]
fn sankey_native_partial_failure_keeps_lookup_node_and_never_repairs_arrays_implicitly() {
    let count = Arc::new(AtomicUsize::new(0));
    let mut engine = PublicEngine::empty();
    engine.register(Counted(count.clone()));
    let mut doc = engine.parse(SOURCE, &Default::default()).unwrap();
    let mut g = doc.snapshot().unwrap();
    let root = root(&g);
    let nodes = reference(g.field("nodes").unwrap().unwrap());
    let links = reference(g.field("links").unwrap().unwrap());
    let lookup = reference(g.field("lookup").unwrap().unwrap());
    let ids = array(&g, nodes);
    let a = reference(&ids[0]);
    let key_a = field(&g, a, "id").clone();
    let d = string(&mut g, "D");
    property(&mut g, nodes, "push", Atom::Number(1.));
    apply_graph(&mut doc, g);
    let rev = doc.revision();
    let out = invoke(&mut doc, "findOrCreateNode", None, vec![d.clone()]);
    assert_eq!(out.revision, rev + 1);
    assert!(
        matches!(out.result,Err(PublicError::JavaScript{name:JavaScriptErrorKind::TypeError,message})if message.is("this.nodes.push is not a function"))
    );
    assert_eq!(array(&out.graph, nodes).len(), 3);
    let Node::Map { entries, .. } = &out.graph.nodes[lookup as usize] else {
        panic!()
    };
    let cached = reference(&entries.last().unwrap().1);
    assert!(matches!(field(&out.graph,cached,"index"),Atom::Number(n)if *n==3.));
    assert!(out
        .graph
        .text(field(&out.graph, cached, "id"))
        .unwrap()
        .is("D"));
    assert_eq!(
        fingerprint(&out.graph),
        fingerprint(&doc.snapshot().unwrap())
    );
    same_source(&mut doc, SOURCE);
    let before = fingerprint(&out.graph);
    let (hit, g) = found(invoke(&mut doc, "findOrCreateNode", None, vec![d.clone()]));
    assert_eq!(hit, cached);
    assert_eq!(before, fingerprint(&g));
    // Fixing push does not append the already cached node on the next call.
    let mut g = g;
    remove(&mut g, nodes, "push");
    apply_graph(&mut doc, g);
    let (_, g) = found(invoke(&mut doc, "findOrCreateNode", None, vec![d]));
    assert_eq!(array(&g, nodes).len(), 3);
    // Explicitly repairing the array preserves the cached record identity.
    let mut g = g;
    property(&mut g, nodes, "3", Atom::Ref(cached));
    if let Node::Array { length, .. } = &mut g.nodes[nodes as usize] {
        *length = 4
    }
    apply_graph(&mut doc, g);
    assert!(matches!(
        invoke(
            &mut doc,
            "addLink",
            None,
            vec![Atom::Ref(a), Atom::Ref(cached), Atom::Number(4.)]
        )
        .result,
        Ok(Atom::Undefined)
    ));
    same_source(&mut doc, "sankey-beta\nA,B,2\nA,C,3\nA,D,4");
    // A readonly array length fails after lookup.set, with the exact cached alias.
    let mut g = doc.snapshot().unwrap();
    property(&mut g, nodes, "length", Atom::Number(4.));
    g.nodes[nodes as usize]
        .properties_mut()
        .iter_mut()
        .find(|p| g.strings[p.key as usize].is("length"))
        .unwrap()
        .flags = 0;
    let e = string(&mut g, "E");
    apply_graph(&mut doc, g);
    let out = invoke(&mut doc, "findOrCreateNode", None, vec![e.clone()]);
    assert!(
        matches!(out.result,Err(PublicError::JavaScript{message,..})if message.is("Cannot assign to read only property 'length' of object '[object Array]'"))
    );
    assert_eq!(array(&out.graph, nodes).len(), 4);
    let Node::Map { entries, .. } = &out.graph.nodes[lookup as usize] else {
        panic!()
    };
    let retained = reference(&entries.last().unwrap().1);
    let (hit, _) = found(invoke(&mut doc, "findOrCreateNode", None, vec![e]));
    assert_eq!(hit, retained);
    // Cache hits bypass broken nodes entirely; misses fail before Map mutation.
    let mut g = doc.snapshot().unwrap();
    property(&mut g, root, "nodes", Atom::Null);
    let missing = string(&mut g, "Missing");
    apply_graph(&mut doc, g);
    let (old, _) = found(invoke(&mut doc, "findOrCreateNode", None, vec![key_a]));
    assert_eq!(old, a);
    let before = fingerprint(&doc.snapshot().unwrap());
    let out = invoke(&mut doc, "findOrCreateNode", None, vec![missing]);
    assert_eq!(before, fingerprint(&out.graph));
    assert!(
        matches!(out.result,Err(PublicError::JavaScript{message,..})if message.is("Cannot read properties of null (reading 'length')"))
    );
    // Restore the live data without reparsing; a cyclic link is stored first and
    // rejected only by the real renderer, with its graph retained for recovery.
    let mut g = doc.snapshot().unwrap();
    property(&mut g, root, "nodes", Atom::Ref(nodes));
    remove(&mut g, nodes, "length");
    apply_graph(&mut doc, g);
    let valid = doc.snapshot().unwrap();
    let out = invoke(
        &mut doc,
        "addLink",
        None,
        vec![Atom::Ref(cached), Atom::Ref(a), Atom::Number(1.)],
    );
    assert!(matches!(out.result, Ok(Atom::Undefined)));
    assert_eq!(array(&out.graph, links).len(), 4);
    assert!(doc
        .render(&Default::default(), &mut MetricsResources)
        .is_err());
    assert_eq!(
        fingerprint(&out.graph),
        fingerprint(&doc.snapshot().unwrap())
    );
    apply_graph(&mut doc, valid);
    same_source(&mut doc, "sankey-beta\nA,B,2\nA,C,3\nA,D,4");
    assert_eq!(count.load(Ordering::SeqCst), 1);
}
#[test]
fn sankey_native_lookup_returns_raw_values_same_value_zero_keys_and_utf16_storage() {
    let mut engine = PublicEngine::empty();
    engine.register(Sankey);
    let mut doc = engine.parse(SOURCE, &Default::default()).unwrap();
    let mut g = doc.snapshot().unwrap();
    let root = root(&g);
    let lookup = reference(g.field("lookup").unwrap().unwrap());
    let nodes = reference(g.field("nodes").unwrap().unwrap());
    let key = string(&mut g, "Raw");
    if let Node::Map { entries, .. } = &mut g.nodes[lookup as usize] {
        entries.push((key.clone(), Atom::Null))
    }
    apply_graph(&mut doc, g);
    let before = fingerprint(&doc.snapshot().unwrap());
    let out = invoke(&mut doc, "findOrCreateNode", None, vec![key.clone()]);
    assert!(matches!(out.result, Ok(Atom::Null)));
    assert_eq!(before, fingerprint(&out.graph));
    let mut g = out.graph;
    if let Node::Map { entries, .. } = &mut g.nodes[lookup as usize] {
        entries.last_mut().unwrap().1 = Atom::Bool(false)
    }
    apply_graph(&mut doc, g);
    assert!(matches!(
        invoke(&mut doc, "findOrCreateNode", None, vec![key.clone()]).result,
        Ok(Atom::Bool(false))
    ));
    let mut g = doc.snapshot().unwrap();
    if let Node::Map { entries, .. } = &mut g.nodes[lookup as usize] {
        entries.last_mut().unwrap().1 = Atom::Undefined
    }
    apply_graph(&mut doc, g);
    let (raw, g) = found(invoke(&mut doc, "findOrCreateNode", None, vec![key]));
    assert_eq!(array(&g, nodes).len(), 4);
    let Node::Map { entries, .. } = &g.nodes[lookup as usize] else {
        panic!()
    };
    assert_eq!(entries.len(), 4);
    assert_eq!(reference(&entries[3].1), raw);
    // -0 and +0 select the same Map key, but id keeps the original -0 bits.
    let (zero, g) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        None,
        vec![Atom::Number(-0.)],
    ));
    let Atom::Number(n) = field(&g, zero, "id") else {
        panic!()
    };
    assert_eq!(n.to_bits(), (-0_f64).to_bits());
    let Node::Map { entries, .. } = &g.nodes[lookup as usize] else {
        panic!()
    };
    assert!(matches!(entries.last().unwrap().0,Atom::Number(n)if n.to_bits()==0));
    let (same, _) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        None,
        vec![Atom::Number(0.)],
    ));
    assert_eq!(zero, same);
    let payload = f64::from_bits(0x7ff8000000000042);
    let (nan, g) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        None,
        vec![Atom::Number(payload)],
    ));
    let Atom::Number(n) = field(&g, nan, "id") else {
        panic!()
    };
    assert_eq!(n.to_bits(), payload.to_bits());
    let Node::Map { entries, .. } = &g.nodes[lookup as usize] else {
        panic!()
    };
    assert!(matches!(entries.last().unwrap().0,Atom::Number(n)if n.to_bits()==payload.to_bits()));
    let (same, _) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        None,
        vec![Atom::Number(f64::NAN)],
    ));
    assert_eq!(nan, same);
    let mut g = doc.snapshot().unwrap();
    let arg = g.strings.len() as u32;
    g.strings.push(JsString(vec![32, 0xd800, 10]));
    apply_graph(&mut doc, g);
    let (lone, g) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        None,
        vec![Atom::String(arg)],
    ));
    assert_eq!(g.text(field(&g, lone, "id")).unwrap().0, [32, 0xd800, 10]);
    let (same, _) = found(invoke(
        &mut doc,
        "findOrCreateNode",
        None,
        vec![Atom::String(arg)],
    ));
    assert_eq!(lone, same);
    // A valid string identifier with an isolated surrogate is accepted by the
    // builder unchanged. Its active label rendering limit stays explicit.
    let mut isolated = engine.parse(SOURCE, &Default::default()).unwrap();
    let mut raw = isolated.snapshot().unwrap();
    let id = raw.strings.len() as u32;
    raw.strings.push(JsString(vec![32, 0xd800, 10]));
    apply_graph(&mut isolated, raw);
    let (lone, raw) = found(invoke(
        &mut isolated,
        "findOrCreateNode",
        None,
        vec![Atom::String(id)],
    ));
    assert_eq!(
        raw.text(field(&raw, lone, "id")).unwrap().0,
        [32, 0xd800, 10]
    );
    assert!(isolated
        .render(&Default::default(), &mut MetricsResources)
        .unwrap_err()
        .message
        .contains("lone UTF-16"));
    assert_eq!(
        fingerprint(&raw),
        fingerprint(&isolated.snapshot().unwrap())
    );
    // Replacing collections keeps the ordinary lookup fields and method receiver
    // identity separate; callback overrides are explicit capability errors.
    let mut g = doc.snapshot().unwrap();
    let clean = push_array(&mut g, vec![]);
    property(&mut g, root, "nodes", Atom::Ref(clean));
    let class = string(&mut g, "callback");
    property(&mut g, lookup, "get", Atom::HostReference(7));
    apply_graph(&mut doc, g);
    let out = invoke(&mut doc, "findOrCreateNode", None, vec![class]);
    assert!(
        matches!(out.result,Err(PublicError::Pele(PeleError{message,..}))if message.contains("callback dispatch is unavailable"))
    );
}

fn find_label(doc: &mut PublicDocument, label: &str) -> u32 {
    let mut g = doc.snapshot().unwrap();
    let label = string(&mut g, label);
    apply_graph(doc, g);
    found(invoke(doc, "findOrCreateNode", None, vec![label])).0
}
#[test]
fn sankey_real_methods_and_authored_edits_match_pinned_complete_svg_vectors() {
    // Oracle uses parse(SOURCE), actual database method calls and the indicated
    // direct ordinary edits. Hashes cover every byte of the resulting SVG.
    let vectors = [
        (
            "builder",
            616.,
            416.,
            "40bd1ab650938ca2ccfc47600178a2fb3120994a9bee927c69b99c0213828ee8",
        ),
        (
            "raw-label",
            616.,
            416.,
            "8e7c24b341afe755728544fc0ec0cdf8fe0fc942585af58145de34375fcd6b0d",
        ),
        (
            "deleted-key",
            616.,
            416.,
            "4a725f7fd21fc0209f72ddc0632ddffefe05879feee4747f0f73cd69d8f121fe",
        ),
        (
            "reordered-array",
            616.,
            416.,
            "34b507d951e62c08728180b245957420d95a4ab6aa6aa58f40f21b5efaeba2ef",
        ),
        (
            "replaced-node",
            616.,
            416.,
            "4f31a5a0d4ff773832a477b54b8eee46a87bba27d087277ffc18a66e1b6d5d77",
        ),
        (
            "cache-alias",
            616.,
            416.,
            "745ff65bac1789ab6b328e4dd638e2e55093e6137c107aa866c43a5ca9e1fb9c",
        ),
        (
            "undefined-cache",
            616.,
            416.,
            "eed39185a45451feb72be09530d9305906b99e1e2bd151adf739a3be8885a03a",
        ),
        (
            "isolated",
            616.,
            427.,
            "b61504640af9cd536468efa51d9abf324e6c03951db1bb7497f0cd1da38b4544",
        ),
        (
            "value-negative",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-nan",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-infinity",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-negative-infinity",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-negative-zero",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-false",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-string",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-undefined",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
        (
            "value-null",
            616.,
            416.,
            "7a82d7b1ad65ac6d1ba48d15484c20619502064fc4cafafaedcafe3b37d0566f",
        ),
    ];
    for (name, width, height, expected) in vectors {
        let mut engine = PublicEngine::empty();
        engine.register(Sankey);
        let mut doc = engine.parse(SOURCE, &Default::default()).unwrap();
        let g = doc.snapshot().unwrap();
        let nodes = reference(g.field("nodes").unwrap().unwrap());
        let lookup = reference(g.field("lookup").unwrap().unwrap());
        let ids = array(&g, nodes);
        let mut a = reference(&ids[0]);
        let c = reference(&ids[2]);
        match name {
            "builder" => {
                let d = find_label(&mut doc, "D");
                assert!(matches!(
                    invoke(
                        &mut doc,
                        "addLink",
                        None,
                        vec![ids[1].clone(), Atom::Ref(d), Atom::Number(4.)]
                    )
                    .result,
                    Ok(Atom::Undefined)
                ));
            }
            "raw-label" => {
                let d = find_label(&mut doc, "  D & \"x\"\n<em>raw</em>  ");
                invoke(
                    &mut doc,
                    "addLink",
                    None,
                    vec![Atom::Ref(a), Atom::Ref(d), Atom::Number(4.)],
                );
            }
            "deleted-key" => {
                let mut g = doc.snapshot().unwrap();
                if let Node::Map { entries, .. } = &mut g.nodes[lookup as usize] {
                    entries.remove(0);
                }
                apply_graph(&mut doc, g);
                a = find_label(&mut doc, "A");
                invoke(
                    &mut doc,
                    "addLink",
                    None,
                    vec![Atom::Ref(c), Atom::Ref(a), Atom::Number(1.)],
                );
            }
            "reordered-array" => {
                let mut g = doc.snapshot().unwrap();
                for (i, id) in ids.iter().rev().enumerate() {
                    property(&mut g, nodes, &i.to_string(), id.clone())
                }
                apply_graph(&mut doc, g);
                let d = find_label(&mut doc, "D");
                let a = find_label(&mut doc, "A");
                invoke(
                    &mut doc,
                    "addLink",
                    None,
                    vec![Atom::Ref(a), Atom::Ref(d), Atom::Number(4.)],
                );
            }
            "replaced-node" => {
                let mut g = doc.snapshot().unwrap();
                let name = string(&mut g, "Replacement");
                let replacement =
                    push_object(&mut g, vec![("id", name), ("index", Atom::Number(9999.))]);
                property(&mut g, nodes, "0", Atom::Ref(replacement));
                apply_graph(&mut doc, g);
                let d = find_label(&mut doc, "D");
                let a = find_label(&mut doc, "A");
                invoke(
                    &mut doc,
                    "addLink",
                    None,
                    vec![Atom::Ref(a), Atom::Ref(d), Atom::Number(4.)],
                );
            }
            "cache-alias" | "undefined-cache" => {
                let mut g = doc.snapshot().unwrap();
                if let Node::Map { entries, .. } = &mut g.nodes[lookup as usize] {
                    entries[0].1 = if name == "cache-alias" {
                        Atom::Ref(c)
                    } else {
                        Atom::Undefined
                    }
                }
                apply_graph(&mut doc, g);
                let a = find_label(&mut doc, "A");
                let d = find_label(&mut doc, "D");
                invoke(
                    &mut doc,
                    "addLink",
                    None,
                    vec![Atom::Ref(a), Atom::Ref(d), Atom::Number(4.)],
                );
            }
            "isolated" => {
                find_label(&mut doc, "D");
            }
            _ => {
                let d = find_label(&mut doc, "D");
                let value = match name {
                    "value-negative" => Atom::Number(-3.),
                    "value-nan" => Atom::Number(f64::NAN),
                    "value-infinity" => Atom::Number(f64::INFINITY),
                    "value-negative-infinity" => Atom::Number(f64::NEG_INFINITY),
                    "value-negative-zero" => Atom::Number(-0.),
                    "value-false" => Atom::Bool(false),
                    "value-string" => {
                        let mut g = doc.snapshot().unwrap();
                        let value = string(&mut g, "4");
                        apply_graph(&mut doc, g);
                        value
                    }
                    "value-undefined" => Atom::Undefined,
                    "value-null" => Atom::Null,
                    _ => panic!("case"),
                };
                assert!(matches!(
                    invoke(
                        &mut doc,
                        "addLink",
                        None,
                        vec![Atom::Ref(a), Atom::Ref(d), value]
                    )
                    .result,
                    Ok(Atom::Undefined)
                ));
            }
        }
        let rendered = doc
            .render(&Default::default(), &mut MetricsResources)
            .unwrap();
        assert_eq!(
            (rendered.rendered.width, rendered.rendered.height),
            (width, height),
            "{name}"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(rendered.rendered.svg.as_bytes())),
            expected,
            "{name}"
        );
    }
}

#[test]
fn sankey_projection_keeps_unicode_across_capture_chunks_and_raw_adoption() {
    let engine = PublicEngine::with_builtins();
    for prefix in [0, 1, 1023, 1024, 1025, 2047] {
        let label = format!("{}🦀β", "x".repeat(prefix));
        let mut edited = engine
            .parse("sankey-beta\nA,B,2", &Default::default())
            .unwrap();
        let mut graph = edited.snapshot().unwrap();
        let nodes = reference(graph.field("nodes").unwrap().unwrap());
        let first = reference(&array(&graph, nodes)[0]);
        let text = string(&mut graph, &label);
        property(&mut graph, first, "id", text);
        let mut adopted =
            PublicDocument::from_graph(Sankey, graph.clone(), edited.retained_config().clone())
                .unwrap();
        apply_graph(&mut edited, graph.clone());
        let expected = direct(&typed(&[(&label, 0), ("B", 1)], &[(0, 1, 2.)], None));
        for document in [&mut edited, &mut adopted] {
            assert_eq!(
                document
                    .render(&Default::default(), &mut MetricsResources)
                    .unwrap()
                    .rendered,
                expected,
                "UTF16 prefix {prefix}"
            );
            assert_eq!(
                fingerprint(&document.snapshot().unwrap()),
                fingerprint(&graph)
            );
        }
    }

    for mut units in [
        vec![0xdc00],
        vec![0xd800],
        vec![0xd800, 0x78],
        vec![0xdc00, 0xd800],
    ] {
        let mut document = engine.parse(SOURCE, &Default::default()).unwrap();
        let original = document.snapshot().unwrap();
        let mut graph = original.clone();
        let nodes = reference(graph.field("nodes").unwrap().unwrap());
        let first = reference(&array(&graph, nodes)[0]);
        let mut boundary = vec![0x78; 1023];
        boundary.append(&mut units);
        let id = graph.strings.len() as u32;
        graph.strings.push(JsString(boundary));
        property(&mut graph, first, "id", Atom::String(id));
        apply_graph(&mut document, graph.clone());
        let error = document
            .render(&Default::default(), &mut MetricsResources)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Semantic);
        assert_eq!(
            error.message,
            "Sankey rendering of lone UTF-16 surrogates is not implemented"
        );
        assert_eq!(
            fingerprint(&document.snapshot().unwrap()),
            fingerprint(&graph)
        );
        apply_graph(&mut document, original);
        same_source(&mut document, SOURCE);
    }
}

#[test]
fn sankey_projection_validates_both_arrays_before_labels_and_repairs_latent_errors() {
    let engine = PublicEngine::with_builtins();
    let mut document = engine.parse(SOURCE, &Default::default()).unwrap();
    let original = document.snapshot().unwrap();
    let mut graph = original.clone();
    let root = root(&graph);
    let nodes = reference(graph.field("nodes").unwrap().unwrap());
    let first = reference(&array(&graph, nodes)[0]);
    property(&mut graph, first, "id", Atom::Number(42.));
    property(&mut graph, root, "links", Atom::Undefined);
    apply_graph(&mut document, graph.clone());
    let error = document
        .render(&Default::default(), &mut MetricsResources)
        .unwrap_err();
    assert_eq!(error.message, "Sankey links must be an Array");
    assert_eq!(error.diagram_type.as_deref(), Some("sankey"));

    let sparse = graph.nodes.len() as u32;
    graph.nodes.push(Node::Array {
        length: 1,
        properties: vec![],
    });
    property(&mut graph, root, "links", Atom::Ref(sparse));
    apply_graph(&mut document, graph.clone());
    let error = document
        .render(&Default::default(), &mut MetricsResources)
        .unwrap_err();
    assert_eq!(
        error.message,
        "Sankey links contains a missing array record"
    );

    property(
        &mut graph,
        root,
        "links",
        original.field("links").unwrap().unwrap().clone(),
    );
    apply_graph(&mut document, graph.clone());
    let error = document
        .render(&Default::default(), &mut MetricsResources)
        .unwrap_err();
    assert_eq!(error.message, "Expected a public string");
    assert_eq!(error.diagram_type, None);
    assert_eq!(
        fingerprint(&document.snapshot().unwrap()),
        fingerprint(&graph)
    );
    apply_graph(&mut document, original);
    same_source(&mut document, SOURCE);
}

#[test]
fn sankey_raw_helpers_preserve_first_equivalent_map_slot_and_original_number_bits() {
    let engine = PublicEngine::with_builtins();
    for (existing, argument) in [
        (-0.0, 0.0),
        (
            f64::from_bits(0x7ff8_0000_0000_0042),
            f64::from_bits(0x7ff8_0000_0000_0099),
        ),
    ] {
        let original = engine.parse(SOURCE, &Default::default()).unwrap();
        let mut graph = original.snapshot().unwrap();
        let lookup = reference(graph.field("lookup").unwrap().unwrap());
        let nodes = reference(graph.field("nodes").unwrap().unwrap());
        let second = array(&graph, nodes)[1].clone();
        let first_index_key = graph.strings.iter().position(|s| s.is("index")).unwrap() as u32;
        graph.strings.push(JsString::from("index"));
        let Node::Map { entries, .. } = &mut graph.nodes[lookup as usize] else {
            panic!()
        };
        *entries = vec![
            (Atom::Number(existing), Atom::Undefined),
            (Atom::Number(argument), second.clone()),
        ];
        let mut document =
            PublicDocument::from_graph(Sankey, graph, original.retained_config().clone()).unwrap();
        let (created, graph) = found(invoke(
            &mut document,
            "findOrCreateNode",
            None,
            vec![Atom::Number(argument)],
        ));
        let Node::Map { entries, .. } = &graph.nodes[lookup as usize] else {
            panic!()
        };
        assert_eq!(entries.len(), 2);
        assert!(matches!(entries[0].0, Atom::Number(n) if n.to_bits() == existing.to_bits()));
        assert_eq!(reference(&entries[0].1), created);
        assert!(matches!(entries[1].0, Atom::Number(n) if n.to_bits() == argument.to_bits()));
        assert_eq!(reference(&entries[1].1), reference(&second));
        assert!(
            matches!(field(&graph, created, "id"), Atom::Number(n) if n.to_bits() == argument.to_bits())
        );
        let index_property = graph.nodes[created as usize]
            .properties()
            .iter()
            .find(|p| graph.strings[p.key as usize].is("index"))
            .unwrap();
        assert_eq!(index_property.key, first_index_key);
        assert_eq!(array(&graph, nodes).len(), 4);
        let (cached, again) = found(invoke(
            &mut document,
            "findOrCreateNode",
            None,
            vec![Atom::Number(argument)],
        ));
        assert_eq!(cached, created);
        assert_eq!(fingerprint(&again), fingerprint(&graph));
        assert_eq!(
            document
                .render(&Default::default(), &mut MetricsResources)
                .unwrap_err()
                .message,
            "Expected a public string"
        );
    }
}

#[test]
fn sankey_raw_helpers_keep_duplicate_descriptor_checks_partial_state_and_repair() {
    let engine = PublicEngine::with_builtins();
    for readonly in ["length", "index", "none"] {
        let original = engine.parse(SOURCE, &Default::default()).unwrap();
        let mut graph = original.snapshot().unwrap();
        let nodes = reference(graph.field("nodes").unwrap().unwrap());
        let lookup = reference(graph.field("lookup").unwrap().unwrap());
        let Atom::String(length_key) = string(&mut graph, "length") else {
            panic!()
        };
        let Atom::String(index_key) = string(&mut graph, "3") else {
            panic!()
        };
        let label = string(&mut graph, "D");
        graph.nodes[nodes as usize].properties_mut().extend([
            Property {
                key: length_key,
                value: Atom::Number(3.),
                flags: 2,
            },
            Property {
                key: length_key,
                value: Atom::Number(3.),
                flags: if readonly == "length" { 0 } else { 2 },
            },
            Property {
                key: index_key,
                value: Atom::Null,
                flags: 7,
            },
            Property {
                key: index_key,
                value: Atom::Undefined,
                flags: if readonly == "none" { 7 } else { 0 },
            },
        ]);
        let mut document =
            PublicDocument::from_graph(Sankey, graph, original.retained_config().clone()).unwrap();
        let outcome = invoke(&mut document, "findOrCreateNode", None, vec![label]);
        let mut graph = outcome.graph;
        let created = if readonly == "none" {
            reference(outcome.result.as_ref().unwrap())
        } else {
            let property = if readonly == "length" { "length" } else { "3" };
            assert!(
                matches!(outcome.result, Err(PublicError::JavaScript { name: JavaScriptErrorKind::TypeError, message }) if message.is(&format!("Cannot assign to read only property '{property}' of object '[object Array]'")))
            );
            let Node::Map { entries, .. } = &graph.nodes[lookup as usize] else {
                panic!()
            };
            let detached = reference(&entries.last().unwrap().1);
            assert_eq!(array(&graph, nodes).len(), 3);
            assert!(matches!(field(&graph, detached, "index"), Atom::Number(n) if *n == 3.));
            for property in graph.nodes[nodes as usize].properties_mut() {
                if property.key == length_key {
                    property.flags = 2;
                }
                if property.key == index_key {
                    property.flags = 7;
                }
            }
            let next_label = string(&mut graph, "E");
            apply_graph(&mut document, graph);
            let (id, after) = found(invoke(
                &mut document,
                "findOrCreateNode",
                None,
                vec![next_label],
            ));
            graph = after;
            id
        };
        let Node::Array { length, properties } = &graph.nodes[nodes as usize] else {
            panic!()
        };
        assert_eq!(*length, 4);
        let lengths: Vec<_> = properties.iter().filter(|p| p.key == length_key).collect();
        assert_eq!(lengths.len(), 2);
        assert!(lengths
            .iter()
            .all(|p| p.flags == 2 && matches!(p.value, Atom::Number(n) if n == 4.)));
        let indexes: Vec<_> = properties.iter().filter(|p| p.key == index_key).collect();
        assert_eq!(indexes.len(), 2);
        assert_eq!(reference(&indexes[0].value), created);
        assert!(matches!(indexes[1].value, Atom::Undefined));
        assert_eq!(
            fingerprint(&document.snapshot().unwrap()),
            fingerprint(&graph)
        );
    }
}

#[test]
fn sankey_mutable_repeated_long_labels_reject_native_expansion_and_recover() {
    let mut engine = PublicEngine::empty();
    engine.register(Sankey);
    let mut doc = engine.parse(SOURCE, &Default::default()).unwrap();
    let original = doc.snapshot().unwrap();
    let mut g = original.clone();
    let root = root(&g);
    let nodes = reference(g.field("nodes").unwrap().unwrap());
    let a = reference(&array(&g, nodes)[0]);
    let long = string(&mut g, &"x".repeat(65536));
    property(&mut g, a, "id", long);
    let repeated = push_array(&mut g, vec![Atom::Ref(a); 2049]);
    let empty = push_array(&mut g, vec![]);
    property(&mut g, root, "nodes", Atom::Ref(repeated));
    property(&mut g, root, "links", Atom::Ref(empty));
    let expected = fingerprint(&g);
    apply_graph(&mut doc, g);
    let error = doc
        .render(&Default::default(), &mut MetricsResources)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Limit);
    assert_eq!(
        error.message,
        "Sankey imported text exceeds native allocation budget"
    );
    assert_eq!(expected, fingerprint(&doc.snapshot().unwrap()));
    apply_graph(&mut doc, original);
    same_source(&mut doc, SOURCE);
}
