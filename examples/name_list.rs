use std::io::{stdin, stdout, Write};

use psi_parser::prelude::*;

#[derive(Debug, Clone)]
enum Name {
    John,
    Jane,
    Jeremiah,
    Josh,
    Jimmy,
}

fn main() {
    let rules = rules! {
        start {
            (list)
        }

        list {
            () => |_, _| Vec::<Name>::new().into_value();
            (list_inner)
        }

        list_inner {
            (name) => |v, _| vec![*v(0).downcast::<Name>().unwrap()].into_value();
            (list_inner "," name) => |v, _| {
                let mut v0 = v(0).downcast::<Vec<Name>>().unwrap();
                v0.push(*v(2).downcast().unwrap());
                v0
            };
        }

        name {
            ("John") => |_, _| Name::John.into_value();
            ("Jane") => |_, _| Name::Jane.into_value();
            ("Jeremiah") => |_, _| Name::Jeremiah.into_value();
            ("Josh") => |_, _| Name::Josh.into_value();
            ("Jimmy") => |_, _| Name::Jimmy.into_value();
        }
    };

    println!("Enter a list of names (separated by commas) or 'exit' to exit.");
    println!("Allowed names: John, Jane, Jeremiah, Josh, Jimmy.");

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
                if let Some(value) = value.downcast_ref::<Vec<Name>>() {
                    println!("Names: {value:#?}")
                } else {
                    println!("{value:?}");
                }
            }
            Err(error) => println!("Error {error:#?}"),
        }
    }
}
