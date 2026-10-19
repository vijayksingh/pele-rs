//! Public graph construction retains exact interning, identity and wire data.
use pele::compat::public::*;

#[test]
fn repeated_property_keys_preserve_string_ids_and_exact_binary_exchange() {
    let keys = [
        String::new(),
        "id".into(),
        "__proto__".into(),
        "a".repeat(64),
        "a".repeat(65),
        "é".repeat(32),
        "é".repeat(33),
        "🦀".repeat(16),
        "🦀".repeat(17),
        "日本語\0key".into(),
    ];
    let build = |properties: bool| {
        let mut writer = GraphWriter::default();
        // Raw UTF-16 remains representable alongside well-formed key lookups.
        let malformed = writer.string(JsString(vec![0xd800, 0x61, 0xdc00]));
        let raw_key = writer.string(JsString(vec![0xdfff]));
        let mut rows = Vec::new();
        for round in 0..3 {
            let mut fields = vec![Property {
                key: raw_key,
                value: Atom::String(malformed),
                flags: 5,
            }];
            for (index, key) in keys.iter().enumerate() {
                let value = match index % 3 {
                    0 => Atom::Number(f64::from_bits(0x7ff8_0000_0000_0042)),
                    1 => Atom::Number(-0.),
                    _ => Atom::Ref(0),
                };
                // The direct string API supplies an independent construction
                // route, including preinterned keys and forward/cyclic refs.
                let property = if properties {
                    writer.property(key, value)
                } else {
                    Property {
                        key: writer.string(key.as_str()),
                        value,
                        flags: 7,
                    }
                };
                assert_eq!(property.key, (index + 2) as u32, "{round}: {key:?}");
                fields.push(property);
            }
            rows.push(writer.node(Node::Object { properties: fields }));
        }
        assert_eq!(rows, [0, 1, 2]);
        writer.finish(Atom::Ref(0)).unwrap()
    };
    let expected = build(false);
    let actual = build(true);
    assert_eq!(actual.strings.len(), keys.len() + 2);
    let bytes = |graph: &ModelGraph| {
        encode_binary_exchange(7, graph, b"keys", DEFAULT_BINARY_BUDGET).unwrap()
    };
    assert_eq!(bytes(&actual), bytes(&expected));
    let decoded = decode_binary_exchange(&bytes(&actual), DEFAULT_BINARY_BUDGET).unwrap();
    assert_eq!(bytes(&decoded.graph), bytes(&expected));
}
