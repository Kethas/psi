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

### Basic Usage

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
It is returned as a `ParseValue`, which is an `Box<dyn Any>`.
In order to extract the values you want, you can use `.downcast::<Vec<ParseValue>()` (or any type you wish to match on). In this case, each item in the `Vec<ParseValue>` can be downcast into a `psi_parser::Token`, which represents a matched terminal.

### Transformer Actions

In order to make the parse output more useful, transformer actions can be attributed to each rule definition using `=>`.
These transformer actions are expressions of the type `Fn(ParseBuffer) -> ParseValue`,
where a `ParseBuffer` is a function that when called with an index, will give you the ParseValue at that index (but only once!). Each rule part corresponds to one item in the buffer.

Currently, the best way to use these actions is to match on indices using `downcast` (as above).

You can return anything as a `ParseValue`, as long as it has a `'static` lifetime, by using `.into_value()`, as seen below.

Make sure to remember the semicolon (`;`) at the end of the transformer expression!

```rust
use psi_parser::prelude::*;

let rules = rules!{
    start {
        (list)
    }

    list {
        ("[" list_inner "]") => |v, _| v(1);
        // Empty list
        ("[]") => |_, _| (Vec::<Token>::new()).into_value();
    }

    list_inner {
        (name) => |v, _| vec![*v(0).downcast::<Token>().unwrap()].into_value();

        (list_inner "," name) => |v, _| {
            let mut vec = v(0).downcast::<Vec<Token>>().unwrap()
            vec.push(*v(2).downcast::<Token>().unwrap());
            vec
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

The default functionality when a transformer isn't specified is to return a `Vec<ParseValue>` as a `ParseValue`, or return the single value inside the buffer if it only has one.

You can additionally use a any type as a `ParseValue` as long as its lifetime is `'static`.

```rust
use psi_parser::prelude::*;

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
        ("John") => |_, _| Name::John.into_value();
        ("Jane") => |_, _| Name::Jane.into_value();
        ("Jeremiah") => |_, _| Name::Jeremiah.into_value();
        ("Josh") => |_, _| Name::Josh.into_value();
        ("Jimmy") => |_, _| Name::Jimmy.into_value();
    }
};
```

### Imports

Psi grammars can be composed using `#[import (expr) as name]`
Any expression that implements `Into<psi_parser::Rules>` can be used.

See [Included Parsers](#included-parsers) for the `declare_rules!` macro which can be used to declare rules that can be easily imported.

```rust
use psi_parser::prelude::*;

let names_rules = rules! {
    name {
        ("John")
        ("Jane")
        ("Jeremiah")
        ("Josh")
        ("Jimmy")
    }
};

let rules = rules! {
    #[import (names_rules) as names]

    start {
        // Note that accessing a namespace requires another set of parentheses
        ("Hello, " (names::name) "!")
    }
};
```

You can also import without specifying `as <namespace>`, though it will merge the existing rules with the imported ones if any have the same name.
It is recommended to always import with a namespace.

### `Not`

Another type of rule part is the `Not` rule part.
It matches only when the next characters of the input do **not** match any of its literals.
It then returns a token the size (in characters) of the smallest literal.

```rust
use psi_parser::prelude::*;

let rules = rules! {
    character {
        ("'" char_inner "'") => |v, _| v(1)
    }

    char_inner {
        (char_escapes)
        ((! "'"))
    }
};
```

Note that adjacent `Not`s are merged. Therefore the following will be merged:

```rust
rule {
    ("a" (! "a"))
    ("a" (! "bcde"))
}
```

Into

```rust
rule {
    ("a" (! "a" "bcde"))
}
```

While the following will be left unchanged

```rust
rule {
    ("a" (! "a"))
    ("b" (! "bcde"))
}
```

### Included Parsers

A small set of parsers is included and can be found in the `src/rules` directory.
Each one is a unit `struct` which implements `Into<Rules>` (so it can be imported easily) as well as having the `parse` and `parse_entire` functions defined for it directly.

These can be defined using the `psi_parser::declare_rules!` macro:

```rust
use psi_parser::prelude::*;

pub enum Name {
    John,
    Jane,
    Jeremiah,
    Josh,
    Jimmy
}

declare_rules! {
    pub Names {
        name {
            ("John") => |_, _| Name::John.into_value();
            ("Jane") => |_, _| Name::Jane.into_value();
            ("Jeremiah") => |_, _| Name::Jeremiah.into_value();
            ("Josh") => |_, _| Name::Josh.into_value();
            ("Jimmy") => |_, _| Name::Jimmy.into_value();
        }
    }
}


// Can be imported!
let rules = rules! {
    #[import (Names) as names]

    start{ ((names::name)) }
};
```

If you need to reuse a single parser, it is better to use `Rules::from(<rules>)` to build a local `Rules` instead of using the `parse` or `parse_entire` functions generated by `declare_rules!`, which rebuild the rules each time.

### Transformer Errors

If a transformer action which, for example, validates data or values of tokens fails, you can return any type that implements `std::error::Error` as an error by using `.into_error()`. The rule then fails using the given error.

```rust
const ALLOWED_NAMES: &[&str] = &["John", "Jack"];

#[derive(Debug)]
struct NameError {
    name: String,
}

impl std::error::Error for NameError {}

impl std::fmt::Display for NameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { name } = self;

        f.write_fmt(format_args!(
            "Name '{name}' is not allowed. Allowed names: {ALLOWED_NAMES:?}"
        ))
    }
}

let rules = rules! {
    #[import (rules::Identifier) as id]

    start {
        ((id::identifier)) => |v, _| {
            let id = v(0).downcast::<String>().unwrap();

            if !ALLOWED_NAMES.contains(&id.as_str()) {
                NameError {
                    name: *id
                }.into_error()
            } else {
                id
            }
        };
    }
};
```

## Known issues

- Errors (`ParseError`) are not very straightforward - And since the procedural implementation are even less helpful.

## Examples

See the `examples` directory for the examples.
They can be run using `cargo run --example <example_name>`.

Shorter examples can be found in `src/tests/basic.rs`.

## License

See LICENSE for details.
