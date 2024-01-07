use super::*;

#[test]
fn integer() {
    init();

    let inputs = [
        ("0", 0),
        ("12850", 12850),
        ("-12850", -12850),
        ("+11100000", 11100000),
        ("-9", -9),
        ("-0", -0),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules::Integer
                .parse_entire("integer", input)
                .expect("Should be parsed")
                .downcast_ref::<isize>()
        )
    }
}

#[test]
fn hex() {
    init();

    let inputs = [
        ("0x12fab9", 0x12fab9),
        ("0x0002", 0x0002),
        (
            "0xFFFf",
            #[allow(clippy::mixed_case_hex_literals)]
            0xFFFf,
        ),
        ("0x10000a", 0x10000a),
        ("0x00000000", 0x00000000),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules::Hex
                .parse_entire("hex", input)
                .expect("Should be parsed")
                .downcast_ref::<usize>()
        )
    }
}

#[test]
fn alpha() {
    init();

    let inputs = [("a", "a"), ("b", "b"), ("Z", "Z"), ("X", "X")];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&Token::from(expected_result)),
            rules::Alpha
                .parse_entire("alpha", input)
                .expect("Should be parsed")
                .downcast_ref()
        )
    }
}

#[test]
fn identifier() {
    init();

    let inputs = [
        ("john1123", "john1123"),
        ("james", "james"),
        ("__HELLO_WORLD__13", "__HELLO_WORLD__13"),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result.to_owned()),
            rules::Identifier
                .parse_entire("identifier", input)
                .expect("Should be parsed")
                .downcast_ref::<String>()
        )
    }
}

#[test]
fn string() {
    init();

    let inputs = [
        ("\"abcd\"", "abcd"),
        ("\"ab\\\"cd\"", "ab\"cd"),
        ("\"\\n\\r\\t\\0\"", "\n\r\t\0"),
        ("\"\\u{1F3A9}\"", "🎩"),
        ("\"\\u{1F3A9}\"", "\u{1F3A9}"),
        ("\"\\x7F\"", "\x7F"),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result.to_owned()),
            rules::StringRules
                .parse_entire("string", input)
                .expect("Should be parsed")
                .downcast_ref::<String>()
        )
    }
}

#[test]
fn boolean() {
    init();

    let inputs = [("true", true), ("false", false)];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules::Boolean
                .parse_entire("boolean", input)
                .expect("Should be parsed")
                .downcast_ref::<bool>()
        )
    }
}
