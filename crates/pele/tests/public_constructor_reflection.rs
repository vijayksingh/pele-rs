//! Captured builtin reflection survives registry lifetime and authenticates class identity.
use pele::compat::public::*;
use serde_json::Value;
#[test]
fn all_captured_builtin_classes_keep_pinned_reflection_after_registry_drop() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/public-constructor-reflection.json"
    ))
    .unwrap();
    let engine = PublicEngine::with_builtins();
    let mut captured = Vec::new();
    for row in fixture["rows"].as_array().unwrap() {
        let id = serde_json::from_value(row["family"].clone()).unwrap();
        let prototype = PrototypeId(row["prototype"].as_u64().unwrap() as u32);
        let factory = engine.capture_constructor(&id, prototype).unwrap().unwrap();
        captured.push((factory, row.clone()));
    }
    drop(engine);
    assert_eq!(captured.len(), 23);
    for (factory, row) in captured {
        let reflection = serde_json::to_value(factory.reflection().unwrap()).unwrap();
        for key in ["prototype", "identity", "monolithName", "moduleName"] {
            assert_eq!(reflection[key], row[key], "{} / {key}", row["family"]);
        }
        if factory.prototype() == PrototypeId(1) {
            let made = factory.construct().unwrap();
            let again = made.capture_constructor(factory.prototype()).unwrap();
            assert_eq!(
                serde_json::to_value(again.reflection().unwrap()).unwrap(),
                reflection
            );
            assert!(made.is_constructor_created());
        }
    }
}

#[cfg(feature = "web-host-er")]
#[test]
fn host_publication_carries_selected_constructor_before_any_object_is_published() {
    use pele::compat::{host_er::Parsed, *};
    let engine = PublicEngine::with_builtins();
    let mut factories = Vec::new();
    // Direct parser, its typed fallback, and raw prepared publication all supply
    // their captured creator before the writer can publish a prototype.
    for source in [
        "erDiagram\nA",
        "erDiagram\nA\nclassDef x fill:red\nclass A x",
    ] {
        let parsed = engine
            .parse_host_er_with_constructor(
                source,
                &ParseOptions::default(),
                ParseLimits::default(),
                |schema, factory| {
                    let factory = factory.expect("authenticated host writer factory");
                    assert_eq!(
                        serde_json::to_value(schema).unwrap(),
                        serde_json::to_value(factory.schema_view()).unwrap()
                    );
                    assert_eq!(factory.reflection().unwrap().module_name, "Ae");
                    factories.push(factory);
                    Ok(GraphWriter::default())
                },
            )
            .unwrap();
        assert!(matches!(parsed, Parsed::Host { .. }));
    }
    let config = Object::default();
    let parsed = engine
        .parse_prepared_host_er_with_constructor(
            &DiagramType::Er.into(),
            ParseContext {
                source: "erDiagram\nA",
                config: &config,
                title: None,
                line_offset: 0,
                max_edges: DEFAULT_MAX_EDGES,
            },
            EdgeLimit::Undefined,
            |schema, factory| {
                let factory = factory.expect("authenticated prepared factory");
                assert_eq!(
                    serde_json::to_value(schema).unwrap(),
                    serde_json::to_value(factory.schema_view()).unwrap()
                );
                factories.push(factory);
                Ok(GraphWriter::default())
            },
        )
        .unwrap();
    assert!(matches!(parsed, Parsed::Host { .. }));
    drop(engine);
    assert!(factories.len() >= 3);
    for factory in factories {
        assert_eq!(factory.reflection().unwrap().monolith_name, "ja");
        assert!(factory.construct().unwrap().is_constructor_created());
    }
}
