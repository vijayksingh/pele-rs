//! Mutable Radar public object/array contract, pinned Pele 1.6.0 at 458322a5.
// Oracle entry node_modules/pele/dist/pele.js SHA256
// d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d.
// src/diagrams/radar/model.ts SHA256
// d691a6309529e19b2a0db93a565aa2ae6c51c0abf3bd77141cb57d959795f6cd.
use pele::compat::{
    diagrams::radar::Radar,
    public::{Atom, JsString, ModelGraph, Node, Property, PublicEngine},
    text::MetricsResources,
    *,
};
use serde_json::Value as Json;
use sha2::{Digest, Sha256};
const SOURCE:&str="radar-beta\naxis a[\"Alpha\"], b[\"Beta\"], c[\"Gamma\"]\ncurve one[\"One\"] {1,2,3}\ncurve two[\"Two\"] {3,2,1}";
fn reference(value: &Atom) -> u32 {
    if let Atom::Ref(id) = value {
        *id
    } else {
        panic!("object reference")
    }
}
fn root(g: &ModelGraph) -> u32 {
    reference(&g.root)
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    &g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .unwrap()
        .value
}
fn set(g: &mut ModelGraph, id: u32, key: &str, value: Atom) {
    let existing = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key));
    let key_id = if existing.is_none() {
        let id = g.strings.len() as u32;
        g.strings.push(key.into());
        Some(id)
    } else {
        None
    };
    let p = g.nodes[id as usize].properties_mut();
    if let Some(i) = existing {
        p[i].value = value
    } else {
        p.push(Property {
            key: key_id.unwrap(),
            value,
            flags: 7,
        })
    }
}
fn remove(g: &mut ModelGraph, id: u32, key: &str) {
    let keep: Vec<_> = g.nodes[id as usize]
        .properties()
        .iter()
        .filter(|p| !g.strings[p.key as usize].is(key))
        .cloned()
        .collect();
    *g.nodes[id as usize].properties_mut() = keep;
}
fn string(g: &mut ModelGraph, text: &str) -> Atom {
    let id = g.strings.len() as u32;
    g.strings.push(text.into());
    Atom::String(id)
}
fn text(g: &mut ModelGraph, id: u32, key: &str, value: &str) {
    let value = string(g, value);
    set(g, id, key, value)
}
fn array_values(g: &ModelGraph, id: u32) -> Vec<Atom> {
    let Node::Array { length, .. } = g.nodes[id as usize] else {
        panic!("array")
    };
    (0..length)
        .map(|i| field(g, id, &i.to_string()).clone())
        .collect()
}
fn reverse(g: &mut ModelGraph, id: u32) {
    for (i, value) in array_values(g, id).into_iter().rev().enumerate() {
        set(g, id, &i.to_string(), value)
    }
}
fn ids(g: &ModelGraph) -> (u32, u32, u32, u32, u32, u32) {
    let axes = reference(g.field("axes").unwrap().unwrap());
    let curves = reference(g.field("curves").unwrap().unwrap());
    let a = reference(field(g, axes, "0"));
    let c0 = reference(field(g, curves, "0"));
    let c1 = reference(field(g, curves, "1"));
    let options = reference(g.field("options").unwrap().unwrap());
    (axes, curves, a, c0, c1, options)
}
fn keys(g: &ModelGraph, id: u32) -> Vec<String> {
    g.nodes[id as usize]
        .properties()
        .iter()
        .map(|p| g.strings[p.key as usize].well_formed().unwrap())
        .collect()
}
#[test]
fn radar_plain_object_arrays_and_optional_fields_match_pinned_probe() {
    // Pinned JS: m=parse(SOURCE); prototype is Object.prototype, no own functions;
    // Object.keys for root/options/axis/curve equal the arrays asserted below.
    // Every ordinary data property is enumerable, writable and configurable.
    let mut engine = PublicEngine::empty();
    engine.register(Radar);
    let document = engine.parse(SOURCE, &ParseOptions::default()).unwrap();
    let g = document.snapshot().unwrap();
    let (axes, curves, a, c0, c1, options) = ids(&g);
    for object in [root(&g), a, c0, c1, options] {
        assert!(matches!(g.nodes[object as usize], Node::Object { .. }));
    }
    assert!(matches!(
        g.nodes[reference(field(&g, c0, "entries")) as usize],
        Node::Array { length: 3, .. }
    ));
    assert_eq!(
        keys(&g, root(&g)),
        ["type", "title", "accTitle", "accDescr", "axes", "curves", "options"]
    );
    assert_eq!(
        keys(&g, options),
        ["showLegend", "ticks", "max", "min", "graticule"]
    );
    assert_eq!(keys(&g, a), ["name", "label"]);
    assert_eq!(keys(&g, c0), ["name", "label", "entries"]);
    assert!(g
        .nodes
        .iter()
        .flat_map(Node::properties)
        .all(|p| p.flags == 7));
    assert!(matches!(
        g.nodes[axes as usize],
        Node::Array { length: 3, .. }
    ));
    assert!(matches!(
        g.nodes[curves as usize],
        Node::Array { length: 2, .. }
    ));
    assert!(matches!(g.field("title").unwrap(), Some(Atom::Undefined)));
    assert!(matches!(
        g.field("accTitle").unwrap(),
        Some(Atom::Undefined)
    ));
    assert!(matches!(
        g.field("accDescr").unwrap(),
        Some(Atom::Undefined)
    ));
    assert!(matches!(field(&g, options, "max"), Atom::Null));
    assert!(matches!(field(&g,options,"ticks"),Atom::Number(n)if *n==5.));
    assert_ne!(
        reference(field(&g, c0, "entries")),
        reference(field(&g, c1, "entries"))
    );
    assert_ne!(
        reference(field(&g, axes, "0")),
        reference(field(&g, axes, "1"))
    );
}
#[test]
fn radar_mutations_match_exact_pinned_public_svg_probes() {
    // Oracle for each ID: m=parse(SOURCE); perform the operation mirrored in the
    // match arm; r=render(m); capture width/height/SHA256(r.svg). No source reparse.
    let probes = [
        (
            "labels",
            518.,
            419.,
            "04a29f425c7134211ca2842d6219f6c9230c9eefbd75a46447611a0de3d8458c",
        ),
        (
            "ticks40",
            467.,
            387.,
            "40342b347bc0368565c3dfada4b33a55c0d20731fdacc4589f1f8898382fff00",
        ),
        (
            "ticksFraction",
            467.,
            387.,
            "782429bdfdef34bfe8f144a59b4df41a3f2b70a72fd614a034d9298eb3b97f1f",
        ),
        (
            "ticksNaN",
            467.,
            387.,
            "1f25976ef13cb53404222cc9ddb67166e631afc841326efff3cae0eed8ae39f0",
        ),
        (
            "maxUndefined",
            467.,
            387.,
            "19768bf82efc68482611a6a5da358fdfd335c09be68f3373e1408d9a3260fc09",
        ),
        (
            "maxDeleted",
            467.,
            387.,
            "19768bf82efc68482611a6a5da358fdfd335c09be68f3373e1408d9a3260fc09",
        ),
        (
            "maxNaN",
            467.,
            387.,
            "19768bf82efc68482611a6a5da358fdfd335c09be68f3373e1408d9a3260fc09",
        ),
        (
            "maxInf",
            467.,
            387.,
            "19768bf82efc68482611a6a5da358fdfd335c09be68f3373e1408d9a3260fc09",
        ),
        (
            "polygon",
            396.,
            387.,
            "33273ab5441827edfb48c8600d37fe76b9577db9f479155c3f364b60d36fb0e0",
        ),
        (
            "sharedEntries",
            467.,
            387.,
            "9b8a2d3489ad8c31558fea704574dcdf29d5b2c221eccde1a2a94d89e3184a16",
        ),
        (
            "sharedAxes",
            484.,
            387.,
            "5a56645a086281cb137c610edbcd1e801507f6f492010b0d30c9863cf45ff505",
        ),
        (
            "reorder",
            453.,
            387.,
            "b36207caa5e8f098f951a0295b3ad6fe3e3583755bc598dfdd43694fa9edabf5",
        ),
        (
            "incomplete",
            467.,
            387.,
            "981876245666711557306a4d294a860d6bc27677ef0585690d0de97a4ad1e5e3",
        ),
        (
            "nonfinite",
            467.,
            387.,
            "19768bf82efc68482611a6a5da358fdfd335c09be68f3373e1408d9a3260fc09",
        ),
        (
            "negativezero",
            467.,
            387.,
            "55789fd579609299bfcd403395e66c02d7e44b9c35861ac66814177d7e9c48fd",
        ),
        (
            "text",
            467.,
            419.,
            "f5eb1e7a94fd1764c978eb78fcd429f03d6945c3a53f58770b91b63379d0d2ba",
        ),
    ];
    let mut engine = PublicEngine::empty();
    engine.register(Radar);
    for (id, width, height, hash) in probes {
        let mut document = engine.parse(SOURCE, &ParseOptions::default()).unwrap();
        let mut g = document.snapshot().unwrap();
        let r = root(&g);
        let (axes, curves, a, c0, c1, o) = ids(&g);
        let e0 = reference(field(&g, c0, "entries"));
        match id {
            "labels" => {
                text(&mut g, a, "label", "Edited axis");
                text(&mut g, c0, "label", "Edited curve");
                for (k, v) in [
                    ("title", "Edited"),
                    ("accTitle", "Accessible"),
                    ("accDescr", "Details"),
                    ("type", "fake"),
                ] {
                    text(&mut g, r, k, v)
                }
            }
            "ticks40" => set(&mut g, o, "ticks", Atom::Number(40.)),
            "ticksFraction" => set(&mut g, o, "ticks", Atom::Number(2.5)),
            "ticksNaN" => set(&mut g, o, "ticks", Atom::Number(f64::NAN)),
            "maxUndefined" => set(&mut g, o, "max", Atom::Undefined),
            "maxDeleted" => remove(&mut g, o, "max"),
            "maxNaN" => set(&mut g, o, "max", Atom::Number(f64::NAN)),
            "maxInf" => set(&mut g, o, "max", Atom::Number(f64::INFINITY)),
            "polygon" => {
                text(&mut g, o, "graticule", "polygon");
                set(&mut g, o, "min", Atom::Number(-1.));
                set(&mut g, o, "max", Atom::Number(5.));
                set(&mut g, o, "showLegend", Atom::Bool(false))
            }
            "sharedEntries" => {
                set(&mut g, c1, "entries", Atom::Ref(e0));
                set(&mut g, e0, "0", Atom::Number(4.))
            }
            "sharedAxes" => {
                set(&mut g, axes, "1", Atom::Ref(a));
                text(&mut g, a, "label", "Aliased")
            }
            "reorder" => {
                reverse(&mut g, axes);
                reverse(&mut g, curves);
                for c in [c0, c1] {
                    let entries = reference(field(&g, c, "entries"));
                    reverse(&mut g, entries)
                }
            }
            "incomplete" => {
                remove(&mut g, e0, "2");
                if let Node::Array { length, .. } = &mut g.nodes[e0 as usize] {
                    *length = 2
                }
            }
            "nonfinite" => {
                for (i, v) in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY]
                    .into_iter()
                    .enumerate()
                {
                    set(&mut g, e0, &i.to_string(), Atom::Number(v))
                }
            }
            "negativezero" => {
                set(&mut g, e0, "0", Atom::Number(-0.));
                set(&mut g, o, "min", Atom::Number(-0.))
            }
            "text" => {
                text(&mut g, a, "name", "<script>&");
                text(&mut g, a, "label", "中文 🙂 #35;");
                text(&mut g, c0, "name", "\"quoted\"");
                text(&mut g, c0, "label", "A<br>B");
                text(&mut g, r, "title", "A & B");
                text(&mut g, r, "accTitle", "<img src=x>");
                text(&mut g, r, "accDescr", "Line\nTwo")
            }
            _ => unreachable!(),
        }
        document.apply(0, g).unwrap();
        let saved = serde_json::to_value(document.snapshot().unwrap()).unwrap();
        let rendered = document
            .render(&RenderOptions::default(), &mut MetricsResources)
            .unwrap();
        assert_eq!(rendered.diagram_type, DiagramType::Radar.into());
        assert_eq!(
            (rendered.rendered.width, rendered.rendered.height),
            (width, height),
            "{id}"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(rendered.rendered.svg.as_bytes())),
            hash,
            "{id}"
        );
        assert_eq!(
            serde_json::to_value(document.snapshot().unwrap()).unwrap(),
            saved
        );
        assert_eq!(
            document
                .render(&RenderOptions::default(), &mut MetricsResources)
                .unwrap(),
            rendered
        );
    }
}
#[test]
fn radar_maps_aliases_deep_extras_reflow_and_registration_remain_owned() {
    let mut engine = PublicEngine::empty();
    engine.register(Radar);
    let mut document = engine.parse(SOURCE, &ParseOptions::default()).unwrap();
    let mut g = document.snapshot().unwrap();
    let r = root(&g);
    let (_, _, a, c0, _, o) = ids(&g);
    let properties = g.nodes[o as usize].properties().to_vec();
    let axis = string(&mut g, "axis");
    let curve = string(&mut g, "curve");
    g.nodes[o as usize] = Node::Map {
        entries: vec![(axis, Atom::Ref(a)), (curve, Atom::Ref(c0))],
        properties,
    };
    set(&mut g, o, "ticks", Atom::Number(4.));
    // Pinned: Object.assign(new Map([['axis',m.axes[0]]]),m.options); set('curve',m.curves[0]); ticks=4.
    document.apply(0, g).unwrap();
    let rendered = document
        .render(&RenderOptions::default(), &mut MetricsResources)
        .unwrap();
    assert_eq!(
        format!("{:x}", Sha256::digest(rendered.rendered.svg.as_bytes())),
        "68b39accb5f4958bb47c0438f3a76770b91c741a643256f0b11a2f3a67914b15"
    );
    let mut g = document.snapshot().unwrap();
    assert!(matches!(g.nodes[o as usize], Node::Map { .. }));
    let Node::Map { entries, .. } = &g.nodes[o as usize] else {
        panic!()
    };
    assert!(matches!(entries[0].1,Atom::Ref(id)if id==a));
    let raw = g.strings.len() as u32;
    g.strings.push(JsString(vec![0xd800, 0x41, 0xdc00]));
    set(&mut g, r, "applicationUTF16", Atom::String(raw));
    for (k, v) in [
        ("applicationNaN", f64::from_bits(0x7ff8000000000042)),
        ("applicationZero", -0.),
        ("applicationInf", f64::INFINITY),
    ] {
        set(&mut g, r, k, Atom::Number(v))
    }
    let mut previous = Atom::Ref(r);
    let key = g.strings.len() as u32;
    g.strings.push("next".into());
    for _ in 0..5000 {
        let id = g.nodes.len() as u32;
        g.nodes.push(Node::Object {
            properties: vec![Property {
                key,
                value: previous,
                flags: 7,
            }],
        });
        previous = Atom::Ref(id)
    }
    set(&mut g, r, "applicationCycle", previous);
    let value = serde_json::to_vec(&g).unwrap();
    document
        .apply(1, serde_json::from_slice(&value).unwrap())
        .unwrap();
    let kept = document.snapshot().unwrap();
    assert_eq!(kept.strings[raw as usize].0, vec![0xd800, 0x41, 0xdc00]);
    assert!(
        matches!(kept.field("applicationZero").unwrap(),Some(Atom::Number(v))if v.to_bits()==(-0.0f64).to_bits())
    );
    assert!(
        matches!(kept.field("applicationNaN").unwrap(),Some(Atom::Number(v))if v.to_bits()==0x7ff8000000000042)
    );
    let mut narrow = RenderOptions::default();
    narrow.render.max_width = Some(240.);
    let narrow_rendered = document.render(&narrow, &mut MetricsResources).unwrap();
    assert!(narrow_rendered.rendered.width < rendered.rendered.width);
    engine.register(Radar);
    drop(engine);
    assert_eq!(
        document
            .render(&RenderOptions::default(), &mut MetricsResources)
            .unwrap(),
        rendered
    );
}
struct FailingHost;
impl RenderResources for FailingHost {
    fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
        Err(PeleError::new("Radar host stopped", ErrorCode::Semantic))
    }
}
#[test]
fn radar_edits_do_not_reparse_and_errors_preserve_state_for_recovery() {
    let mut engine = PublicEngine::empty();
    engine.register(Radar);
    let mut document = engine.parse(SOURCE, &ParseOptions::default()).unwrap();
    let mut g = document.snapshot().unwrap();
    let (_, _, a, c0, _, o) = ids(&g);
    let r = root(&g);
    text(&mut g, a, "label", "Edited axis");
    text(&mut g, c0, "label", "Edited curve");
    text(&mut g, r, "title", "Edited");
    text(&mut g, r, "accTitle", "Accessible");
    text(&mut g, r, "accDescr", "Details");
    document.apply(0, g).unwrap();
    let expected=Engine::with_builtins().render_source("radar-beta\ntitle Edited\naccTitle: Accessible\naccDescr: Details\naxis a[\"Edited axis\"], b[\"Beta\"], c[\"Gamma\"]\ncurve one[\"Edited curve\"] {1,2,3}\ncurve two[\"Two\"] {3,2,1}",&RenderOptions::default(),&mut MetricsResources).unwrap();
    let mut options = RenderOptions::default();
    options.parse.preparation.limit = 0;
    assert_eq!(
        document.render(&options, &mut MetricsResources).unwrap(),
        expected
    );
    let saved = serde_json::to_value(document.snapshot().unwrap()).unwrap();
    assert_eq!(
        document
            .render(&options, &mut FailingHost)
            .unwrap_err()
            .message,
        "Radar host stopped"
    );
    options.render.output_limit = Some(1);
    assert_eq!(
        document
            .render(&options, &mut MetricsResources)
            .unwrap_err()
            .code,
        ErrorCode::Limit
    );
    assert_eq!(
        serde_json::to_value(document.snapshot().unwrap()).unwrap(),
        saved
    );
    options.render.output_limit = None;
    let mut revision = 1;
    for (value, message) in [
        (Atom::Number(f64::INFINITY), "work budget"),
        (Atom::Number(8_000_001.), "work budget"),
    ] {
        let mut g = document.snapshot().unwrap();
        set(&mut g, o, "ticks", value);
        document.apply(revision, g).unwrap();
        revision += 1;
        let error = document
            .render(&options, &mut MetricsResources)
            .unwrap_err();
        assert_eq!(error.code, ErrorCode::Limit);
        assert!(error.message.contains(message));
    }
    let mut g = document.snapshot().unwrap();
    set(&mut g, o, "ticks", Atom::Number(5.));
    let raw = g.strings.len() as u32;
    g.strings.push(JsString(vec![0xd800]));
    set(&mut g, a, "label", Atom::String(raw));
    document.apply(revision, g).unwrap();
    revision += 1;
    assert!(document
        .render(&options, &mut MetricsResources)
        .unwrap_err()
        .message
        .contains("lone UTF-16"));
    let mut g = document.snapshot().unwrap();
    text(&mut g, a, "label", "Edited axis");
    document.apply(revision, g).unwrap();
    assert_eq!(
        document.render(&options, &mut MetricsResources).unwrap(),
        expected
    );
}
struct FixtureResources {
    host: Json,
}
impl RenderResources for FixtureResources {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        if self.host["measure"] == "fixed" {
            Ok(r.text.encode_utf16().count() as f64
                * r.size
                * if r.style & 1 != 0 { 0.71 } else { 0.6 })
        } else {
            MetricsResources.text_width(r)
        }
    }
    fn icon(&mut self, name: &str) -> Result<Option<TrustedSvg>, PeleError> {
        Ok(self.host["icons"][name]
            .as_str()
            .map(|s| TrustedSvg::from_trusted_markup(s.into())))
    }
}
#[test]
fn mutable_radar_unedited_results_keep_the_entire_existing_pinned_render_corpus() {
    let fixture: Json = serde_json::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/pele/radar.json"
        ))
        .unwrap(),
    )
    .unwrap();
    let mut engine = PublicEngine::empty();
    engine.register(Radar);
    for case in fixture["cases"].as_array().unwrap() {
        let options: RenderOptions = serde_json::from_value(case["options"].clone()).unwrap();
        let actual = engine
            .parse(case["source"].as_str().unwrap(), &options.parse)
            .and_then(|mut d| {
                d.render(
                    &options,
                    &mut FixtureResources {
                        host: case["host"].clone(),
                    },
                )
            });
        match actual {
            Ok(result) => assert_eq!(
                serde_json::to_value(result).unwrap(),
                case["expected"]["result"],
                "{}",
                case["id"]
            ),
            Err(error) => assert_eq!(
                serde_json::to_value(error).unwrap(),
                case["expected"]["error"],
                "{}",
                case["id"]
            ),
        }
    }
}
#[test]
fn radar_runtime_shape_errors_preserve_graph_and_recover() {
    let mut engine = PublicEngine::empty();
    engine.register(Radar);
    let mut document = engine.parse(SOURCE, &ParseOptions::default()).unwrap();
    let mut graph = document.snapshot().unwrap();
    let (axes, curves, _, curve0, _, options) = ids(&graph);
    let missing = field(&graph, axes, "1").clone();
    remove(&mut graph, axes, "1");
    document.apply(0, graph).unwrap();
    let error = document
        .render(&RenderOptions::default(), &mut MetricsResources)
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Semantic);
    assert_eq!(error.name, "TypeError");
    assert_eq!(
        error.message,
        "Cannot read properties of undefined (reading 'x')"
    );
    let mut graph = document.snapshot().unwrap();
    assert!(matches!(
        graph.nodes[axes as usize],
        Node::Array { length: 3, .. }
    ));
    assert!(!graph.nodes[axes as usize]
        .properties()
        .iter()
        .any(|p| graph.strings[p.key as usize].is("1")));
    set(&mut graph, axes, "1", missing);
    document.apply(1, graph).unwrap();
    assert!(document
        .render(&RenderOptions::default(), &mut MetricsResources)
        .is_ok());
    // Pinned empty circle curves throw TypeError at points[0].x and retain edits.
    let mut graph = document.snapshot().unwrap();
    graph.nodes[axes as usize] = Node::Array {
        length: 0,
        properties: vec![],
    };
    graph.nodes[curves as usize] = Node::Array {
        length: 1,
        properties: graph.nodes[curves as usize]
            .properties()
            .iter()
            .filter(|p| graph.strings[p.key as usize].is("0"))
            .cloned()
            .collect(),
    };
    let entries = reference(field(&graph, curve0, "entries"));
    graph.nodes[entries as usize] = Node::Array {
        length: 0,
        properties: vec![],
    };
    document.apply(2, graph).unwrap();
    assert_eq!(
        document
            .render(&RenderOptions::default(), &mut MetricsResources)
            .unwrap_err()
            .message,
        "Cannot read properties of undefined (reading 'x')"
    );
    let mut graph = document.snapshot().unwrap();
    text(&mut graph, options, "graticule", "polygon");
    document.apply(3, graph).unwrap();
    assert!(document
        .render(&RenderOptions::default(), &mut MetricsResources)
        .is_ok());
}
