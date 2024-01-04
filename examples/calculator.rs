use std::io::{stdin, stdout, Write};

use psi_parser::prelude::*;

fn main() {
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
            (term ws "+" ws factor) => |v| {
                match (v[0].downcast_ref::<f32>(), v[4].downcast_ref::<f32>()) {
                    (Some(a), Some(b)) => (a + b).into_value(),
                    _ => unreachable!()
                }
            };
            (term ws "-" ws factor) => |v| {
                match (v[0].downcast_ref::<f32>(), v[4].downcast_ref::<f32>()) {
                    (Some(a), Some(b)) => (a - b).into_value(),
                    _ => unreachable!()
                }
            };
        }

        factor {
            (float)
            ("(" ws expr ws ")") => |v| v[2].clone();
            (factor ws "*" ws float) => |v| {
                match (v[0].downcast_ref::<f32>(), v[4].downcast_ref::<f32>()) {
                    (Some(a), Some(b)) => (a * b).into_value(),
                    _ => unreachable!()
                }
            };
            (factor ws "/" ws float) => |v| {
                match (v[0].downcast_ref::<f32>(), v[4].downcast_ref::<f32>()) {
                    (Some(a), Some(b)) => (a / b).into_value(),
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

        digit {
            ("0")
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

        digits {
            (digit) => |v| match v[0].downcast_ref::<Token>() {
                Some(s) => s.to_string().into_value(),
                _ => unreachable!()
            };
            (digits digit) => |v| match (v[0].downcast_ref::<String>(), v[1].downcast_ref::<Token>()) {
                (Some(s0), Some(s1)) => format!("{s0}{s1}").into_value(),
                _ => unreachable!()
            };
        }

        float {
            (int) => |v| match v[0].downcast_ref::<String>() {
                Some(s) => s.parse::<f32>().unwrap().into_value(),
                _ => unreachable!()
            };
            (int "." digits) => |v| match (v[0].downcast_ref::<String>(), v[2].downcast_ref::<String>()) {
                (Some(s0), Some(s1)) => format!("{s0}.{s1}").parse::<f32>().unwrap().into_value(),
                _ => unreachable!()
            };
        }

        int {
            ("0") => |_| "0".to_owned().into_value();
            (_int)
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

    println!("Enter simple arithmetic to be calculated or 'exit' to exit.");

    let stdin = stdin();
    let mut stdout = stdout();

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut line = String::new();

        stdin.read_line(&mut line).unwrap();

        let line = line.trim_end();

        if line.trim().eq_ignore_ascii_case("exit") || line.trim().eq_ignore_ascii_case("quit") {
            break;
        }

        let parse_result = rules.parse_entire("start", line);

        match parse_result {
            Ok(value) => {
                if let Some(value) = value.downcast_ref::<f32>() {
                    println!("= {value}")
                } else {
                    println!("{value:?}");
                }
            }
            Err(error) => println!("Error {error:#?}"),
        }
    }
}
