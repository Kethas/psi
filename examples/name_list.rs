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
            () => |_| Vec::<Name>::new().into_value();
            (list_inner)
        }

        list_inner {
            (name) => |v| match v[0].downcast_ref::<Name>() {
                Some(name) => vec![name.clone()].into_value(),
                _ => unreachable!()
            };
            (list_inner "," name) => |v| match (v[0].downcast_ref::<Vec<Name>>(), v[2].downcast_ref::<Name>()) {
                (Some(list), Some(name)) => {
                    let mut vec = list.clone();
                    vec.push(name.clone());
                    vec.into_value()
                },

                _ => unreachable!()
            };
        }

        name {
            ("John") => |_| Name::John.into_value();
            ("Jane") => |_| Name::Jane.into_value();
            ("Jeremiah") => |_| Name::Jeremiah.into_value();
            ("Josh") => |_| Name::Josh.into_value();
            ("Jimmy") => |_| Name::Jimmy.into_value();
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
