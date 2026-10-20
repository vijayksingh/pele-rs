//! Public immutable ER projection, including the captured builtin policy.
//! Ordinary public registration remains the independent legacy normalization lane.
use pele::compat::{
    diagrams::er::{Er, ErModel},
    text::MetricsResources,
    *,
};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

fn legacy() -> Engine {
    let mut engine = Engine::empty();
    engine.register(Er);
    engine
}
fn exact(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => a.to_bits() == b.to_bits(),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| exact(a, b))
        }
        (Value::Object(a), Value::Object(b)) => {
            let a = a.entries();
            let b = b.entries();
            a.len() == b.len()
                && a.into_iter()
                    .zip(b)
                    .all(|((ak, av), (bk, bv))| ak.as_bytes() == bk.as_bytes() && exact(av, bv))
        }
        _ => a == b,
    }
}
fn compare_documents(a: &ParsedDocument, b: &ParsedDocument, id: &str) {
    match (a.model_view(), b.model_view()) {
        (Ok(a), Ok(b)) => assert!(exact(&a, &b), "model projection differs: {id}"),
        (Err(a), Err(b)) => assert!(a == b, "projection error differs: {id}"),
        _ => panic!("projection acceptance differs: {id}"),
    }
    // String equality also pins field order and exact envelope escaping.
    // Boolean assertions intentionally avoid dumping large values on failure.
    assert!(
        document_envelope(a) == document_envelope(b),
        "document envelope differs: {id}"
    );
}

#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    source: String,
    options: ParseOptions,
}
#[test]
fn whole_er_corpus_matches_legacy_projection_and_complete_errors() {
    // Ignore the large expected SVG/model fields during deserialization. The
    // existing oracle suite pins grammar; this slice crosses both actual
    // model_view policies for every frozen source and parse-option combination.
    let corpus: Corpus =
        serde_json::from_str(include_str!("../../../fixtures/pele/er.json")).unwrap();
    assert_eq!(corpus.cases.len(), 17_437);
    let builtin = Engine::with_builtins();
    let old = legacy();
    let mut accepted = 0;
    let mut rejected = 0;
    for case in corpus.cases {
        match (
            builtin.parse(&case.source, &case.options),
            old.parse(&case.source, &case.options),
        ) {
            (Ok(a), Ok(b)) => {
                accepted += 1;
                compare_documents(&a, &b, &case.id);
            }
            (Err(a), Err(b)) => {
                rejected += 1;
                assert!(a == b, "parse error differs: {}", case.id);
                assert!(
                    error_envelope(&a) == error_envelope(&b),
                    "error envelope differs: {}",
                    case.id
                );
            }
            _ => panic!("parse acceptance differs: {}", case.id),
        }
    }
    assert!(accepted > 0 && rejected > 0);
}

