//! Actual pinned class construction through captured native public capability.
//! No parser seeds; default state, aliases, partial helpers and raw rendering.
use pele::compat::{
    diagrams::sankey::{Sankey, SankeyModel},
    public::*,
    *,
};
use serde_json::{json, Value};
use std::collections::HashMap;

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/public-sankey-constructor.json"
    ))
    .unwrap()
}
fn reference(atom: &Atom) -> u32 {
    let Atom::Ref(id) = atom else {
        panic!("expected object")
    };
    *id
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    &g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .unwrap()
        .value
}
fn graph(g: &ModelGraph) -> Value {
    fn atom(
        g: &ModelGraph,
        value: &Atom,
        ids: &mut HashMap<u32, usize>,
        nodes: &mut Vec<Value>,
    ) -> Value {
        match value {
            Atom::Undefined => json!(["undefined"]),
            Atom::Null => json!(["null"]),
            Atom::Bool(v) => json!(["bool", v]),
            Atom::Number(v) => json!(["number", format!("{:016x}", v.to_bits())]),
            Atom::String(id) => json!(["string", g.strings[*id as usize].well_formed().unwrap()]),
            Atom::Ref(id) => {
                if let Some(index) = ids.get(id) {
                    return json!(["ref", index]);
                }
                let index = nodes.len();
                ids.insert(*id, index);
                nodes.push(Value::Null);
                let n = &g.nodes[*id as usize];
                let kind = match n {
                    Node::Array { .. } => "array",
                    Node::Map { .. } => "map",
                    Node::Instance { .. } => "instance",
                    Node::Object { .. } => "object",
                    _ => panic!("fixture kind"),
                };
                let mut output = json!({"kind":kind,"properties":[]});
                if let Node::Map { entries, .. } = n {
                    output["entries"] = entries
                        .iter()
                        .map(|(k, v)| json!([atom(g, k, ids, nodes), atom(g, v, ids, nodes)]))
                        .collect();
                }
                let mut properties: Vec<Value> = n
                    .properties()
                    .iter()
                    .map(|p| {
                        json!([
                            g.strings[p.key as usize].well_formed().unwrap(),
                            atom(g, &p.value, ids, nodes),
                            p.flags
                        ])
                    })
                    .collect();
                if let Node::Array { length, .. } = n {
                    if !n
                        .properties()
                        .iter()
                        .any(|p| g.strings[p.key as usize].is("length"))
                    {
                        properties.push(json!([
                            "length",
                            ["number", format!("{:016x}", (*length as f64).to_bits())],
                            2
                        ]));
                    }
                }
                output["properties"] = properties.into();
                nodes[index] = output;
                json!(["ref", index])
            }
            _ => panic!("fixture atom"),
        }
    }
    let mut ids = HashMap::new();
    let mut nodes = Vec::new();
    let root = atom(g, &g.root, &mut ids, &mut nodes);
    json!({"root":root,"nodes":nodes})
}
#[derive(Default)]
struct Width {
    trace: Vec<Value>,
}
impl RenderResources for Width {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        self.trace
            .push(json!([r.text, format!("{:016x}", r.size.to_bits())]));
        Ok(r.text.encode_utf16().count() as f64 * r.size * 0.6)
    }
}
fn raw(doc: &mut PublicDocument) -> Value {
    let mut host = Width::default();
    let r = doc
        .render_raw_operation(
            doc.revision(),
            &Object::default(),
            &RenderSettings::default(),
            &mut host,
        )
        .unwrap()
        .result
        .unwrap();
    json!({"svg":r.svg,"width":format!("{:016x}",r.width.to_bits()),"height":format!("{:016x}",r.height.to_bits()),"trace":host.trace})
}
fn invoke(doc: &mut PublicDocument, name: &str, args: Vec<Atom>) -> InvocationOutcome {
    let g = doc.snapshot().unwrap();
    let method = doc
        .model_schema()
        .methods
        .iter()
        .find(|m| m.name == name)
        .unwrap()
        .id;
    doc.invoke(
        doc.revision(),
        MethodCall {
            method,
            receiver: g.root,
            arguments: args,
        },
        &mut Width::default(),
    )
    .unwrap()
}
fn label(doc: &mut PublicDocument, name: &str) -> InvocationOutcome {
    let mut g = doc.snapshot().unwrap();
    let id = g.strings.len() as u32;
    g.strings.push(JsString::from(name));
    doc.apply(doc.revision(), g).unwrap();
    invoke(doc, "findOrCreateNode", vec![Atom::String(id)])
}

