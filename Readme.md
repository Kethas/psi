# Psi
> **Warning**
>
> This project is under development. It has many known bugs and missing features.

Psi is a parser aiming to trade performance for ease of use.

## Grammar
A grammar is a collection of rules. 
Each rule is comprised of one or more definitions.
Each definition is a list of atoms. 
An atom is either a literal or a rule. 
> *This is a parallel of terminals and nonterminals, or of tokens and productions.* 

These constructs can be used to represent anything that can be parsed.
This means that a grammar is essentially a declarative assembly language for a Psi parser.

Ideally, there could be a more higher-level rust macro or language to compile into a lower-level grammar.

## Using this crate
To parse using Psi, you must first compile a grammar. 
The easiest way to do this is to use the macro from the module `psi::psi_macro`.

Using the `psi` or `rules` macro, you can declaratively define the rules of the grammar, including precedence and associativity.

```rust
// include the prelude.
use psi::*;

let grammar = psi!{
    start: expr;

    expr: "()",
          ("(" expr ")");
    
    @prec left = 20,
    expr: (expr "+" expr),
          (expr "-" expr),
          expr;
    
    @prec left = 10,
    expr: (expr "*" expr),
          (expr "/" expr),
          expr;
}
```

The Grammar starts at the rule called `start`. 
Rules can be defined as seen above.

Then a parser can be used to obtain a detailed parse tree.

```rust
let grammar = psi!{...};
let source = "...".chars();
let mut parser = Parser::<CharsInput>::new(source);

let result = parser.parse(&grammar);
```

Parsing the above grammar will yield a parse tree. 
In order to make the parsed output more useful, actions can be attributed to each rule definition.

```rust
let grammar = psi!{
      expr: number -> |o| Ok(Float( o.to_string().parse()? )),
            (expr "+" expr) -> |o| Ok(Float( o[0].as_float()? + o[1].as_float()? ));
}
```

The type of an action is a `psi::grammar::RuleTransformer`, or rather, a closure receiving a `ParseObject` and returning an `eyre::Result<ParseObject>`. 

Actions are powerful enough to both vaidate data, but also transform it, allowing for simple parsing programs (such as a calculator) to be directly defined as a grammar. 

Actions can be used to build an AST to be returned instead of the parse tree.

## Known issues
   - Left recursion is impossible currently. This is possible to implement, and kind of necessary because recursion is encouraged.
   - The current parsing implementation is recursive using function calls. This could be optimized in some way

## License
See LICENSE for details.