//! Public projection feature: builtins versus ordinary public registration.
//! Provider controls deliberately remain legacy; they do not exercise private
//! formatter arms unreachable from a parsed immutable Gantt source model.
use pele::compat::{
    diagrams::gantt::{Gantt, GanttModel},
    text::MetricsResources,
    *,
};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};
use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

fn legacy() -> Engine {
    let mut engine = Engine::empty();
    engine.register(Gantt);
    engine
}
fn exact(a: &Value, b: &Value) {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a.to_bits(), b.to_bits()),
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            for (a, b) in a.iter().zip(b) {
                exact(a, b);
            }
        }
        (Value::Object(a), Value::Object(b)) => {
            let a = a.entries();
            let b = b.entries();
            assert_eq!(a.len(), b.len());
            for ((ak, av), (bk, bv)) in a.into_iter().zip(b) {
                assert_eq!(ak.as_bytes(), bk.as_bytes());
                exact(av, bv);
            }
        }
        _ => assert_eq!(a, b),
    }
}
fn assert_projection(source: &str, options: &ParseOptions) {
    let actual = Engine::with_builtins().parse(source, options).unwrap();
    let expected = legacy().parse(source, options).unwrap();
    exact(
        &actual.model_view().unwrap(),
        &expected.model_view().unwrap(),
    );
    assert_eq!(
        document_envelope(&actual).unwrap(),
        document_envelope(&expected).unwrap()
    );
}
#[test]
fn gantt_builtin_projection_matches_public_registration_and_retained_render() {
    let fixtures: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/pele/gantt-construction.json"
    ))
    .unwrap();
    for fixture in fixtures["cases"].as_array().unwrap() {
        let source = fixture["source"].as_str().unwrap();
        let mut options = render_options(&fixture["options"].to_string()).unwrap();
        options.render.now = Some(1704067200000.0);
        assert_projection(source, &options.parse);
        let builtin = Engine::with_builtins();
        let old = legacy();
        let a = builtin.parse(source, &options.parse).unwrap();
        let b = old.parse(source, &options.parse).unwrap();
        let before = a.model_view().unwrap();
        assert_eq!(
            builtin
                .render_document(&a, &options, &mut MetricsResources)
                .unwrap(),
            old.render_document(&b, &options, &mut MetricsResources)
                .unwrap(),
        );
        exact(&a.model_view().unwrap(), &before);
    }
}
#[test]
fn gantt_projection_preserves_bom_escapes_duplicate_ids_and_missing_fields() {
    let mut options = ParseOptions::default();
    let mut gantt = Object::default();
    gantt.insert(
        "displayMode".into(),
        Value::String("\u{feff}é e\u{301} 🚀 \0 \u{8}\u{c}\n\r\t \\\"".into()),
    );
    options
        .preparation
        .config
        .insert("gantt".into(), Value::Object(gantt));
    assert_projection("gantt\ndateFormat YYYY-MM-DD\nsection 🚀\nFirst:a,2024-01-01,1d\nclick a call handler()\nReplacement:a,2024-01-02,2d\nNext:after a,3d\n", &options);
    assert_projection("gantt\n", &options);
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
fn settings(mode: &str) -> ParseOptions {
    let mut options = ParseOptions::default();
    options.preparation.limit = MAX_MODEL_JSON_BYTES;
    let mut gantt = Object::default();
    gantt.insert("displayMode".into(), Value::String(mode.into()));
    options
        .preparation
        .config
        .insert("gantt".into(), Value::Object(gantt));
    options
}
fn task_source(label: usize) -> String {
    let mut source = String::with_capacity(label + 64);
    source.push_str("gantt\ndateFormat YYYY-MM-DD\n");
    source.extend(std::iter::repeat_n('L', label));
    source.push_str(":task,2024-01-01,1d\n");
    source
}
#[test]
fn gantt_projection_exact_sixteen_mib_admission_matches_legacy() {
    let old = legacy();
    let small = old.parse(&task_source(1), &settings("")).unwrap();
    let mut size = Size::default();
    serde_json::to_writer(&mut size, small.model::<GanttModel>().unwrap()).unwrap();
    let base = size.0;
    drop(small);
    // Task labels appear twice, in tasks and byId. One displayMode ASCII byte
    // selects either parity, so these hit exactly limit-1, limit and limit+1.
    // Each large case is dropped before the next; no huge assertion dumps.
    for target in [
        MAX_MODEL_JSON_BYTES - 1,
        MAX_MODEL_JSON_BYTES,
        MAX_MODEL_JSON_BYTES + 1,
    ] {
        let adjustment = (target - base) % 2;
        let options = settings(if adjustment == 0 { "" } else { "x" });
        let source = task_source(1 + (target - base - adjustment) / 2);
        assert!(source.len() < MAX_PUBLIC_SOURCE_BYTES);
        let expected = old.parse(&source, &options).unwrap();
        let mut size = Size::default();
        serde_json::to_writer(&mut size, expected.model::<GanttModel>().unwrap()).unwrap();
        assert_eq!(
            size.0, target,
            "fixture must hit exact serialized byte boundary"
        );
        let expected_error = match expected.model_view() {
            Ok(value) => {
                assert!(target <= MAX_MODEL_JSON_BYTES);
                drop(value);
                None
            }
            Err(error) => {
                assert!(target > MAX_MODEL_JSON_BYTES);
                Some(error)
            }
        };
        drop(expected);
        let actual = Engine::with_builtins().parse(&source, &options).unwrap();
        match actual.model_view() {
            Ok(value) => {
                assert!(expected_error.is_none());
                drop(value);
            }
            Err(error) => assert_eq!(Some(error), expected_error),
        }
    }
}

#[derive(Clone)]
enum Mode {
    Scalars,
    Raw,
    Depth(usize),
    MalformedKey,
    LateError,
    DepthThenError,
    BudgetThenError,
    Date,
}
struct ProbeModel {
    mode: Mode,
    calls: Arc<AtomicUsize>,
}
struct Deep(usize);
impl Serialize for Deep {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        if self.0 == 0 {
            return s.serialize_unit();
        }
        let mut values = s.serialize_seq(Some(1))?;
        values.serialize_element(&Deep(self.0 - 1))?;
        values.end()
    }
}
impl Serialize for ProbeModel {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        match &self.mode {
            Mode::Depth(depth) => Deep(*depth).serialize(s),
            Mode::Raw => serde_json::value::RawValue::from_string(
                r#"{"same":1,"same":2,"2":2,"1":1,"é":3,"e\u0301":4,"zero":-0,"large":1e400,"bom":"\ufeffA"}"#.into()
            ).unwrap().serialize(s),
            Mode::Scalars => {
                let mut map = s.serialize_map(None)?;
                map.serialize_entry("f32", &0.1_f32)?;
                map.serialize_entry("negativeZero", &-0.0_f64)?;
                map.serialize_entry("minimum", &f64::from_bits(1))?;
                map.serialize_entry("maximum", &f64::MAX)?;
                map.serialize_entry("nonfinite", &[f64::NAN, f64::INFINITY, f64::NEG_INFINITY])?;
                map.serialize_entry("u128", &u128::MAX)?;
                map.serialize_entry("i128", &i128::MIN)?;
                map.serialize_entry(&123_u64, &"numeric key")?;
                map.serialize_entry(&true, &"bool key")?;
                map.serialize_entry("duplicate", &1)?;
                map.serialize_entry("duplicate", &2)?;
                map.end()
            }
            Mode::Date => {
                let mut model = GanttModel::new(Some("\u{feff}Date projection"), String::new());
                model.add_task("Task".into(), ":task,2024-01-01,1d".into()).unwrap();
                model.tasks[0].start_time = Some(-0.0);
                model.tasks[0].end_time = Some(f64::from_bits(0x7ff8_0000_0000_0042));
                model.tasks[0].render_end_time = Some(1704067200000.0);
                model.tasks[0].raw.start = None;
                model.tasks[0].classes = vec!["é".into(), "\u{feff}e\u{301}".into()];
                model.serialize(s)
            }
            mode => {
                let mut map = s.serialize_map(None)?;
                match mode {
                    Mode::MalformedKey => map.serialize_entry(&vec![1, 2], &true)?,
                    Mode::DepthThenError => map.serialize_entry("deep", &Deep(129))?,
                    Mode::BudgetThenError => {
                        // Deliberately swallow the writer failure. Historical
                        // model byte admission must still outrank this error.
                        let _ = map.serialize_entry("big", &"x".repeat(MAX_MODEL_JSON_BYTES + 1));
                    }
                    _ => map.serialize_entry("before", &42)?,
                }
                Err(serde::ser::Error::custom("provider's late serialization failure"))
            }
        }
    }
}
struct Probe {
    mode: Mode,
    calls: Arc<AtomicUsize>,
}
impl DiagramModule for Probe {
    type Model = ProbeModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Gantt)
    }
    fn parse(&self, _: ParseContext<'_>) -> Result<ProbeModel, PeleError> {
        Ok(ProbeModel {
            mode: self.mode.clone(),
            calls: self.calls.clone(),
        })
    }
    fn render(
        &self,
        _: &ProbeModel,
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
fn provider_result(mode: Mode, builtins: bool) -> (Result<Value, PeleError>, usize) {
    let calls = Arc::new(AtomicUsize::new(0));
    let mut engine = if builtins {
        Engine::with_builtins()
    } else {
        Engine::empty()
    };
    engine.register(Probe {
        mode,
        calls: calls.clone(),
    });
    let document = engine.parse("gantt", &ParseOptions::default()).unwrap();
    let result = document.model_view();
    (result, calls.load(Ordering::SeqCst))
}
#[test]
fn public_providers_keep_raw_numbers_dates_duplicates_and_single_serialization() {
    for mode in [Mode::Scalars, Mode::Raw, Mode::Date, Mode::Depth(128)] {
        let (actual, calls) = provider_result(mode.clone(), true);
        let (expected, old_calls) = provider_result(mode, false);
        assert_eq!((calls, old_calls), (1, 1));
        exact(&actual.unwrap(), &expected.unwrap());
    }
}
#[test]
fn provider_error_and_budget_precedence_stays_legacy_without_retries() {
    for mode in [
        Mode::Depth(129),
        Mode::MalformedKey,
        Mode::LateError,
        Mode::DepthThenError,
        Mode::BudgetThenError,
    ] {
        let (actual, calls) = provider_result(mode.clone(), true);
        let (expected, old_calls) = provider_result(mode.clone(), false);
        assert_eq!((calls, old_calls), (1, 1));
        let actual = actual.unwrap_err();
        assert_eq!(actual, expected.unwrap_err());
        match mode {
            Mode::Depth(129) => assert_eq!(
                actual.message,
                "Module model cannot be serialized: model nesting is too deep"
            ),
            Mode::MalformedKey => assert!(actual.message.contains("key must be a string")),
            Mode::BudgetThenError => {
                assert_eq!(actual.code, ErrorCode::Limit);
                assert_eq!(
                    actual.message,
                    format!("Parsed model JSON exceeds {MAX_MODEL_JSON_BYTES} bytes")
                );
            }
            _ => assert!(actual
                .message
                .contains("provider's late serialization failure")),
        }
    }
}
#[test]
fn captured_builtin_projection_survives_provider_replacement() {
    let mut engine = Engine::with_builtins();
    let source = "gantt\nA:a,2024-01-01,1d\n";
    let document = engine.parse(source, &ParseOptions::default()).unwrap();
    let expected = legacy()
        .parse(source, &ParseOptions::default())
        .unwrap()
        .model_view()
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    engine.register(Probe {
        mode: Mode::Raw,
        calls: calls.clone(),
    });
    let replacement = engine.parse("gantt", &ParseOptions::default()).unwrap();
    drop(engine);
    exact(&document.model_view().unwrap(), &expected);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let value = replacement.model_view().unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        value.as_object().unwrap().get("same"),
        Some(&Value::Number(2.0))
    );
}