#[test]
fn captured_constructor_owns_defaults_helpers_partial_errors_and_raw_rendering() {
    let expected = fixture();
    let engine = PublicEngine::with_builtins();
    let factory = engine
        .capture_constructor(&DiagramType::Sankey.into(), PrototypeId(1))
        .unwrap()
        .unwrap();
    assert!(factory.ignores_arguments());
    assert_eq!(factory.prototype(), PrototypeId(1));
    assert_eq!(factory.schema_view().model.id, Sankey.model_schema().id);
    assert!(engine
        .capture_constructor(&DiagramType::Sankey.into(), PrototypeId(u32::MAX))
        .unwrap()
        .is_none());
    drop(engine); // A captured class remains independent of registry lifetime.
    let mut doc = factory.construct().unwrap();
    let untouched = factory.construct().unwrap();
    assert!(doc.is_constructor_created());
    assert_eq!(doc.revision(), 0);
    assert_eq!(
        graph(&doc.snapshot().unwrap()),
        expected["initial"]["graph"]
    );
    assert_eq!(raw(&mut doc), expected["initial"]["render"]);
    let a = label(&mut doc, "A").result.unwrap();
    let b = label(&mut doc, "B").result.unwrap();
    invoke(&mut doc, "addLink", vec![a.clone(), b, Atom::Number(2.0)])
        .result
        .unwrap();
    assert_eq!(
        reference(&label(&mut doc, "A").result.unwrap()),
        reference(&a)
    );
    assert_eq!(
        graph(&doc.snapshot().unwrap()),
        expected["populated"]["graph"]
    );
    assert_eq!(raw(&mut doc), expected["populated"]["render"]);
    assert_eq!(
        graph(&untouched.snapshot().unwrap()),
        expected["initial"]["graph"]
    );
    let mut g = doc.snapshot().unwrap();
    let nodes = reference(field(&g, reference(&g.root), "nodes"));
    let key = g.strings.len() as u32;
    g.strings.push(JsString::from("length"));
    g.nodes[nodes as usize].properties_mut().push(Property {
        key,
        value: Atom::Number(2.0),
        flags: 0,
    });
    doc.apply(doc.revision(), g).unwrap();
    let before = doc.revision();
    let out = label(&mut doc, "D");
    assert_eq!(out.revision, before + 2);
    let PublicError::JavaScript { name, message } = out.result.unwrap_err() else {
        panic!("readonly array must throw JavaScript TypeError")
    };
    assert_eq!(name, JavaScriptErrorKind::TypeError);
    assert_eq!(
        message.well_formed().unwrap(),
        expected["partial"]["failure"]["message"].as_str().unwrap()
    );
    assert_eq!(graph(&out.graph), expected["partial"]["graph"]);
    let mut g = out.graph;
    let root = reference(&g.root);
    let lookup = reference(field(&g, root, "lookup"));
    let Node::Map { entries, .. } = &g.nodes[lookup as usize] else {
        panic!()
    };
    let orphan = entries.last().unwrap().1.clone();
    let zero = g.strings.len() as u32;
    g.strings.extend([
        JsString::from("0"),
        JsString::from("1"),
        JsString::from("2"),
    ]);
    let prior = g.nodes[nodes as usize].properties();
    let values = [prior[0].value.clone(), prior[1].value.clone(), orphan];
    let next = g.nodes.len() as u32;
    g.nodes.push(Node::Array {
        length: 3,
        properties: values
            .into_iter()
            .enumerate()
            .map(|(i, value)| Property {
                key: zero + i as u32,
                value,
                flags: 7,
            })
            .collect(),
    });
    let slot = g.nodes[root as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is("nodes"))
        .unwrap();
    g.nodes[root as usize].properties_mut()[slot].value = Atom::Ref(next);
    doc.apply(doc.revision(), g).unwrap();
    assert_eq!(
        graph(&doc.snapshot().unwrap()),
        expected["repaired"]["graph"]
    );
    assert_eq!(raw(&mut doc), expected["repaired"]["render"]);
    assert!(doc.is_constructor_created());
}

// Same schema, public state and forwarded factory do not certify this provider
// as the builtin constructor. Existing arbitrary-provider hooks remain separate.
struct Delegate;
impl DiagramModule for Delegate {
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
impl PublicDiagramModule for Delegate {
    type PublicState = <Sankey as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Sankey.model_schema()
    }
    fn public_constructor(
        &self,
        p: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        Sankey.public_constructor(p)
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
fn captured_constructor_survives_replacement_without_authorizing_forwarded_provider() {
    let mut engine = PublicEngine::with_builtins();
    let parsed = engine
        .parse("sankey\nA,B,1", &ParseOptions::default())
        .unwrap();
    assert!(!parsed.is_constructor_created());
    let factory = parsed.capture_constructor(PrototypeId(1)).unwrap();
    engine.register(Delegate);
    assert!(engine
        .capture_constructor(&DiagramType::Sankey.into(), PrototypeId(1))
        .unwrap()
        .is_none());
    let delegated = engine
        .parse("sankey\nA,B,1", &ParseOptions::default())
        .unwrap();
    assert!(delegated.capture_constructor(PrototypeId(1)).is_none());
    drop(parsed);
    drop(delegated);
    drop(engine);
    let made = factory.construct().unwrap();
    assert!(made.is_constructor_created());
    assert!(made.capture_constructor(PrototypeId(1)).is_some());
    assert_eq!(
        graph(&made.snapshot().unwrap()),
        fixture()["initial"]["graph"]
    );
}
