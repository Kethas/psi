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

Running the above code will yield a `ParseValue::List` representing the list `["Hello, ", "Jimmy", "!"]` as `ParseValue::Token`s.
In order to make the parsed output more useful, transformer actions can be attributed to each rule definition using `=>`.
These transformer actions are expressions of the type `Fn(&Vec<ParseValue>) -> ParseValue`.
Currently, the best way to use these actions is to match on indices and clone when needed.

Make sure to remember the semicolon (`;`) at the end of the transformer expression!

```rust
use psi_parser::prelude::*;

let rules = rules!{
    start {
        (list)
    }

    list {
        ("[" list_inner "]") => |v| v[1].clone();
        // Empty list
        ("[]") => |_| ParseValue::List(Vec::new());
    }

    list_inner {
        (name) => |v| ParseValue::List(v.clone());

        // v[0] and v[2] are always lists, so we match against them and use unreachable! for the rest
        (list_inner "," name) => |v| match (&v[0], &v[2]) {
            (ParseValue::List(v0), ParseValue::List(v2)) => {
                let mut vec = v0.clone();
                vec.extend(v2.clone());
                ParseValue::List(vec)
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

Using the above code to parse "[John,John,Josh,Jimmy,Jane,Jeremiah]" will yield a `ParseValue::List` containing a `Vec` of `ParseValue::Token` of the names.

The default functionality when a transformer isn't specified is to return the `Vec<ParseValue>` inside a `ParseValue::List`, or return the single value inside the `Vec` if it only has one value.

You can additionally use a custom type for `ParseValue::Value` using `#[type = Type]`

```rust
use psi_parser::prelude::*;

// Your type must implement these three traits
#[derive(Debug, Clone, PartialEq)]
enum Name {
    John,
    Jane,
    Jeremiah,
    Josh,
    Jimmy
}

let rules = rules! {
    // Declare that our custom type is Name
    #[type = Name]

    start {
        (name)
    }

    name {
        ("John") => |_| ParseValue::Value(Name::John);
        ("Jane") => |_| ParseValue::Value(Name::Jane);
        ("Jeremiah") => |_| ParseValue::Value(Name::Jeremiah);
        ("Josh") => |_| ParseValue::Value(Name::Josh);
        ("Jimmy") => |_| Name::Jimmy.into_value(); // You can also use .into_value()
    }
}
```

Note that you can use the `Into` trait for the `List`, `Integer`, `Float`, `String`, and `Map` variants of `ParseValue` with their respective types. For the `Value` variant use `.into_value()` which is included in the prelude.

This isn't the best method, but it does work until there's a better alternative.

## Known issues

- The current parsing implementation is recursive using function calls. This could be optimized to use loops instead.
- Errors (`ParseError`) are not very straightforward.

## Examples

See the `examples` directory for the examples.
They can be run using `cargo run --example <example_name>`.

## License

See LICENSE for details.
