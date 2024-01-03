use super::*;

#[test]
fn hello_world() {
    let rules = rules! {
        start {
            (hello_world)
        }
        hello_world {
            ("hello" " " "world")
        }
    };

    let input = "hello world";

    let result = rules.parse("start", input);

    let result = result
        .expect("Could not parse")
        .downcast_ref::<Vec<ParseValue>>()
        .expect("Result should be a Vec")
        .iter()
        .map(|x| x.downcast_ref::<Token>().expect("Should be Token").clone())
        .collect::<Vec<_>>();

    assert_eq!(
        vec![
            Token::from("hello".to_owned()),
            Token::from(" ".to_owned()),
            Token::from("world".to_owned())
        ],
        result
    );
}

#[test]
fn aab() {
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
    let input4 = "c";

    assert_eq!(
        Some(Token::from("b".to_owned())),
        rules
            .parse("start", input0)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![
            Token::from("a".to_owned()),
            Token::from("b".to_owned())
        ]),
        rules
            .parse("start", input1)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![
            Token::from("a".to_owned()),
            Token::from("a".to_owned()),
            Token::from("b".to_owned())
        ]),
        rules
            .parse("start", input2)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(vec![
            Token::from("a".to_owned()),
            Token::from("a".to_owned()),
            Token::from("a".to_owned()),
            Token::from("b".to_owned())
        ]),
        rules
            .parse("start", input3)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    assert_eq!(
        Some(ParseError::UnexpectedChar {
            current_rule: "aab".to_owned(),
            char: Some('c'),
            pos: 0,
            row: 1,
            col: 1,
        }),
        rules.parse("start", input4).err()
    );
}

#[test]
fn calculator() {
    let rules = rules! {
        start {
            (ws term ws) => |v| v[1].clone();
        }

        ws {
            ()
            (ws " ")
        }

        expr {
            (term)
        }

        term {
            (factor)
            (term ws "+" ws term) => |v| {
                match (v[0].downcast_ref::<i32>(), v[4].downcast_ref::<i32>()) {
                    (Some(a), Some(b)) => (a + b).into_value(),
                    _ => unreachable!()
                }
            };
        }

        factor {
            (int)
            ("(" ws expr ws ")") => |v| v[2].clone();
            (factor ws "*" ws factor) => |v| {
                match (v[0].downcast_ref::<i32>(), v[4].downcast_ref::<i32>()) {
                    (Some(a), Some(b)) => (a * b).into_value(),
                    _ => unreachable!()
                }
            };
        }

        digit_nonzero {
            ("1")
            ("2")
            ("3")
            ("4")
            ("5")
            ("6")
            ("7")
            ("8")
            ("9")
        }

        int {
            ("0") => |_| 0.into_value();
            (_int) => |v| match v[0].downcast_ref::<String>() {
                Some(s) => s.parse::<i32>().unwrap().into_value(),
                _ => unreachable!(),
            };
        }

        _int {
            (digit_nonzero) => |v| match v[0].downcast_ref::<Token>() {
                Some(digit) => digit.to_string().into_value(),
                _ => unreachable!()
            };

            (_int digit_nonzero) => |v| match (v[0].downcast_ref::<String>(), v[1].downcast_ref::<Token>()) {
                (Some(int), Some(digit)) => format!("{int}{digit}").into_value(),
                _ => unreachable!()
            };

            (_int "0") => |v| match v[0].downcast_ref::<String>() {
                Some(int) => format!("{int}0").into_value(),
                _ => unreachable!()
            };
        }
    };

    let input = "       12 * 5 + 16 * 2     ";

    let expected_result = 12 * 5 + 16 * 2;

    let result = rules.parse("start", input);

    assert_eq!(
        Some(expected_result),
        result
            .expect("Should be parsed")
            .downcast_ref::<i32>()
            .cloned()
    )
}
