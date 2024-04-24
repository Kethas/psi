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

    let result = rules.parse_entire("start", input);

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
            ("a" aab) => |v, _| {
                let v0 = *v(0).downcast::<Token>().unwrap();
                let v1 = v(1);

                match v1.downcast::<Vec<Token>>() {
                    // If v1 is a list, then add v0 to the start of it
                    Ok(mut v1) => {
                        v1.insert(0, v0);

                        v1
                    }

                    // otherwise, v1 has to be a token
                    Err(v1) => {
                        let v1 = *v1.downcast::<Token>().unwrap();

                        vec![v0, v1].into_value()
                    }
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
            .parse_entire("start", input0)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![Token::from("a"), Token::from("b")]),
        rules
            .parse_entire("start", input1)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![Token::from("a"), Token::from("a"), Token::from("b")]),
        rules
            .parse_entire("start", input2)
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
            .parse_entire("start", input3)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    let times = 1000;

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
            .parse_entire("start", &input_huge)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );
}

#[test]
fn abc() {
    init();

    #[derive(Debug, PartialEq)]
    enum Abc {
        Ab,
        Ac(Box<Abc>),
    }

    let rules = rules! {
        start {
            (abc)
        }

        abc {
            ("a" "b") => |_, _| Abc::Ab.into_value();
            ("a" abc "c") => |v, _| Abc::Ac(v(1).downcast::<Abc>().unwrap()).into_value();
        }
    };

    let input = "aaabcc";
    let expected_result = Abc::Ac(Box::new(Abc::Ac(Box::new(Abc::Ab))));

    assert_eq!(
        &expected_result,
        rules
            .parse_entire("start", input)
            .expect("Should be parsed")
            .downcast_ref()
            .unwrap()
    )
}

#[test]
fn xab() {
    init();

    let rules = rules! {
        start {
            (xab)
        }

        xab {
            ("x" "a") => |_, _| "xa".to_owned().into_value();
            ((!"a") "b") => |v, _| format!("{}b", v(0).downcast::<Token>().unwrap()).into_value();
        }
    };

    let inputs = [("xa", "xa".to_owned()), ("xb", "xb".to_owned())];

    for (input, expected_result) in inputs {
        log::debug!("input = \"{input}\"");

        assert_eq!(
            Some(&expected_result),
            rules
                .parse_entire("start", input)
                .expect("Should be parsed")
                .downcast_ref()
        )
    }
}

#[test]
fn char_literal() {
    init();

    let rules = rules! {
        start { (char) }

        char /* char */ {
            ("'" char_inner "'") => |v, _| v(1);
        }

        char_inner /* char */ {
            (char_escape)
            ((! "'")) => |v, _| {
                // Take first character
                v(0).downcast::<Token>().unwrap().chars().next().unwrap().into_value()
            };
        }

        char_escape /* char */ {
            ("\\" "'") => |_, _| '\''.into_value();
            ("\\" "n") => |_, _| '\n'.into_value();
            ("\\" "r") => |_, _| '\r'.into_value();
            ("\\" "t") => |_, _| '\t'.into_value();
            ("\\" "\\") => |_, _| '\\'.into_value();
            ("\\" "0") => |_, _| '\0'.into_value();
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
                .parse_entire("start", input)
                .expect("Should be parsed.")
                .downcast_ref()
                .unwrap()
        );
    }
}

#[test]
fn advanced_not() {
    init();

    #[derive(Eq, PartialEq, Debug)]
    enum Quotes {
        OnePair(String),
        TwoPairs(String),
        ThreePairs(String),
    }

    let rules = rules! {
        start {
            ("'" _one_pair "'") => |v, _| {
                let inner = *v(1).downcast::<String>().unwrap();

                Quotes::OnePair(inner).into_value()
            };
            ("''" _two_pairs "''") => |v, _| {
                let inner = *v(1).downcast::<String>().unwrap();

                Quotes::TwoPairs(inner).into_value()
            };
            ("'''" _three_pairs "'''") => |v, _| {
                let inner = *v(1).downcast::<String>().unwrap();

                Quotes::ThreePairs(inner).into_value()
            };
        }

        _one_pair {
            () => |_, _| String::new().into_value();
            (_one_pair (! "'")) => |v, _| {
                let str = v(0).downcast::<String>().unwrap();
                let char = v(1).downcast::<Token>().unwrap();

                format!("{str}{char}").into_value()
            };
        }

        _two_pairs {
            () => |_, _| String::new().into_value();
            (_two_pairs (! "''")) => |v, _| {
                let str = v(0).downcast::<String>().unwrap();
                let char = v(1).downcast::<Token>().unwrap();

                format!("{str}{char}").into_value()
            };
        }

        _three_pairs {
            () => |_, _| String::new().into_value();
            (_three_pairs (! "'''")) => |v, _| {
                let str = v(0).downcast::<String>().unwrap();
                let char = v(1).downcast::<Token>().unwrap();

                format!("{str}{char}").into_value()
            };
        }
    };

    let inputs = [
        ("''", Quotes::OnePair("".to_owned())),
        ("' '", Quotes::OnePair(" ".to_owned())),
        ("''''", Quotes::TwoPairs("".to_owned())),
        ("'' ' ''", Quotes::TwoPairs(" ' ".to_owned())),
        ("''''''", Quotes::ThreePairs("".to_owned())),
        ("''' ' '''", Quotes::ThreePairs(" ' ".to_owned())),
        ("''' '' '''", Quotes::ThreePairs(" '' ".to_owned())),
    ];

    for (input, expected_result) in inputs {
        println!("input: {input}");
        assert_eq!(
            expected_result,
            *rules
                .parse_entire("start", input)
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
            ("true") => |_, _| true.into_value();
            ("false") => |_, _| false.into_value();
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
                .parse_entire("start", input)
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
                    ("John") => |_, _| "John".to_owned().into_value();
                    ("James") => |_, _| "James".to_owned().into_value();
                    ("Joey") => |_, _| "Joey".to_owned().into_value();
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
            ("Hello " (names::name) "!") => |v, _| Greeting::Hello(*v(1).downcast::<String>().unwrap()).into_value();
            ("Hi " (names::name) "!") => |v, _| Greeting::Hi(*v(1).downcast::<String>().unwrap()).into_value();
            ("Greetings " (names::name) "!") => |v, _| Greeting::Greetings(*v(1).downcast::<String>().unwrap()).into_value();
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
                .parse_entire("start", input)
                .expect("Should be parsed")
                .downcast_ref()
        );
    }
}
