//! Extension registration preserves validation, replacement and method lifetimes.
use pele::compat::{public::*, *};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
const SCHEMA_A: ModelSchema = ModelSchema {
    id: "consumer/schema@1",
    version: 1,
    root: None,
    records: &[],
    prototypes: &[PrototypeSchema {
        id: PrototypeId(1),
        name: "A",
        parent: PrototypeParent::Object,
        members: &[],
    }],
    methods: &[MethodSchema {
        id: MethodId(1),
        name: "read",
        arity: 0,
    }],
};
const SCHEMA_B: ModelSchema = ModelSchema {
    id: "consumer/schema@1",
    version: 1,
    root: None,
    records: &[],
    prototypes: &[PrototypeSchema {
        id: PrototypeId(2),
        name: "B",
        parent: PrototypeParent::Object,
        members: &[],
    }],
    methods: &[MethodSchema {
        id: MethodId(2),
        name: "read",
        arity: 0,
    }],
};
const INVALID: ModelSchema = ModelSchema { id: "", ..SCHEMA_A };
struct Provider {
    schema: &'static ModelSchema,
    creates: Arc<AtomicUsize>,
}
impl DiagramModule for Provider {
    type Model = ModelGraph;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor {
            id: DiagramId::extension("schema-probe"),
            section: None,
            keep_comments: false,
        }
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<ModelGraph, PeleError> {
        if context.source.contains("reject") {
            return Err(PeleError::new(
                "provider grammar rejected",
                ErrorCode::Syntax,
            ));
        }
        let mut w = GraphWriter::default();
        let p = w.property("read", Atom::BuiltinFunction(self.schema.methods[0].id.0));
        let root = w.node(Node::Instance {
            prototype: self.schema.prototypes[0].id,
            properties: vec![p],
        });
        w.finish(Atom::Ref(root))
    }
    fn render(
        &self,
        _: &ModelGraph,
        _: &Object,
        _: &RenderSettings,
        _: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Ok(Rendered {
            svg: "<svg/>".into(),
            width: 1.,
            height: 1.,
            links: vec![],
        })
    }
}
impl PublicDiagramModule for Provider {
    type PublicState = ModelGraph;
    fn model_schema(&self) -> &'static ModelSchema {
        self.schema
    }
    fn create_public(&self, m: &ModelGraph) -> Result<ModelGraph, PeleError> {
        self.creates.fetch_add(1, Ordering::SeqCst);
        Ok(m.clone())
    }
    fn create_public_from_graph(&self, g: ModelGraph) -> Result<ModelGraph, PeleError> {
        self.creates.fetch_add(1, Ordering::SeqCst);
        Ok(g)
    }
    fn export_public(&self, g: &ModelGraph) -> Result<ModelGraph, PeleError> {
        Ok(g.clone())
    }
    fn borrow_public_graph<'a>(&self, g: &'a ModelGraph) -> Option<&'a ModelGraph> {
        Some(g)
    }
    fn apply_public(&self, state: &mut ModelGraph, g: ModelGraph) -> Result<(), PeleError> {
        *state = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        _: &mut ModelGraph,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        Ok(Atom::Number(call.method.0 as f64))
    }
    fn render_public(
        &self,
        g: &mut ModelGraph,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(g, c, o, h)
    }
}
struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, _: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(1.)
    }
}
fn call(doc: &mut PublicDocument, method: u32) -> Result<InvocationOutcome, PeleError> {
    let receiver = doc.snapshot()?.root;
    doc.invoke(
        doc.revision(),
        MethodCall {
            method: MethodId(method),
            receiver,
            arguments: vec![],
        },
        &mut Host,
    )
}
#[test]
fn replacement_with_identical_schema_identity_keeps_each_document_tables() {
    let mut engine = PublicEngine::empty();
    let id = DiagramId::extension("schema-probe");
    let creates = Arc::new(AtomicUsize::new(0));
    engine.register(Provider {
        schema: &SCHEMA_A,
        creates: creates.clone(),
    });
    let mut old = engine
        .parse_as(&id, "body", &ParseOptions::default())
        .unwrap();
    old.snapshot().unwrap();
    engine.register(Provider {
        schema: &SCHEMA_B,
        creates: creates.clone(),
    });
    let mut new = engine
        .parse_as(&id, "body", &ParseOptions::default())
        .unwrap();
    new.snapshot().unwrap();
    assert_eq!(old.model_schema().id, new.model_schema().id);
    assert_eq!(creates.load(Ordering::SeqCst), 2);
    drop(engine);
    for _ in 0..3 {
        assert_eq!(
            call(&mut old, 2).unwrap_err().message,
            "Public model method is unavailable"
        );
        assert_eq!(
            call(&mut new, 1).unwrap_err().message,
            "Public model method is unavailable"
        );
        assert!(matches!(
            call(&mut old, 1).unwrap().result,
            Ok(Atom::Number(1.))
        ));
        assert!(matches!(
            call(&mut new, 2).unwrap().result,
            Ok(Atom::Number(2.))
        ));
        for doc in [&mut old, &mut new] {
            let graph = doc.snapshot().unwrap();
            doc.apply(doc.revision(), graph).unwrap();
            doc.render_operation(doc.revision(), &RenderOptions::default(), &mut Host)
                .unwrap();
        }
    }
    assert_eq!(old.revision(), 9);
    assert_eq!(new.revision(), 9);
}
#[test]
fn invalid_registration_preserves_grammar_and_graph_error_precedence() {
    let mut engine = PublicEngine::empty();
    let creates = Arc::new(AtomicUsize::new(0));
    let id = DiagramId::extension("schema-probe");
    engine.register(Provider {
        schema: &INVALID,
        creates: creates.clone(),
    });
    assert_eq!(
        engine
            .parse_as(&id, "reject", &ParseOptions::default())
            .err()
            .unwrap()
            .message,
        "provider grammar rejected"
    );
    for _ in 0..3 {
        let mut invalid = GraphWriter::default().finish(Atom::Null).unwrap();
        invalid.root = Atom::Ref(u32::MAX);
        let error = engine
            .create_from_graph_as(&id, invalid, Object::default())
            .err()
            .unwrap();
        assert_eq!(error.message, "Invalid public model schema identity");
        assert_eq!(error.code, ErrorCode::Semantic);
    }
    assert_eq!(creates.load(Ordering::SeqCst), 0);
}
#[test]
fn public_schema_validators_observe_current_stack_schema_each_time() {
    let graph = GraphWriter::default().finish(Atom::Null).unwrap();
    let call = MethodCall {
        method: MethodId(1),
        receiver: Atom::Null,
        arguments: vec![],
    };
    let mut schema = SCHEMA_A;
    graph.validate_schema(&schema).unwrap();
    call.validate(&graph, &schema).unwrap();
    schema.methods = SCHEMA_B.methods;
    graph.validate_schema(&schema).unwrap();
    assert_eq!(
        call.validate(&graph, &schema).unwrap_err().message,
        "Public model method is unavailable"
    );
    schema.id = "";
    assert_eq!(
        graph.validate_schema(&schema).unwrap_err().message,
        "Invalid public model schema identity"
    );
    schema = SCHEMA_A;
    call.validate(&graph, &schema).unwrap();
}
