use super::*;

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
