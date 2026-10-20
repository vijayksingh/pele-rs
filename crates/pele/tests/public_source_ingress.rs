//! Complete source edits through the public document lifecycle, compared with
//! its existing complete-graph apply path. No private admission/counter access.
use pele::compat::{
    diagrams::sankey::{Sankey, SankeyModel},
    public::*,
    text::MetricsResources,
    *,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
const SOURCE: &str = "sankey\nA,B,5\nA,C,2\nB,C,1";
fn parsed() -> PublicDocument {
    PublicEngine::with_builtins()
        .parse(SOURCE, &Default::default())
        .unwrap()
}
// The public source update carries exactly the complete current replacements.
// Adapt its fixed atom header to the ingress's canonical variable atom header;
// all edited record/string bytes come from the actual native document export.
fn ingress(donor: &PublicDocument, id: u64, revision: u64) -> Vec<u8> {
    let p = donor
        .snapshot_read()
        .unwrap()
        .source_layout_update(donor.revision())
        .unwrap()
        .unwrap();
    assert_eq!(&p[..8], b"PLCSRU01");
    let mut out = b"PLCSRI01".to_vec();
    out.extend(id.to_le_bytes());
    out.extend(revision.to_le_bytes());
    out.extend(&p[24..28]);
    let tag = p[28];
    out.push(tag);
    if tag >= 4 {
        out.extend(&p[29..33]);
    }
    if tag == 4 {
        out.extend(&p[33..37]);
    }
    out.extend(&p[37..]);
    out
}
fn bytes(d: &PublicDocument) -> Vec<u8> {
    d.snapshot_read()
        .unwrap()
        .encode_binary(d.revision(), &[], DEFAULT_BINARY_BUDGET)
        .unwrap()
}
fn field<'a>(g: &'a mut ModelGraph, id: usize, name: &str) -> &'a mut Atom {
    let key = g.strings.iter().position(|s| s.is(name)).unwrap() as u32;
    &mut g.nodes[id]
        .properties_mut()
        .iter_mut()
        .find(|p| p.key == key)
        .unwrap()
        .value
}
fn root(g: &ModelGraph) -> usize {
    let Atom::Ref(id) = g.root else { panic!() };
    id as usize
}
fn render_pair(a: &mut PublicDocument, b: &mut PublicDocument) {
    let x = a.render(&Default::default(), &mut MetricsResources);
    let y = b.render(&Default::default(), &mut MetricsResources);
    assert_eq!(x, y);
    assert_eq!(bytes(a), bytes(b));
}