#[test]
fn er_projection_preserves_order_unicode_duplicates_and_retained_render() {
    let builtin = Engine::with_builtins();
    let old = legacy();
    // Quoted numeric-looking IDs remain strings in ordered pair arrays. Quote
    // standalone 1 so the following declaration cannot make it a cardinality.
    // Alias starts with a BOM; the comment requires JSON control escaping.
    let source = concat!(
        "erDiagram\n",
        "\"2\"[\"\u{feff}é e\u{301} 🚀\"] {\n",
        " string name \"\u{feff}comment \0\u{8}\u{c}\t\\path\"\n}\n",
        "\"10\"\n\"1\"\n\"2\"[\"ignored replacement\"] { int id PK }\n",
        "\"2\" ||--o{ \"10\" : places\n",
        "classDef first fill:red,color:blue\n",
        "classDef second fill:green\n",
        "classDef first stroke:black\n",
    );
    let a = builtin.parse(source, &ParseOptions::default()).unwrap();
    let b = old.parse(source, &ParseOptions::default()).unwrap();
    let typed = a.model::<ErModel>().unwrap();
    assert_eq!(
        typed
            .entities
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>(),
        ["2", "10", "1"]
    );
    assert_eq!(typed.entities[0].1.alias, "\u{feff}é e\u{301} 🚀");
    assert_eq!(typed.entities[0].1.attributes.len(), 2);
    assert_eq!(
        typed.entities[0].1.attributes[0].comment,
        "\u{feff}comment \0\u{8}\u{c}\t\\path"
    );
    assert!(typed.title.is_none() && typed.acc_title.is_none());
    assert_eq!(typed.classes.len(), 2);
    assert_eq!(typed.classes[0].0, "first");
    assert_eq!(
        typed.classes[0].1.styles,
        ["fill:red", "color:blue", "stroke:black"]
    );
    compare_documents(&a, &b, "ordered-unicode-duplicate");

    // Rendering uses a separate small valid string model: embedded XML control
    // characters above test projection, not SVG acceptance or font behavior.
    let source = concat!(
        "erDiagram\naccTitle: Records\nA { int id PK }\nA ||--o{ B : owns\n",
        "classDef first fill:red\nclass A first\nclass A first\n",
    );
    let a = builtin.parse(source, &ParseOptions::default()).unwrap();
    let b = old.parse(source, &ParseOptions::default()).unwrap();
    // CLASS accepts identifier tokens, not numeric or quoted entity-name
    // tokens. Exercise repeated class assignment with a grammar-valid ID.
    assert_eq!(
        a.model::<ErModel>().unwrap().entities[0].1.css_classes,
        "default first first"
    );
    let before = a.model_view().unwrap();
    let options = RenderOptions::default();
    let actual_render = builtin
        .render_document(&a, &options, &mut MetricsResources)
        .unwrap();
    let expected_render = old
        .render_document(&b, &options, &mut MetricsResources)
        .unwrap();
    assert!(actual_render == expected_render);
    assert!(exact(&a.model_view().unwrap(), &before));
    compare_documents(&a, &b, "after-retained-render");
}

