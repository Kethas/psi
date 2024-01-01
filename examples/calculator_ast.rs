use std::io::{stdin, stdout, Write};

use psi_parser::*;

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
                    (ParseValue::Value(a), ParseValue::Value(b)) => ParseValue::Value(ExprAst::Add(Box::new(a.clone()), Box::new(b.clone()))),
                    _ => unreachable!()
                }
            };
            (term ws "-" ws term) => |v| {
                match (&v[0], &v[4]) {
                    (ParseValue::Value(a), ParseValue::Value(b)) => ParseValue::Value(ExprAst::Sub(Box::new(a.clone()), Box::new(b.clone()))),
                    _ => unreachable!()
                }
            };
        }

        factor {
            (float)
            ("(" ws expr ws ")") => |v| v[2].clone();
            (factor ws "*" ws factor) => |v| {
                match (&v[0], &v[4]) {
                    (ParseValue::Value(a), ParseValue::Value(b)) => ParseValue::Value(ExprAst::Mul(Box::new(a.clone()), Box::new(b.clone()))),
                    _ => unreachable!()
                }
            };
            (factor ws "/" ws factor) => |v| {
                match (&v[0], &v[4]) {
                    (ParseValue::Value(a), ParseValue::Value(b)) => ParseValue::Value(ExprAst::Div(Box::new(a.clone()), Box::new(b.clone()))),
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
                ParseValue::Token(s) => ParseValue::String(s.clone()),
                _ => unreachable!()
            };
            (digits digit) => |v| match (&v[0], &v[1]) {
                (ParseValue::String(s0), ParseValue::Token(s1)) => ParseValue::String(format!("{s0}{s1}")),
                _ => unreachable!()
            };
        }

        float {
            (int) => |v| match &v[0] {
                ParseValue::String(s) => ParseValue::Value(ExprAst::Int(s.parse().unwrap())),
                _ => unreachable!()
            };
            (int "." digits) => |v| match (&v[0], &v[2]) {
                (ParseValue::String(s0), ParseValue::String(s1)) => ParseValue::Value(ExprAst::Float(format!("{s0}.{s1}").parse().unwrap())),
                _ => unreachable!()
            };
        }

        int {
            ("0") => |_| ParseValue::String("0".to_owned());
            (_int)
        }

        _int {
            (digit_nonzero) => |v| match &v[0] {
                ParseValue::Token(digit) => ParseValue::String(digit.clone()),
                _ => unreachable!()
            };

            (_int digit_nonzero) => |v| match (&v[0], &v[1]) {
                (ParseValue::String(int), ParseValue::Token(digit)) => ParseValue::String(format!("{int}{digit}")),
                _ => unreachable!()
            };

            (_int "0") => |v| match &v[0] {
                ParseValue::String(int) => ParseValue::String(format!("{int}0")),
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