#[test]
fn source_edits_preserve_complete_history_latent_errors_and_rejected_repair() {
    let mut actual = parsed();
    let mut expected = parsed();
    let identity = actual.source_ingress_identity().unwrap();
    let original = expected.snapshot().unwrap();
    let mut graph = original.clone();
    let key = graph.strings.len() as u32;
    graph.strings.extend([
        JsString::from("extension"),
        JsString(vec![0xd800, 0, 0xdc00]),
    ]);
    let id = graph.nodes.len() as u32;
    graph.nodes.push(Node::Object {
        properties: vec![
            Property {
                key,
                value: Atom::Ref(id),
                flags: 7,
            },
            Property {
                key: key + 1,
                value: Atom::Number(f64::from_bits(0x7ff8000000000042)),
                flags: 1,
            },
        ],
    });
    let r = root(&graph);
    graph.nodes[r].properties_mut().push(Property {
        key,
        value: Atom::Ref(id),
        flags: 7,
    });
    expected.apply(0, graph).unwrap();
    actual
        .apply_source_input(&ingress(&expected, identity, 0))
        .unwrap();
    assert_eq!(bytes(&actual), bytes(&expected));
    render_pair(&mut actual, &mut expected);
    // A malformed reference wins over stale revision, just like binary decode.
    let before = bytes(&actual);
    let mut bad = ingress(&expected, identity, 0);
    assert_eq!(bad[28], 6);
    bad[29..33].copy_from_slice(&u32::MAX.to_le_bytes());
    let mut frame = expected
        .snapshot_read()
        .unwrap()
        .encode_binary(0, &[], DEFAULT_BINARY_BUDGET)
        .unwrap();
    assert_eq!(frame[24], 6);
    frame[25..29].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        actual.apply_source_input(&bad).unwrap_err(),
        decode_binary_exchange(&frame, DEFAULT_BINARY_BUDGET).unwrap_err()
    );
    assert_eq!(bytes(&actual), before);
    // The accepted native state still has extension/history. Complete omission
    // means immutable source again, not a merge with that old accepted overlay.
    let revision = actual.revision();
    expected.apply(revision, original.clone()).unwrap();
    actual
        .apply_source_input(&ingress(&expected, identity, revision))
        .unwrap();
    assert_eq!(bytes(&actual), bytes(&expected));
    // Structurally admitted drawable error is stored eagerly, not rejected by
    // apply and not postponed as construction work until the first render.
    let mut invalid = original.clone();
    *field(&mut invalid, 0, "id") = Atom::Number(7.);
    let revision = actual.revision();
    expected.apply(revision, invalid).unwrap();
    actual
        .apply_source_input(&ingress(&expected, identity, revision))
        .unwrap();
    assert_eq!(actual.revision(), expected.revision());
    render_pair(&mut actual, &mut expected);
    let revision = actual.revision();
    expected.apply(revision, original).unwrap();
    actual
        .apply_source_input(&ingress(&expected, identity, revision))
        .unwrap();
    render_pair(&mut actual, &mut expected);
}

#[test]
fn source_edits_reject_schema_identity_and_preserve_partial_helper_state() {
    let mut actual = parsed();
    let mut expected = parsed();
    let identity = actual.source_ingress_identity().unwrap();
    let original = actual.snapshot().unwrap();
    // Instance schema rejection occurs after canonical structural admission.
    let mut bad = ingress(&expected, identity, 0);
    let wrong = parsed().source_ingress_identity().unwrap();
    bad[8..16].copy_from_slice(&wrong.to_le_bytes());
    assert!(actual.apply_source_input(&bad).is_err());
    assert_eq!(actual.revision(), 0);
    // A valid detached Instance can be published through the old API; changing
    // only its prototype ID creates a structural-valid/schema-invalid edit.
    let mut donor = parsed();
    let mut with_instance = original.clone();
    with_instance.nodes.push(Node::Instance {
        prototype: PrototypeId(1),
        properties: vec![],
    });
    donor.apply(0, with_instance).unwrap();
    let mut wrong_schema = ingress(&donor, identity, 0);
    let n = wrong_schema.len();
    wrong_schema[n - 8..n - 4].copy_from_slice(&999u32.to_le_bytes());
    let mut canonical = donor
        .snapshot_read()
        .unwrap()
        .encode_binary(0, &[], DEFAULT_BINARY_BUDGET)
        .unwrap();
    let n = canonical.len();
    canonical[n - 8..n - 4].copy_from_slice(&999u32.to_le_bytes());
    let invalid = decode_binary_exchange(&canonical, DEFAULT_BINARY_BUDGET)
        .unwrap()
        .graph;
    let expected_error = expected.apply(0, invalid).unwrap_err();
    assert_eq!(
        actual.apply_source_input(&wrong_schema).unwrap_err(),
        expected_error
    );
    assert_eq!(actual.revision(), 0);
    let mut locked = original.clone();
    let r = root(&locked);
    let Atom::Ref(nodes) = field(&mut locked, r, "nodes").clone() else {
        panic!()
    };
    let key = locked.strings.len() as u32;
    locked.strings.push(JsString::from("length"));
    let Node::Array { length, properties } = &mut locked.nodes[nodes as usize] else {
        panic!()
    };
    properties.push(Property {
        key,
        value: Atom::Number(*length as f64),
        flags: 0,
    });
    expected.apply(0, locked).unwrap();
    actual
        .apply_source_input(&ingress(&expected, identity, 0))
        .unwrap();
    let call = MethodCall {
        method: MethodId(1),
        receiver: original.root.clone(),
        arguments: vec![Atom::Number(123.)],
    };
    let a = actual
        .invoke(1, call.clone(), &mut MetricsResources)
        .unwrap();
    let b = expected.invoke(1, call, &mut MetricsResources).unwrap();
    assert!(a.result.is_err());
    assert_eq!(
        serde_json::to_vec(&a.result).unwrap(),
        serde_json::to_vec(&b.result).unwrap()
    );
    assert_eq!(bytes(&actual), bytes(&expected));
    let revision = actual.revision();
    expected.apply(revision, original).unwrap();
    actual
        .apply_source_input(&ingress(&expected, identity, revision))
        .unwrap();
    render_pair(&mut actual, &mut expected);
}

