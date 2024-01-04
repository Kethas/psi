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
                match (v[0].downcast_ref::<ExprAst>(), v[4].downcast_ref::<ExprAst>()) {
                    (Some(a), Some(b)) => ExprAst::Add(Box::new(a.clone()),Box::new(b.clone())).into_value(),
                    _ => unreachable!()
                }
            };
            (term ws "-" ws factor) => |v| {
                match (v[0].downcast_ref::<ExprAst>(), v[4].downcast_ref::<ExprAst>()) {
                    (Some(a), Some(b)) => ExprAst::Sub(Box::new(a.clone()),Box::new(b.clone())).into_value(),
                    _ => unreachable!()
                }
            };
        }

        factor {
            (float)
            ("(" ws expr ws ")") => |v| v[2].clone();
            (factor ws "*" ws float) => |v| {
                match (v[0].downcast_ref::<ExprAst>(), v[4].downcast_ref::<ExprAst>()) {
                    (Some(a), Some(b)) => ExprAst::Mul(Box::new(a.clone()),Box::new(b.clone())).into_value(),
                    _ => unreachable!()
                }
            };
            (factor ws "/" ws float) => |v| {
                match (v[0].downcast_ref::<ExprAst>(), v[4].downcast_ref::<ExprAst>()) {
                    (Some(a), Some(b)) => ExprAst::Div(Box::new(a.clone()),Box::new(b.clone())).into_value(),
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
                Some(s) => ExprAst::Int(s.parse().unwrap()).into_value(),
                _ => unreachable!()
            };
            (int "." digits) => |v| match (v[0].downcast_ref::<String>(), v[2].downcast_ref::<String>()) {
                (Some(s0), Some(s1)) => ExprAst::Float(format!("{s0}.{s1}").parse().unwrap()).into_value(),
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
