use std::collections::HashMap;

use super::*;

#[test]
fn whitespace() {
    let inputs = ["", "     ", "  \t", "\t\t   ", "\t", "\t \t"];

    let inputs_ml = [
        "",
        "\n",
        "\r\n",
        "\n\r",
        "\t \n\n\n  \t  \r   ",
        "\n\n\r\r\n\n\t                                    ",
    ];

    for input in inputs {
        log::debug!("input = \"{input}\"");

        assert!(rules::Whitespace.parse_entire("ws", input).is_ok());
    }

    for input in inputs_ml {
        log::debug!("input = \"{input}\"");

        assert!(rules::Whitespace.parse_entire("ws_ml", input).is_ok());
    }
}

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
        ("\"\"", ""),
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

#[test]
fn float() {
    init();

    let inputs = [
        ("0", 0.0),
        ("200.300", 200.300),
        ("0.000001", 0.000001),
        ("00001.30000002", 00001.30000002),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules::Float
                .parse_entire("float", input)
                .expect("Should be parsed")
                .downcast_ref::<f64>()
        )
    }
}

#[test]
fn json() {
    use rules::json::Json;

    init();

    let inputs = [(
        r#"
        {
            "key": "value",
            "0": 12323.222976,
            "something\n": null,
            "is_something": false,
            "array": [
                "one",
                "two",
                {
                    "name": "untitled"
                },
                true
            ]
        }
        "#,
        Json::from(
            [
                ("key".to_owned(), Json::from("value".to_owned())),
                ("0".to_owned(), Json::from(12323.222976)),
                ("something\n".to_owned(), Json::Null),
                ("is_something".to_owned(), Json::from(false)),
                (
                    "array".to_owned(),
                    Json::from(vec![
                        Json::from("one".to_owned()),
                        Json::from("two".to_owned()),
                        Json::from(
                            [("name".to_owned(), Json::from("untitled".to_owned()))]
                                .into_iter()
                                .collect::<HashMap<String, Json>>(),
                        ),
                        Json::from(true),
                    ]),
                ),
            ]
            .into_iter()
            .collect::<HashMap<String, Json>>(),
        ),
    )];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules::JsonRules
                .parse_entire("start", input)
                .expect("Should be parsed")
                .downcast_ref::<Json>()
        )
    }
}

#[test]
fn xml() {
    use rules::simple_xml::Xml;
    init();

    let inputs = [
        (
            "<node attribute=id attribute2='string&amp;'>text <self_closing/> </node>",
            Ok(Xml::Node(
                "node".to_owned(),
                {
                    let mut map = HashMap::new();
                    map.insert("attribute".to_owned(), "id".to_string());
                    map.insert("attribute2".to_owned(), "string&".to_string());
                    map
                },
                vec![
                    Xml::Text("text ".to_owned()),
                    Xml::Node("self_closing".to_owned(), HashMap::new(), Vec::new()),
                    Xml::Text(" ".to_owned()),
                ],
            )),
        ),
        (
            "<xml/>",
            Err(
                /*ParseError::TransformerError {
                    current_rule: "node".to_owned(),
                    pos: 6,
                    row: 1,
                    col: 7,
                    error: Box::new(XmlParseError::IllegalTagName {
                        tag: "xml".to_owned(),
                    }),
                }
                .to_string()*/
                (),
            ),
        ),
        (
            "<a><b/></c>",
            Err(
                /*ParseError::TransformerError {
                    current_rule: "node".to_owned(),
                    pos: 11,
                    row: 1,
                    col: 12,
                    error: Box::new(XmlParseError::NonMatchingTagNames {
                        start_tag: "a".to_owned(),
                        end_tag: "c".to_owned(),
                    }),
                }
                .to_string()*/
                (),
            ),
        ),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            expected_result,
            rules::XmlRules
                .parse_entire("start", input)
                .map(|res| *res.downcast::<Xml>().unwrap())
                .map_err(|_| ())
        )
    }
}