#[test]
fn source_ingress_keeps_physical_budget_before_invalid_header_and_revision() {
    let mut actual = parsed();
    let before = bytes(&actual);
    // One bounded transport buffer, no enormous object graph. This proves the
    // physical 64MiB gate; virtual full-graph limit is a distinct gate.
    let oversized = vec![0xff; DEFAULT_BINARY_BUDGET + 1];
    assert_eq!(
        actual.apply_source_input(&oversized).unwrap_err(),
        decode_binary_exchange(&oversized, DEFAULT_BINARY_BUDGET).unwrap_err()
    );
    assert_eq!(bytes(&actual), before);
    assert_eq!(actual.revision(), 0);
}

struct Forwarded {
    applies: Arc<AtomicUsize>,
}
impl DiagramModule for Forwarded {
    type Model = SankeyModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Sankey.descriptor()
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<SankeyModel, PeleError> {
        Sankey.parse(c)
    }
    fn render(
        &self,
        m: &SankeyModel,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Sankey.render(m, c, o, h)
    }
}
impl PublicDiagramModule for Forwarded {
    type PublicState = <Sankey as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Sankey.model_schema()
    }
    fn try_parse_public(
        &self,
        c: ParseContext<'_>,
        e: Option<EdgeLimit>,
    ) -> Option<Result<Self::PublicState, PeleError>> {
        Sankey.try_parse_public(c, e)
    }
    fn create_public(&self, m: &SankeyModel) -> Result<Self::PublicState, PeleError> {
        Sankey.create_public(m)
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Sankey.export_public(s)
    }
    fn source_ingress<'a>(&self, s: &'a mut Self::PublicState) -> Option<SourceIngress<'a>> {
        let token = Sankey.source_ingress(s);
        assert!(token.is_some());
        token
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        self.applies.fetch_add(1, Ordering::SeqCst);
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
fn forwarded_source_capability_does_not_bypass_captured_provider_apply() {
    let applies = Arc::new(AtomicUsize::new(0));
    let mut engine = PublicEngine::empty();
    engine.register(Forwarded {
        applies: applies.clone(),
    });
    let mut model = engine.parse(SOURCE, &Default::default()).unwrap();
    assert!(model.source_ingress_identity().is_none());
    let graph = model.snapshot().unwrap();
    model.apply(0, graph).unwrap();
    assert_eq!(applies.load(Ordering::SeqCst), 1);
    assert!(model
        .render(&Default::default(), &mut MetricsResources)
        .is_ok());
}

