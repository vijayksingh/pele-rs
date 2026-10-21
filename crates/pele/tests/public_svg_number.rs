//! Complete public rendering output at coordinate formatting boundaries.
use pele::compat::{public::*, *};
use serde_json::Value;
struct Host;
impl RenderResources for Host {
    fn text_width(&mut self, r: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(r.text.encode_utf16().count() as f64
            * r.size
            * if r.style & 1 != 0 { 0.71 } else { 0.6 })
    }
}
fn number(bits: &Value) -> f64 {
    f64::from_bits(u64::from_str_radix(bits.as_str().unwrap(), 16).unwrap())
}
fn check(result: RenderResult, case: &Value) {
    assert_eq!(
        serde_json::to_value(&result).unwrap(),
        case["result"],
        "complete SVG {}",
        case["id"]
    );
    for (actual, expected) in [
        (result.rendered.width, number(&case["widthBits"])),
        (result.rendered.height, number(&case["heightBits"])),
    ] {
        assert!(
            (actual.is_nan() && expected.is_nan()) || actual.to_bits() == expected.to_bits(),
            "dimension {}",
            case["id"]
        );
    }
    assert!(result.rendered.svg.contains(&format!(
        "class=\"pele-rule-name\" x=\"{}\"",
        case["ruleNameX"].as_str().unwrap()
    )));
}
#[test]
fn railroad_coordinate_boundaries_preserve_all_public_render_lanes() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/pele/svg-number-boundaries.json"
    ))
    .unwrap();
    let engine = Engine::with_builtins();
    let source = fixture["source"].as_str().unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let mut options = RenderOptions::default();
        options.render.font_size = Some(13.);
        options.render.max_width = Some(220.);
        options.render.padding = Some(number(&case["paddingBits"]));
        options.render.now = Some(1704067200000.);
        check(
            engine
                .render_source_bounded(source, &options, Some(OutputLimit::Undefined), &mut Host)
                .unwrap(),
            case,
        );
        let immutable = engine.parse(source, &options.parse).unwrap();
        check(
            engine
                .render_document_bounded(
                    &immutable,
                    &options,
                    Some(OutputLimit::Undefined),
                    &mut Host,
                )
                .unwrap(),
            case,
        );
        let mut mutable = PublicEngine::with_builtins()
            .parse(source, &options.parse)
            .unwrap();
        let outcome = mutable
            .render_operation_bounded(0, &options, Some(OutputLimit::Undefined), &mut Host)
            .unwrap();
        check(outcome.result.unwrap(), case);
    }
}
