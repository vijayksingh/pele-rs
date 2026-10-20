//! Captured raw ER selection: no parser, graph, or host object enters binding.
#![cfg(feature = "web-host-er")]
use pele::compat::{
    diagrams::er::{Er, ErModel},
    public::*,
    *,
};

struct Replacement;
impl DiagramModule for Replacement {
    type Model = ErModel;
    fn descriptor(&self) -> ModuleDescriptor {
        Er.descriptor()
    }
    fn parse(&self, _: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        panic!("raw selection parsed")
    }
    fn render(
        &self,
        m: &ErModel,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Er.render(m, c, o, h)
    }
}
impl PublicDiagramModule for Replacement {
    type PublicState = <Er as PublicDiagramModule>::PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        Er.model_schema()
    }
    fn create_public(&self, _: &ErModel) -> Result<Self::PublicState, PeleError> {
        panic!("raw selection adapted")
    }
    fn create_public_from_graph(&self, _: ModelGraph) -> Result<Self::PublicState, PeleError> {
        panic!("raw selection allocated graph")
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Er.render_public(s, c, o, h)
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Er.export_public(s)
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        Er.apply_public(s, g)
    }
}
#[test]
fn raw_builtin_selection_retains_exact_policy_across_replacement_and_engine_drop() {
    let mut engine = PublicEngine::with_builtins();
    let id = DiagramId::from(DiagramType::Er);
    let selected = engine.select_host_er_entry(&id).unwrap().unwrap();
    let schema = serde_json::to_vec(&selected.schema()).unwrap();
    assert_eq!(
        schema,
        serde_json::to_vec(&engine.schema_for(&id).unwrap()).unwrap()
    );
    engine.register(Replacement);
    assert!(engine.select_host_er_entry(&id).unwrap().is_none());
    assert_eq!(serde_json::to_vec(&selected.schema()).unwrap(), schema);
    drop(engine);
    let config: Object =
        serde_json::from_str(r#"{"er":{"nodeSpacing":79},"stamp":"captured"}"#).unwrap();
    let policy = selected.bind(config.clone()).unwrap();
    assert_eq!(policy.config(), &config);
    drop(selected);
    assert_eq!(policy.config(), &config);
}
#[test]
fn raw_selection_keeps_other_codecs_and_unknown_type_errors_on_the_legacy_path() {
    let engine = PublicEngine::with_builtins();
    for kind in DIAGRAM_TYPES {
        let selected = engine.select_host_er_entry(&kind.into()).unwrap();
        assert_eq!(selected.is_some(), kind == DiagramType::Er);
    }
    let missing = DiagramId::extension("raw-selection-missing");
    let expected = engine.schema_for(&missing).unwrap_err();
    let actual = match engine.select_host_er_entry(&missing) {
        Err(error) => error,
        Ok(_) => panic!("unknown selection succeeded"),
    };
    assert_eq!(actual, expected);
    let empty = PublicEngine::empty();
    let id = DiagramType::Er.into();
    let expected = empty.schema_for(&id).unwrap_err();
    let actual = match empty.select_host_er_entry(&id) {
        Err(error) => error,
        Ok(_) => panic!("unregistered selection succeeded"),
    };
    assert_eq!(actual, expected);
}
