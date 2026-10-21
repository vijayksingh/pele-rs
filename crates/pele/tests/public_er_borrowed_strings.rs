//! Public ER source publication retains exact string tables across borrowed
//! native construction and a separate sink using the public owned conversion API.
#![cfg(feature = "web-host-er")]
use pele::compat::{
    host_er::{InitialWriter, Parsed},
    public::*,
    *,
};

struct OwnedSink(GraphWriter);
impl InitialWriter for OwnedSink {
    type Output = ModelGraph;
    fn string(&mut self, value: &str) -> Result<u32, PeleError> {
        // Deliberately use the inherent generic API, not InitialWriter::string.
        // This remains the independent owned-conversion publication route.
        Ok(GraphWriter::string(&mut self.0, value))
    }
    fn node(&mut self, node: Node) -> Result<u32, PeleError> {
        Ok(self.0.node(node))
    }
    fn finish(self, root: Atom) -> Result<ModelGraph, PeleError> {
        self.0.finish(root)
    }
}
fn owned(
    engine: &PublicEngine,
    source: &str,
    limits: ParseLimits,
) -> Result<ModelGraph, PeleError> {
    match engine.parse_host_er_with_limits(source, &ParseOptions::default(), limits, |_| {
        Ok(OwnedSink(GraphWriter::default()))
    })? {
        Parsed::Host { output, .. } => Ok(output),
        Parsed::Arena(_) => panic!("builtin ER must use the publication sink"),
    }
}
fn encoded(graph: &ModelGraph) -> Vec<u8> {
    encode_binary_exchange(0, graph, br#"{"type":"er"}"#, DEFAULT_BINARY_BUDGET).unwrap()
}
#[test]
fn repeated_er_text_preserves_full_graph_at_utf8_scratch_boundaries() {
    let engine = PublicEngine::with_builtins();
    for value in [
        "a".repeat(64),
        "a".repeat(65),
        "é".repeat(32),
        "é".repeat(33),
        "🦀".repeat(16),
        "🦀".repeat(17),
        "日本語".to_owned(),
    ] {
        let source = format!(
            "erDiagram\nA[\"{value}\"] {{\n string label PK \"{value}\"\n string other \"{value}\"\n}}\nB[\"{value}\"]\nA ||--o{{ B : owns"
        );
        // The second source forces typed fallback before publication, while
        // retaining the repeated aliases/comments and their insertion order.
        for source in [source.clone(), format!("{source}\nstyle A fill:red")] {
            let expected = owned(&engine, &source, ParseLimits::default())
                .expect("boundary source must parse successfully");
            let document = engine.parse(&source, &ParseOptions::default()).unwrap();
            let actual = document.snapshot_view().unwrap();
            assert_eq!(document.revision(), 0);
            assert!(
                encoded(actual.as_ref()) == encoded(&expected),
                "complete public graph differs for {} UTF8 bytes",
                value.len()
            );
            assert_eq!(
                actual.strings.iter().filter(|s| s.is(&value)).count(),
                1,
                "repeated alias/comment must have one canonical string ID"
            );
        }
    }
}
#[test]
fn borrowed_publication_preserves_source_admission_and_late_syntax_errors() {
    let engine = PublicEngine::with_builtins();
    let source = "erDiagram\nA { string label PK \"repeated\" }\nB { int }";
    for limits in [
        ParseLimits::default(),
        ParseLimits {
            source: Some(SourceLimit::Null),
            max_edges: None,
        },
    ] {
        let expected = owned(&engine, source, limits).unwrap_err();
        let actual = engine
            .parse_with_limits(source, &ParseOptions::default(), limits)
            .err()
            .expect("malformed source must fail");
        assert_eq!(actual, expected);
    }
}
