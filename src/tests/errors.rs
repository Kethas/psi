use super::*;

#[test]
fn transformer_error() {
    init();

    const ALLOWED_NAMES: &[&str] = &["John", "Jack"];

    #[derive(Debug)]
    struct NameError {
        name: String,
    }

    impl std::error::Error for NameError {}

    impl std::fmt::Display for NameError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let Self { name } = self;

            f.write_fmt(format_args!(
                "Name '{name}' is not allowed. Allowed names: {ALLOWED_NAMES:?}"
            ))
        }
    }

    let rules = rules! {
        #[import (rules::Identifier) as id]

        start {
            ((id::identifier)) => |v, _| {
                let id = v(0).downcast::<String>().unwrap();

                if !ALLOWED_NAMES.contains(&id.as_str()) {
                    NameError {
                        name: *id
                    }.into_error()
                } else {
                    id
                }
            };
        }
    };

    // The errors have to be compared as strings since Error doesn't implement PartialEq

    let inputs = [
        ("John", "John".to_owned()),
        ("Jack", "Jack".to_owned()),
        (
            "Garfield",
            ParseError::TransformerError {
                current_rule: "start".to_owned(),
                pos: 8,
                row: 1,
                col: 9,
                error: Box::new(NameError {
                    name: "Garfield".to_owned(),
                }),
            }
            .to_string(),
        ),
    ];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        let result = match rules.parse_entire("start", input) {
            Ok(parse_value) => *parse_value.downcast::<String>().unwrap(),
            Err(err) => err.to_string(),
        };

        assert_eq!(expected_result, result);
    }
}
