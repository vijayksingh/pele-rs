//! SVG document assembly through source, retained and mutable public interfaces.
use pele::compat::{public::PublicEngine, *};
use serde_json::Value;

struct Host<'a>(&'a str);
impl RenderResources for Host<'_> {
    fn text_width(&mut self, request: TextRequest<'_>) -> Result<f64, PeleError> {
        Ok(request.text.encode_utf16().count() as f64
            * request.size
            * if request.style & 1 != 0 { 0.71 } else { 0.6 })
    }
    fn icon(&mut self, _: &str) -> Result<Option<TrustedSvg>, PeleError> {
        Ok(Some(TrustedSvg::from_trusted_markup(self.0.to_owned())))
    }
}

#[test]
fn complete_svg_root_output_matches_pinned_pele_through_public_render_lanes() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/pele/svg-root.json")).unwrap();
    let engine = Engine::with_builtins();
    let public = PublicEngine::with_builtins();
    for case in fixture["cases"].as_array().unwrap() {
        let source = case["source"].as_str().unwrap();
        let mut options: RenderOptions = serde_json::from_value(case["options"].clone()).unwrap();
        options.render.now = Some(1704067200000.);
        let mut host = Host(case["markup"].as_str().unwrap());
        let check = |result: RenderResult| {
            assert_eq!(
                serde_json::to_value(result).unwrap(),
                case["result"],
                "{}",
                case["id"]
            );
        };
        check(engine.render_source(source, &options, &mut host).unwrap());
        let typed = engine.parse(source, &options.parse).unwrap();
        check(engine.render_document(&typed, &options, &mut host).unwrap());
        let mut mutable = public.parse(source, &options.parse).unwrap();
        check(
            mutable
                .render_operation(0, &options, &mut host)
                .unwrap()
                .result
                .unwrap(),
        );
    }
}
