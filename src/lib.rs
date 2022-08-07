mod utils;

//TODO: expand input types and optimize their cloning efficiency
pub mod input;

//TODO: split into grammar and rules, maybe grammar > rules with pub use Rules
pub mod grammar;

//TODO: make a non-recursive parsing function
pub mod parse;

mod psi_macro;

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        grammar::{Grammar, RuleDef, RuleEntry, RulePart, Rules},
        input::CharsInput,
        parse::{parsed::ParseTree, Parser},
    };

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
                    },
                ],
            ),
        ]);

        rules.into_grammar()
    }

    fn expr_grammar() -> Grammar {
        Grammar::new([])
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
        //let expected_grammar = expr_grammar();

        println!("Compiled Grammar:\n{:#?}", compiled_grammar);

        //assert_eq!(expected_grammar, compiled_grammar);

        let input = "12+33*85+233".chars();
        let expected_result = 12.0 + 33.0 * 85.0 + 233.0;

        let mut parser = Parser::<CharsInput>::new(input);

        let result = parser.parse(&compiled_grammar);

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
