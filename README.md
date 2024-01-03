# Psi
>
> **Warning**
>
> This project is under development. It has many known and unknown bugs and missing features.

Psi is a parser aiming to trade performance for ease of use.

## Grammar

A grammar is a collection of rules.
Each rule is comprised of one or more definitions.
Each definition is a list of rule parts.
A rule part can either be a terminal (AKA token or literal) or a non-terminal (AKA production or rule).

These constructs can be used to represent (hopefully) almost anything that can be parsed.

## Using this crate

The easiest way to make a Psi grammar is using the `rules!` macro.

```rust
use psi_parser::prelude::*;

let rules = rules! {
    start {
        (hello)
    }

    hello {
        ("Hello, " name "!")
    }

    name {
        ("John")
        ("Jane")
        ("Jeremiah")
        ("Josh")
        ("Jimmy")
    }
};

// The arguments to parse_entire are which rule to start at and the input.
let result = rules.parse_entire("start", "Hello, Jimmy!");
```

Running the above code will yield a list of tokens (`["Hello, ", "Jimmy", "!"]`).
It is returned as a `ParseValue`, which is an `Arc<dyn Any>`.
In order to extract the values you want, you can use `.downcast_ref::<Vec<ParseValue>()` (or any type you wish to match on). In this case, each item in the `Vec<ParseValue>` can be downcast into a `psi_parser::Token`, which represents a matched terminal.

In order to make the parse output more useful, transformer actions can be attributed to each rule definition using `=>`.
These transformer actions are expressions of the type `Fn(&Vec<ParseValue>) -> ParseValue`.
Currently, the best way to use these actions is to match on indices using `downcast_ref` (as above) and clone when needed.

You can return anything as a `ParseValue`, as long as it has a `'static` lifetime, by using `.into_value()`, as seen below.

Make sure to remember the semicolon (`;`) at the end of the transformer expression!

```rust
use psi_parser::prelude::*;

let rules = rules!{
    start {
        (list)
    }

    list {
        ("[" list_inner "]") => |v| v[1].clone().into_value();
        // Empty list
        ("[]") => |_| (Vec::<Token>::new()).into_value();
    }

    list_inner {
        (name) => |v| {
            match v[0].downcast_ref::<Token>() {
                Some(token) => vec![token.clone()].into_value(),
                _ => unreachable!()
            }

        };

        (list_inner "," name) => |v| match (v[0].downcast_ref::<Vec<Token>>(), v[2].downcast_ref::<Token>()) {
            (Some(list), Some(name)) => {
                let mut vec = list.clone();
                vec.extend(name.clone());
                vec.into_value()
            }

            _ => unreachable!()
        };
    }

    name {
        ("John")
        ("Jane")
        ("Jeremiah")
        ("Josh")
        ("Jimmy")
    }
};
```

Using the above code to parse "[John,John,Josh,Jimmy,Jane,Jeremiah]" will yield a `ParseValue` that can be downcast to a `Vec` of `psi_parser::Token` of the names.

The default functionality when a transformer isn't specified is to return the `Vec<ParseValue>` as a `ParseValue`, or return the single value inside the `Vec` if it only has one value.

You can additionally use a any type as a `ParseValue` as long as its lifetime is `'static`.

```rust
use psi_parser::prelude::*;

#[derive(Clone)]
enum Name {
    John,
    Jane,
    Jeremiah,
    Josh,
    Jimmy
}

let rules = rules! {
    start {
        (name)
    }

    name {
        ("John") => |_| Name::John.into_value();
        ("Jane") => |_| Name::Jane.into_value();
        ("Jeremiah") => |_| Name::Jeremiah.into_value();
        ("Josh") => |_| Name::Josh.into_value();
        ("Jimmy") => |_| Name::Jimmy.into_value();
    }
}
```

## Known issues

- The current parsing implementation is recursive using function calls. This could be optimized to use loops instead.
- Errors (`ParseError`) are not very straightforward.

## Examples

See the `examples` directory for the examples.
They can be run using `cargo run --example <example_name>`.

Shorter examples can be found at the `tests` module at the end of `src/lib.rs`.

## License

See LICENSE for details.
