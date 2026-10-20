//! Public parse fallback preserves successful graph bytes and later parser errors.
//! Both large fixtures run sequentially so their graphs/token buffers never coexist.
use pele::compat::{
    diagrams::gantt::{Gantt, GanttModel},
    public::{encode_binary_exchange, PublicDocument, PublicEngine, DEFAULT_BINARY_BUDGET},
    Engine, ErrorCode, ParseOptions,
};

fn options() -> ParseOptions {
    let mut options = ParseOptions::default();
    options.preparation.limit = 10_000_000;
    options
}
fn bytes(document: &PublicDocument) -> Vec<u8> {
    encode_binary_exchange(
        document.revision(),
        &document.snapshot_view().unwrap(),
        &[],
        DEFAULT_BINARY_BUDGET,
    )
    .unwrap()
}

#[test]
fn gantt_direct_storage_fallback_preserves_legacy_success_and_error_precedence() {
    let options = options();
    {
        // The final label exceeds the direct target's conservative 4M-unit
        // eligibility bound, while remaining within every actual graph limit.
        // A successful legacy fallback must retain its complete original table
        // order, including IDs and unreferenced strings, not only visible fields.
        let mut source = String::with_capacity(4_000_064);
        source.push_str("gantt\ndateFormat YYYY-MM-DD\n");
        source.extend(std::iter::repeat_n('L', 4_000_001));
        source.push_str(":a,2024-01-01,1d\n");
        assert!(source.len() < 16 * 1024 * 1024);
        let expected = {
            let parsed = Engine::with_builtins().parse(&source, &options).unwrap();
            PublicDocument::from_module(
                Gantt,
                parsed.model::<GanttModel>().unwrap(),
                parsed.config().clone(),
            )
            .unwrap()
        };
        let actual = PublicEngine::with_builtins()
            .parse(&source, &options)
            .unwrap();
        assert_eq!(actual.revision(), 0);
        // Deliberately avoid assert_eq! on multi-megabyte Vecs: diagnostics must
        // not dump the entire transport if a regression occurs.
        assert!(
            bytes(&actual) == bytes(&expected),
            "fallback graph differs from typed adaptation"
        );
    }
    {
        // The ID occurs in raw.data and in id, so two distinct strings already
        // exceed 16M UTF16 units. This construction failure occurs while the
        // grammar has not yet reached the malformed trailing click statement.
        // Source stays below the independent 16MiB byte limit. The same valid
        // prefix proves this fixture reaches graph admission through public APIs.
        let mut source = String::with_capacity(8_000_080);
        source.push_str("gantt\ndateFormat YYYY-MM-DD\nA:");
        source.extend(std::iter::repeat_n('i', 8_000_001));
        source.push_str(",2024-01-01,1d\n");
        assert!(source.len() < 16 * 1024 * 1024);
        {
            let parsed = Engine::with_builtins().parse(&source, &options).unwrap();
            let admission = PublicDocument::from_module(
                Gantt,
                parsed.model::<GanttModel>().unwrap(),
                parsed.config().clone(),
            )
            .err()
            .expect("the valid prefix must exceed native graph string admission");
            assert_eq!(admission.code, ErrorCode::Limit);
            assert_eq!(
                admission.message,
                "Gantt native graph operation exceeds work budget"
            );
        }
        source.push_str("click a\n");
        let expected = Engine::with_builtins()
            .parse(&source, &options)
            .err()
            .expect("trailing click must be a parser error");
        assert_eq!(expected.code, ErrorCode::Syntax);
        let actual = PublicEngine::with_builtins()
            .parse(&source, &options)
            .err()
            .expect("public parse must preserve the trailing parser error");
        assert_eq!(actual, expected);
    }
}
