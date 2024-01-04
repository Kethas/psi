use super::*;
use std::io::Write;

fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .try_init();
}

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
        Some(vec![
            Token::from("a".to_owned()),
            Token::from("b".to_owned())
        ]),
        rules
            .parse_proc("start", input1)
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
            .parse_proc("start", input2)
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
            .parse_proc("start", input3)
            .expect("Should be parsed")
            .downcast_ref()
            .cloned()
    );

    let times = 1000;

    let input_huge = (0..times).map(|_| 'a').chain(['b']).collect::<String>();

    println!("input_huge: \"{input_huge}\"");

    assert_eq!(
        Some(
            (0..times)
                .map(|_| Token::from("a".to_owned()))
                .chain([Token::from("b".to_owned())])
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
fn calculator() {
    init();

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

    let result = rules.parse_proc("start", input);

    assert_eq!(
        Some(expected_result),
        result
            .expect("Should be parsed")
            .downcast_ref::<i32>()
            .cloned()
    )
}

#[test]
fn left_recursion() {
    init();

    let rules = rules! {
        start {
            (expr)
        }

        expr {
            ("x") => |_| "x".to_owned().into_value();
            (expr "+" "x")
            => |v| format!(
                "{}+{}",
                v[0].downcast_ref::<String>().unwrap(),
                v[2].downcast_ref::<Token>().unwrap()
            ).into_value();
        }
    };

    let input0 = "x";

    assert_eq!(
        Some(&String::from("x")),
        rules
            .parse_proc("start", input0)
            .expect("Should be parsed")
            .downcast_ref(),
    );

    let input1 = "x+x";

    assert_eq!(
        Some(&String::from("x+x")),
        rules
            .parse_proc("start", input1)
            .expect("Should be parsed")
            .downcast_ref(),
    );

    let input2 = "x+x+x";

    assert_eq!(
        Some(&String::from("x+x+x")),
        rules
            .parse_proc("start", input2)
            .expect("Should be parsed")
            .downcast_ref(),
    );
}
