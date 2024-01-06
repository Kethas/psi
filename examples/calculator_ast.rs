use std::io::{stdin, stdout, Write};

use psi_parser::prelude::*;

#[derive(Debug, Clone)]
enum ExprAst {
    Int(i32),
    Float(f32),
    Add(Box<ExprAst>, Box<ExprAst>),
    Sub(Box<ExprAst>, Box<ExprAst>),
    Mul(Box<ExprAst>, Box<ExprAst>),
    Div(Box<ExprAst>, Box<ExprAst>),
}

fn main() {
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
            (term ws "+" ws factor) => |v| {
                ExprAst::Add(v(0).downcast().unwrap(), v(4).downcast().unwrap()).into_value()
            };
            (term ws "-" ws factor) => |v| {
                ExprAst::Sub(v(0).downcast().unwrap(), v(4).downcast().unwrap()).into_value()
            };
        }

        factor {
            (float)
            ("(" ws expr ws ")") => |v| v(2);
            (factor ws "*" ws float) => |v| {
                ExprAst::Mul(v(0).downcast().unwrap(), v(4).downcast().unwrap()).into_value()
            };
            (factor ws "/" ws float) => |v| {
                ExprAst::Div(v(0).downcast().unwrap(), v(4).downcast().unwrap()).into_value()
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
            (digit) => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();
            (digits digit)
                => |v| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();
        }

        float {
            (int) => |v| ExprAst::Int(v(0).downcast::<String>().unwrap().parse().unwrap()).into_value();
            (int "." digits) => |v| {
                let str = format!("{}.{}", v(0).downcast::<String>().unwrap(), v(2).downcast::<String>().unwrap());

                ExprAst::Float(str.parse().unwrap()).into_value()
            };
        }

        int {
            ("0") => |_| "0".to_owned().into_value();
            (_int)
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

    println!("Enter a simple arithmetic to be converted into AST or 'exit' to exit.");

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
                if let Some(ast) = value.downcast_ref::<ExprAst>() {
                    println!("AST: {ast:#?}");
                } else {
                    println!("{value:?}");
                }
            }
            Err(error) => println!("Error: {error:#?}"),
        }
    }
}