// Build large *transport* inputs directly. These never construct a test-side
// ModelGraph/Vec<Atom> for the rejected collections. The public canonical
// decoder remains the error-order oracle, including aggregate work and tables.
#[test]
fn source_ingress_bounds_expansion_before_bad_root_stale_revision_and_repairs() {
    enum Oversize {
        Set(&'static [u32]),
        Map(u32),
        Nodes,
        Strings,
    }
    for case in [
        Oversize::Set(&[8_000_001]),
        Oversize::Set(&[4_000_001, 4_000_001]),
        Oversize::Map(4_000_001),
        Oversize::Nodes,
        Oversize::Strings,
    ] {
        let mut actual = parsed();
        let expected = parsed();
        let original = expected.snapshot().unwrap();
        let identity = actual.source_ingress_identity().unwrap();
        let before = bytes(&actual);
        let old_strings = original.strings.len() as u32;
        let old_nodes = original.nodes.len() as u32;
        let extra_nodes = match case {
            Oversize::Set(counts) => counts.len() as u32,
            Oversize::Map(_) => 1,
            Oversize::Nodes => 1_000_001 - old_nodes,
            Oversize::Strings => 0,
        };
        let strings = if matches!(case, Oversize::Strings) {
            1_000_001
        } else {
            old_strings
        };
        let nodes = old_nodes + extra_nodes;
        // Known public frame prefix: magic, version, revision, metadata length,
        // then the parsed root Ref. Retain its actual catalog/node body bytes.
        let mut at = 24;
        assert_eq!(before[at], 6);
        at += 5;
        assert_eq!(
            u32::from_le_bytes(before[at..at + 4].try_into().unwrap()),
            old_strings
        );
        at += 4;
        let string_start = at;
        for _ in 0..old_strings {
            let count = u32::from_le_bytes(before[at..at + 4].try_into().unwrap()) as usize;
            at += 4 + count * 2;
        }
        let node_start = at + 4;
        assert_eq!(
            u32::from_le_bytes(before[at..node_start].try_into().unwrap()),
            old_nodes
        );
        let mut canonical = b"PELGBIN1".to_vec();
        canonical.extend(1u32.to_le_bytes());
        canonical.extend(999u64.to_le_bytes());
        canonical.extend(0u32.to_le_bytes());
        canonical.push(6);
        canonical.extend(u32::MAX.to_le_bytes());
        canonical.extend(strings.to_le_bytes());
        let mut sparse = b"PLCSRI01".to_vec();
        sparse.extend(identity.to_le_bytes());
        sparse.extend(999u64.to_le_bytes());
        sparse.extend(1u32.to_le_bytes());
        sparse.push(6);
        sparse.extend(u32::MAX.to_le_bytes());
        sparse.extend(strings.to_le_bytes());
        sparse.extend(nodes.to_le_bytes());
        if matches!(case, Oversize::Strings) {
            sparse.extend(strings.to_le_bytes());
            for id in 0..strings {
                sparse.extend(id.to_le_bytes());
                sparse.extend(0u32.to_le_bytes());
                canonical.extend(0u32.to_le_bytes());
            }
        } else {
            sparse.extend(0u32.to_le_bytes());
            canonical.extend(&before[string_start..at]);
        }
        canonical.extend(nodes.to_le_bytes());
        canonical.extend(&before[node_start..]);
        sparse.extend(extra_nodes.to_le_bytes());
        for i in 0..extra_nodes {
            sparse.extend((old_nodes + i).to_le_bytes());
            let (kind, count, atoms) = match case {
                Oversize::Set(counts) => (4, counts[i as usize], counts[i as usize] as usize),
                Oversize::Map(count) => (3, count, count as usize * 2),
                _ => (0, 0, 0),
            };
            for out in [&mut sparse, &mut canonical] {
                out.push(kind);
                if kind != 0 {
                    out.extend(count.to_le_bytes());
                }
                out.resize(out.len() + atoms, 0); // canonical Undefined atoms
                out.extend(0u32.to_le_bytes()); // no own properties
            }
        }
        assert!(sparse.len() < DEFAULT_BINARY_BUDGET);
        assert!(canonical.len() < DEFAULT_BINARY_BUDGET);
        let old = decode_binary_exchange(&canonical, DEFAULT_BINARY_BUDGET).unwrap_err();
        assert_eq!(old.code, ErrorCode::Limit);
        drop(canonical);
        assert_eq!(actual.apply_source_input(&sparse).unwrap_err(), old);
        assert_eq!(actual.revision(), 0);
        assert_eq!(bytes(&actual), before);
        drop(sparse);
        actual
            .apply_source_input(&ingress(&expected, identity, 0))
            .unwrap();
        assert_eq!(actual.revision(), 1);
        assert!(actual
            .render(&Default::default(), &mut MetricsResources)
            .is_ok());
    }
}