#[derive(Default)]
struct Size(usize);
impl Write for Size {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn size(document: &ParsedDocument) -> usize {
    let mut out = Size::default();
    serde_json::to_writer(&mut out, document.model::<ErModel>().unwrap()).unwrap();
    out.0
}
fn entity_source(name_length: usize, title_padding: usize) -> String {
    let mut source = String::with_capacity(name_length + title_padding + 32);
    source.push_str("erDiagram\n");
    source.extend(std::iter::repeat_n('N', name_length));
    source.push_str("\naccTitle: x");
    source.extend(std::iter::repeat_n('p', title_padding));
    source.push('\n');
    source
}
#[test]
fn er_exact_model_byte_admission_and_later_syntax_error_match_legacy() {
    let old = legacy();
    let builtin = Engine::with_builtins();
    let mut options = ParseOptions::default();
    options.preparation.limit = MAX_MODEL_JSON_BYTES;
    let base = size(&old.parse(&entity_source(1, 0), &options).unwrap());
    let step = size(&old.parse(&entity_source(2, 0), &options).unwrap()) - base;
    // One entity name is published as pair key, ID suffix, and label. This
    // source expands threefold; independent source admission stays far below
    // 16 MiB. Short accTitle padding selects the exact byte-boundary residue.
    assert_eq!(step, 3);
    for target in [
        MAX_MODEL_JSON_BYTES - 1,
        MAX_MODEL_JSON_BYTES,
        MAX_MODEL_JSON_BYTES + 1,
    ] {
        let padding = (target - base) % step;
        let mut source = entity_source(1 + (target - base - padding) / step, padding);
        assert!(source.len() < MAX_PUBLIC_SOURCE_BYTES);
        let expected = old.parse(&source, &options).unwrap();
        assert_eq!(
            size(&expected),
            target,
            "fixture must hit exact byte boundary"
        );
        let expected_error = match expected.model_view() {
            Ok(value) => {
                assert!(target <= MAX_MODEL_JSON_BYTES);
                drop(value);
                None
            }
            Err(error) => {
                assert!(target > MAX_MODEL_JSON_BYTES);
                assert_eq!(error.code, ErrorCode::Limit);
                assert_eq!(error.diagram_type.as_deref(), Some("er"));
                assert_eq!(
                    error.message,
                    format!("Parsed model JSON exceeds {MAX_MODEL_JSON_BYTES} bytes")
                );
                Some(error)
            }
        };
        drop(expected);
        let actual = builtin.parse(&source, &options).unwrap();
        match actual.model_view() {
            Ok(value) => {
                assert!(expected_error.is_none());
                drop(value);
            }
            Err(error) => assert!(
                Some(error) == expected_error,
                "byte admission error differs"
            ),
        }
        drop(actual);
        if target > MAX_MODEL_JSON_BYTES {
            // Projection is not attempted until grammar succeeds. A later
            // short syntax error must beat the otherwise oversized model.
            source.push_str("A { int }\n");
            let expected = old.parse(&source, &options).err().expect("syntax error");
            let actual = builtin
                .parse(&source, &options)
                .err()
                .expect("syntax error");
            assert_eq!(expected.code, ErrorCode::Syntax);
            assert!(actual == expected, "syntax/projection precedence differs");
        }
    }
}

struct ReplacementModel {
    calls: Arc<AtomicUsize>,
    fail_validation: bool,
}
impl Serialize for ReplacementModel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        serde_json::value::RawValue::from_string(
            r#"{"same":1,"same":2,"2":"two","1":"one","bom":"\ufeffA","zero":-0}"#.into(),
        )
        .unwrap()
        .serialize(serializer)
    }
}
struct Replacement {
    calls: Arc<AtomicUsize>,
    fail_validation: bool,
}
impl DiagramModule for Replacement {
    type Model = ReplacementModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Er)
    }
    fn parse(&self, _: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        Ok(ReplacementModel {
            calls: self.calls.clone(),
            fail_validation: self.fail_validation,
        })
    }
    fn validate_model_projection(&self, model: &Self::Model) -> Result<(), PeleError> {
        if model.fail_validation {
            Err(PeleError::new(
                "replacement validation",
                ErrorCode::Semantic,
            ))
        } else {
            Ok(())
        }
    }
    fn render(
        &self,
        _: &Self::Model,
        _: &Object,
        _: &RenderSettings,
        _: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        Ok(Rendered {
            svg: "<svg/>".into(),
            width: 1.0,
            height: 1.0,
            links: vec![],
        })
    }
}
#[test]
fn er_policy_is_captured_and_public_replacement_remains_single_pass_legacy() {
    let mut builtin = Engine::with_builtins();
    let old = legacy();
    let source = "erDiagram\nA ||--o{ B : owns\n";
    let captured = builtin.parse(source, &ParseOptions::default()).unwrap();
    let expected = old.parse(source, &ParseOptions::default()).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    builtin.register(Replacement {
        calls: calls.clone(),
        fail_validation: false,
    });
    let replacement = builtin
        .parse("erDiagram", &ParseOptions::default())
        .unwrap();
    builtin.register(Replacement {
        calls: calls.clone(),
        fail_validation: true,
    });
    let rejected = builtin
        .parse("erDiagram", &ParseOptions::default())
        .unwrap();
    drop(builtin);
    compare_documents(&captured, &expected, "captured-er-after-replacement");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let projected = replacement.model_view().unwrap();
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "public raw provider must not retry"
    );
    let object = projected.as_object().unwrap();
    assert_eq!(object.get("same"), Some(&Value::Number(2.0)));
    assert_eq!(object.get("bom"), Some(&Value::String("\u{feff}A".into())));
    let error = rejected.model_view().unwrap_err();
    assert_eq!(error.message, "replacement validation");
    assert_eq!(error.diagram_type.as_deref(), Some("er"));
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "validation must precede serialization"
    );
}
