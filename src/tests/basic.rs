use super::*;

#[test]
fn hello_world() {
    init();

    let rules = rules! {
        start {
            (hello_world)
        }
        hello_world {
            ("hello" " " "world")
        }
    };

    let input = "hello world";

    let result = rules.parse_proc("start", input);

    let result = result
        .expect("Could not parse")
        .downcast_ref::<Vec<ParseValue>>()
        .expect("Result should be a Vec")
        .iter()
        .map(|x| x.downcast_ref::<Token>().expect("Should be Token").clone())
        .collect::<Vec<_>>();

    assert_eq!(
        vec![Token::from("hello"), Token::from(" "), Token::from("world")],
        result
    );
}

#[test]
fn aab() {
    init();

    let rules = rules! {
        start { (aab) }
        aab {
            ("b")
            ("a" aab) => |v| {
                if let Some(mut list) = v[1].downcast_ref::<Vec<Token>>().cloned() {
                    list.insert(0, v[0].clone().downcast_ref::<Token>().unwrap().clone());

                    list.into_value()
                } else {
                    v
                        .iter()
                        .map(|token| token.downcast_ref::<Token>().unwrap().clone())
                        .collect::<Vec<Token>>()
                        .into_value()
                }
            };
        }
    };

    let input0 = "b";
    let input1 = "ab";
    let input2 = "aab";
    let input3 = "aaab";

    assert_eq!(
        Some(Token::from("b".to_owned())),
        rules
            .parse_proc("start", input0)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![Token::from("a"), Token::from("b")]),
        rules
            .parse_proc("start", input1)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![Token::from("a"), Token::from("a"), Token::from("b")]),
        rules
            .parse_proc("start", input2)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![
            Token::from("a"),
            Token::from("a"),
            Token::from("a"),
            Token::from("b")
        ]),
        rules
            .parse_proc("start", input3)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    let times = 40000;

    let input_huge = (0..times).map(|_| 'a').chain(['b']).collect::<String>();

    log::debug!("input_huge: \"{input_huge}\"");

    assert_eq!(
        Some(
            (0..times)
                .map(|_| Token::from("a"))
                .chain([Token::from("b")])
                .collect::<Vec<Token>>()
        ),
        rules
            .parse_proc("start", &input_huge)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );
}

#[test]
fn abc() {
    #[derive(Clone, Debug, PartialEq)]
    enum Abc {
        Ab,
        Ac(Box<Abc>),
    }

    let rules = rules! {
        start {
            (abc)
        }

        abc {
            ("a" "b") => |_| Abc::Ab.into_value();
            ("a" abc "c") => |v| Abc::Ac(Box::new(v[1].downcast_ref::<Abc>().unwrap().clone())).into_value();
        }
    };

    let input = "aaabcc";
    let expected_result = Abc::Ac(Box::new(Abc::Ac(Box::new(Abc::Ab))));

    assert_eq!(
        &expected_result,
        rules
            .parse_proc("start", input)
            .expect("Should be parsed")
            .downcast_ref()
            .unwrap()
    )
}

#[test]
fn char_literal() {
    init();

    let rules = rules! {
        start { (char) }

        char /* char */ {
            ("'" char_inner "'") => |v| v[1].clone();
        }

        char_inner /* char */ {
            (char_escape)
            ((! "'")) => |v| {
                v[0].downcast_ref::<Token>().unwrap().chars().next().unwrap().into_value()
            };
        }

        char_escape /* char */ {
            ("\\" "'") => |_| '\''.into_value();
            ("\\" "n") => |_| '\n'.into_value();
            ("\\" "r") => |_| '\r'.into_value();
            ("\\" "t") => |_| '\t'.into_value();
            ("\\" "\\") => |_| '\\'.into_value();
            ("\\" "0") => |_| '\0'.into_value();
        }

    };

    let inputs = [
        ("'a'", 'a'),
        ("' '", ' '),
        ("'\\''", '\''),
        ("'\\n'", '\n'),
        ("'\\r'", '\r'),
        ("'\\t'", '\t'),
        ("'\\\\'", '\\'),
        ("'\\0'", '\0'),
    ];

    for (input, expected_result) in inputs {
        assert_eq!(
            expected_result,
            *rules
                .parse_proc("start", input)
                .expect("Should be parsed.")
                .downcast_ref()
                .unwrap()
        );
    }
}

#[test]
fn import() {
    init();

    let boolean_rules = rules! {
        boolean {
            ("true") => |_| true.into_value();
            ("false") => |_| false.into_value();
        }
    };

    let rules = rules! {
        #[import (boolean_rules) as boolean]

        start {
            ((boolean::boolean))
        }
    };

    let inputs = [("true", true), ("false", false)];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules
                .parse_proc("start", input)
                .expect("Should be parsed")
                .downcast_ref()
        )
    }
}

#[test]
fn import2() {
    init();

    struct NamesRules;

    impl From<NamesRules> for Rules {
        fn from(_: NamesRules) -> Self {
            rules! {
                name {
                    ("John") => |_| "John".to_owned().into_value();
                    ("James") => |_| "James".to_owned().into_value();
                    ("Joey") => |_| "Joey".to_owned().into_value();
                }
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum Greeting {
        Hello(String),
        Hi(String),
        Greetings(String),
    }

    let greeting_rules = rules! {
        #[import (NamesRules) as names]

        greeting {
            ("Hello " (names::name) "!") => |v| Greeting::Hello(v[1].downcast_ref::<String>().unwrap().clone()).into_value();
            ("Hi " (names::name) "!") => |v| Greeting::Hi(v[1].downcast_ref::<String>().unwrap().clone()).into_value();
            ("Greetings " (names::name) "!") => |v| Greeting::Greetings(v[1].downcast_ref::<String>().unwrap().clone()).into_value();
        }
    };

    let rules = rules! {
        #[import (greeting_rules) as greetings]

        start {
            ((greetings::greeting))
        }
    };

    let inputs = [
        ("Hello John!", Greeting::Hello("John".to_owned())),
        ("Hello Joey!", Greeting::Hello("Joey".to_owned())),
        ("Hi John!", Greeting::Hi("John".to_owned())),
        ("Greetings James!", Greeting::Greetings("James".to_owned())),
        ("Greetings John!", Greeting::Greetings("John".to_owned())),
    ];

    log::debug!("Rules: {:?}", rules.rule_names());

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules
                .parse_proc("start", input)
                .expect("Should be parsed")
                .downcast_ref()
        );
    }
}
