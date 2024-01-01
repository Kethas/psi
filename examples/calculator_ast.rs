use std::io::{stdin, stdout, Write};

use psi_parser::prelude::*;

#[derive(Debug, Clone, PartialEq)]
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
        #[type = ExprAst]

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
                match (&v[0], &v[4]) {
                    (ParseValue::Value(a), ParseValue::Value(b)) => ExprAst::Add(Box::new(a.clone()), Box::new(b.clone())).into_value(),
                    _ => unreachable!()
                }
            };
            (term ws "-" ws term) => |v| {
                match (&v[0], &v[4]) {
                    (ParseValue::Value(a), ParseValue::Value(b)) => ExprAst::Sub(Box::new(a.clone()), Box::new(b.clone())).into_value(),
                    _ => unreachable!()
                }
            };
        }

        factor {
            (float)
            ("(" ws expr ws ")") => |v| v[2].clone();
            (factor ws "*" ws factor) => |v| {
                match (&v[0], &v[4]) {
                    (ParseValue::Value(a), ParseValue::Value(b)) => ExprAst::Mul(Box::new(a.clone()), Box::new(b.clone())).into_value(),
                    _ => unreachable!()
                }
            };
            (factor ws "/" ws factor) => |v| {
                match (&v[0], &v[4]) {
                    (ParseValue::Value(a), ParseValue::Value(b)) => ExprAst::Div(Box::new(a.clone()), Box::new(b.clone())).into_value(),
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
            (digit) => |v| match &v[0] {
                ParseValue::Token(s) => s.clone().into(),
                _ => unreachable!()
            };
            (digits digit) => |v| match (&v[0], &v[1]) {
                (ParseValue::String(s0), ParseValue::Token(s1)) => format!("{s0}{s1}").into(),
                _ => unreachable!()
            };
        }

        float {
            (int) => |v| match &v[0] {
                ParseValue::String(s) => ExprAst::Int(s.parse().unwrap()).into_value(),
                _ => unreachable!()
            };
            (int "." digits) => |v| match (&v[0], &v[2]) {
                (ParseValue::String(s0), ParseValue::String(s1)) => ExprAst::Float(format!("{s0}.{s1}").parse().unwrap()).into_value(),
                _ => unreachable!()
            };
        }

        int {
            ("0") => |_| "0".to_owned().into();
            (_int)
        }

        _int {
            (digit_nonzero) => |v| match &v[0] {
                ParseValue::Token(digit) => digit.clone().into(),
                _ => unreachable!()
            };

            (_int digit_nonzero) => |v| match (&v[0], &v[1]) {
                (ParseValue::String(int), ParseValue::Token(digit)) => format!("{int}{digit}").into(),
                _ => unreachable!()
            };

            (_int "0") => |v| match &v[0] {
                ParseValue::String(int) => format!("{int}0").into(),
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
            Ok(ParseValue::Value(value)) => println!("AST: {value:#?}"),
            Err(error) => println!("Error: {error:#?}"),
            Ok(value) => println!("{value:#?}"),
        }
    }
}
