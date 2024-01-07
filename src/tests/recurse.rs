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
                v(0).downcast::<String>().unwrap(),
                v(2).downcast::<Token>().unwrap()
            ).into_value();
        }
    };

    let input0 = "x";

    assert_eq!(
        Some(&String::from("x")),
        rules
            .parse_entire("start", input0)
            .expect("Should be parsed")
            .downcast_ref(),
    );

    let input1 = "x+x";

    assert_eq!(
        Some(&String::from("x+x")),
        rules
            .parse_entire("start", input1)
            .expect("Should be parsed")
            .downcast_ref(),
    );

    let input2 = "x+x+x";

    assert_eq!(
        Some(&String::from("x+x+x")),
        rules
            .parse_entire("start", input2)
            .expect("Should be parsed")
            .downcast_ref(),
    );
}

#[test]
fn ternary() {
    #[derive(PartialEq, Debug)]
    enum TernaryDigit {
        Zero,
        One,
        Two,
    }

    let rules = rules! {
        start {
            (ternary)
        }

        ternary {
            ("0") => |_| vec![TernaryDigit::Zero].into_value();
            (ternary_inner)
        }

        ternary_inner /* Vec<TernaryDigit> */ {
            (ternary_inner digit) => |v| {
                let mut vec = v(0).downcast::<Vec<TernaryDigit>>().unwrap();
                vec.push(*v(1).downcast::<TernaryDigit>().unwrap());

                vec
            };
            (digit_nonzero) => |v| vec![*v(0).downcast::<TernaryDigit>().unwrap()].into_value();
        }


        digit_nonzero /* TernaryDigit */ {
            ("1") => |_| TernaryDigit::One.into_value();
            ("2") => |_| TernaryDigit::Two.into_value();
        }

        digit /* TernaryDigit */ {
            ("0") => |_| TernaryDigit::Zero.into_value();
            ("1") => |_| TernaryDigit::One.into_value();
            ("2") => |_| TernaryDigit::Two.into_value();
        }
    };

    let input = "120";
    let expected_result = vec![TernaryDigit::One, TernaryDigit::Two, TernaryDigit::Zero];

    assert_eq!(
        Some(&expected_result),
        rules
            .parse_entire("start", input)
            .expect("Should be parsed")
            .downcast_ref::<Vec<TernaryDigit>>()
    )
}

#[test]
fn calculator() {
    init();

    let rules = rules! {
        start {
            (ws term ws) => |v| v(1);
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
            (term ws "+" ws term)
             => |v| (*v(0).downcast::<i32>().unwrap() + *v(4).downcast::<i32>().unwrap()).into_value();
        }

        factor {
            (int)
            ("(" ws expr ws ")") => |v| v(2);
            (factor ws "*" ws factor)
                => |v| (*v(0).downcast::<i32>().unwrap() * *v(4).downcast::<i32>().unwrap()).into_value();
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
            (_int)
                => |v| v(0).downcast::<String>().unwrap().parse::<i32>().unwrap().into_value();
        }

        _int {
            (digit_nonzero) => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();

            (_int digit_nonzero)
                => |v| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();

            (_int "0")
            => |v| format!(
                "{}0",
                v(0).downcast::<String>().unwrap(),
            ).into_value();
        }
    };

    let input = "       12 * 5 + 16 * 2     ";

    let expected_result = 12 * 5 + 16 * 2;

    let result = rules.parse_entire("start", input);

    assert_eq!(
        Some(expected_result),
        result
            .expect("Should be parsed")
            .downcast_ref::<i32>()
            .cloned()
    )
}
