mod utils;

//TODO: expand input types and optimize their cloning efficiency
pub mod input;
pub use input::{CharsInput, Input};

//TODO: split into grammar and rules, maybe grammar > rules with pub use Rules
pub mod grammar;
pub use grammar::{Grammar, Rules};

//TODO: make a non-recursive parsing function
pub mod parse;
pub use parse::Parser;

#[macro_use]
pub mod psi_macro;

/// This macro can be used to generate a Psi Grammar.
/// Example:
/// ```
/// # #[macro_use] extern crate psi;
/// use psi::*;
///
/// # fn main() {
/// let grammar = psi!{
///     start: a;
///
///     a: "a",
///        (b a);
///     b: "b";
/// };
///
/// let source = "ba".chars();
/// let mut parser = Parser::<CharsInput>::new(source);
///
/// let result = parser.parse(&grammar).expect("Failed to parse.");
///
/// use psi::parse::parsed::ParseTree::*;
/// assert_eq!(result,
///     Rule("start".to_owned(),
///         vec![
///             Rule("a".to_owned(), vec![
///                 Rule("b".to_owned(), vec![
///                     Literal("b".to_owned())
///                 ]),
///                 Rule("a".to_owned(), vec![
///                     Literal("a".to_owned())
///                 ])
///             ])
///         ]
///     )
/// )
/// # }
///
/// ```
pub use psi_macro::rules as psi;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        grammar::{Grammar, RuleDef, RuleEntry, RulePart, Rules},
        input::CharsInput,
        parse::{parsed::ParseTree, Parser},
    };
    use std::vec;

    fn compile_expr_grammar() -> Grammar {
        let rules = Rules::new([
            (
                "start".to_owned(),
                vec![RuleEntry {
                    definitions: vec![RuleDef {
                        parts: vec![RulePart::Rule("expr".to_owned())],
                        ..Default::default()
                    }],
                    precedence: 0,
                    ..Default::default()
                }],
            ),
            //parts: vec![RulePart::Literal("0".to_owned())],
            //
            (
                "digit_nz".to_owned(),
                vec![RuleEntry {
                    definitions: vec![
                        RuleDef {
                            parts: vec![RulePart::Literal("1".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("2".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("3".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("4".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("5".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("6".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("7".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("8".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Literal("9".to_owned())],
                            ..Default::default()
                        },
                    ],
                    precedence: 0,
                    ..Default::default()
                }],
            ),
            (
                "zero".to_owned(),
                vec![RuleEntry {
                    definitions: vec![RuleDef {
                        parts: vec![RulePart::Literal("0".to_owned())],
                        ..Default::default()
                    }],
                    precedence: 0,
                    ..Default::default()
                }],
            ),
            (
                "digit".to_owned(),
                vec![RuleEntry {
                    definitions: vec![
                        RuleDef {
                            parts: vec![RulePart::Rule("zero".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![RulePart::Rule("digit_nz".to_owned())],
                            ..Default::default()
                        },
                    ],
                    precedence: 0,
                    ..Default::default()
                }],
            ),
            (
                "number".to_owned(),
                vec![RuleEntry {
                    definitions: vec![
                        RuleDef {
                            parts: vec![RulePart::Rule("digit".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![
                                RulePart::Rule("digit".to_owned()),
                                RulePart::Rule("number".to_owned()),
                            ],
                            ..Default::default()
                        },
                    ],
                    precedence: 0,
                    ..Default::default()
                }],
            ),
            (
                "expr".to_owned(),
                vec![
                    RuleEntry {
                        precedence: 30,
                        definitions: vec![
                            RuleDef {
                                parts: vec![
                                    RulePart::Literal("-".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                ],
                                ..Default::default()
                            },
                            RuleDef {
                                parts: vec![RulePart::Rule("expr".to_owned())],
                                ..Default::default()
                            },
                        ],
                        ..Default::default()
                    },
                    RuleEntry {
                        definitions: vec![
                            RuleDef {
                                parts: vec![
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal("+".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                ],
                                ..Default::default()
                            },
                            RuleDef {
                                parts: vec![
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal("-".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                ],
                                ..Default::default()
                            },
                            RuleDef {
                                parts: vec![RulePart::Rule("expr".to_owned())],
                                ..Default::default()
                            },
                        ],
                        precedence: 20,
                        ..Default::default()
                    },
                    RuleEntry {
                        definitions: vec![
                            RuleDef {
                                parts: vec![
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal("*".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                ],
                                ..Default::default()
                            },
                            RuleDef {
                                parts: vec![
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal("/".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                ],
                                ..Default::default()
                            },
                            RuleDef {
                                parts: vec![RulePart::Rule("expr".to_owned())],
                                ..Default::default()
                            },
                        ],
                        precedence: 10,
                        ..Default::default()
                    },
                    RuleEntry {
                        definitions: vec![
                            RuleDef {
                                parts: vec![RulePart::Rule("number".to_owned())],
                                ..Default::default()
                            },
                            RuleDef {
                                parts: vec![
                                    RulePart::Literal("(".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal(")".to_owned()),
                                ],
                                ..Default::default()
                            },
                        ],
                        precedence: 0,
                        ..Default::default()
                    },
                ],
            ),
        ]);

        rules.into_grammar()
    }

    fn macro_expr_grammar() -> Grammar {
        psi! {
            start: expr;

            digit_nz: "1", "2", "3", "4", "5", "6", "7", "8", "9";
            zero: "0";
            digit: zero,
                   digit_nz;
            number: digit,
                    (digit number);

            @prec = 30,
            expr: ("-" expr),
                  expr;
            @prec = 20,
            expr: (expr "+" expr),
                  (expr "-" expr),
                  expr;
            @prec = 10,
            expr: (expr "*" expr),
                  (expr "/" expr),
                  expr;
            expr: number,
                  ("(" expr ")");
        }
    }

    fn eval_expr(parsed: &ParseTree) -> f64 {
        match parsed {
            ParseTree::End => 0.0,

            ParseTree::Rule(name, inner) if name == "expr@0" => match inner[0].to_string().as_str()
            {
                "(" => eval_expr(&inner[1]),
                num => num.to_string().parse().unwrap(),
            },

            ParseTree::Rule(name, inner)
                if inner.len() == 1
                    && ["expr", "start"]
                        .iter()
                        .any(|x| name == x || name.starts_with(&format!("{x}@"))) =>
            {
                eval_expr(&inner[0])
            }

            ParseTree::Rule(name, inner) if name == "expr@30" => -eval_expr(&inner[1]),
            ParseTree::Rule(name, inner) if name == "expr@20" => {
                match inner[1].to_string().as_str() {
                    "+" => eval_expr(&inner[0]) + eval_expr(&inner[2]),
                    "-" => eval_expr(&inner[0]) - eval_expr(&inner[2]),

                    _ => unreachable!(),
                }
            }
            ParseTree::Rule(name, inner) if name == "expr@10" => {
                match inner[1].to_string().as_str() {
                    "*" => eval_expr(&inner[0]) * eval_expr(&inner[2]),
                    "/" => eval_expr(&inner[0]) / eval_expr(&inner[2]),

                    _ => unreachable!(),
                }
            }

            x => panic!("Unreachable: {:#?}", x),
        }
    }

    #[test]
    fn test_expr_0() {
        let compiled_grammar = compile_expr_grammar();
        let macro_grammar = macro_expr_grammar();
        println!(
            "compiled:\n{:#?}\nmacro:\n{:#?}\n",
            compiled_grammar, macro_grammar
        );
        assert_eq!(compiled_grammar, macro_grammar);
        //let expected_grammar = expr_grammar();

        println!("Compiled Grammar:\n{:#?}", compiled_grammar);

        //assert_eq!(expected_grammar, compiled_grammar);

        let input = "12+33*85+233".chars();
        let expected_result = 12.0 + 33.0 * 85.0 + 233.0;

        let mut parser = Parser::<CharsInput>::new(input);

        let result = parser.parse(&macro_grammar);

        if let Err(e) = result {
            eprintln!("ParseError:\n{:#?}", e);
            panic!("Error encountered while parsing.");
        }

        let parsed = result.unwrap();

        println!("Parsed:\n{:#?}", parsed);

        assert_eq!(expected_result, eval_expr(&parsed));

        // great success! wawaweewa!
    }
}
