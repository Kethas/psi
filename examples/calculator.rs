use std::io::{stdin, stdout, Write};

use psi_parser::prelude::*;

fn main() {
    let rules = rules! {
        start {
            (ws term ws) => |v, _| v(1);
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
            (term ws "+" ws factor) => |v, _| {
                (*v(0).downcast::<f32>().unwrap() + *v(4).downcast::<f32>().unwrap()).into_value()
            };
            (term ws "-" ws factor) => |v, _| {
                (*v(0).downcast::<f32>().unwrap() - *v(4).downcast::<f32>().unwrap()).into_value()
            };
        }

        factor {
            (float)
            ("(" ws expr ws ")") => |v, _| v(2);
            (factor ws "*" ws float) => |v, _| {
                (*v(0).downcast::<f32>().unwrap() * *v(4).downcast::<f32>().unwrap()).into_value()
            };
            (factor ws "/" ws float) => |v, _| {
                (*v(0).downcast::<f32>().unwrap() / *v(4).downcast::<f32>().unwrap()).into_value()
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
            (digit) => |v, _| v(0).downcast::<Token>().unwrap().to_string().into_value();
            (digits digit)
                => |v, _| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();
        }

        float {
            (int) => |v, _| v(0).downcast::<String>().unwrap().parse::<f32>().unwrap().into_value();
            (int "." digits) => |v, _| {
                let str = format!("{}.{}", v(0).downcast::<String>().unwrap(), v(2).downcast::<String>().unwrap());

                str.parse::<f32>().unwrap().into_value()
            };
        }

        int {
            ("0") => |_, _| "0".to_owned().into_value();
            (_int)
        }

        _int {
            (digit_nonzero) => |v, _| v(0).downcast::<Token>().unwrap().to_string().into_value();

            (_int digit_nonzero)
                => |v, _| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();

            (_int "0")
                => |v, _| format!(
                    "{}0",
                    v(0).downcast::<String>().unwrap(),
                ).into_value();
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
