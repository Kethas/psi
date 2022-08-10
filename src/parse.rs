use crate::{grammar::*, input::Input, utils::Boxs};

pub mod parsed;
use parsed::*;

#[derive(Clone, Debug)]
pub enum ParseError {
    Many(Boxs<ParseError>),
    Unexpected {
        expected: String,
        found: String,
        pos: usize,
    },
    NoSuchRule {
        name: String,
        pos: usize,
    },
}

pub type ParseResult = Result<ParseObject, ParseError>;

#[derive(Clone, Debug)]
pub struct Parser<I: Input> {
    input: I,
}

impl<I: Input> Parser<I> {
    pub fn new(input: impl Into<I>) -> Self {
        Self {
            input: input.into(),
        }
    }

    pub fn parse(&mut self, grammar: &Grammar) -> ParseResult {
        let tree_buffer = self.parse_rule_by_name(grammar, "start")?;

        Ok(tree_buffer.transfrom())
    }

    pub fn parse_rule_by_name(
        &mut self,
        grammar: &Grammar,
        rule_name: &str,
    ) -> Result<TreeBuffer, ParseError> {
        let rule = grammar
            .get_rule(rule_name)
            .ok_or_else(|| ParseError::NoSuchRule {
                name: rule_name.to_owned(),
                pos: self.input.get_pos(),
            })?;

        self.parse_rule(grammar, rule, rule_name, 0)
    }

    pub fn parse_rule(
        &mut self,
        grammar: &Grammar,
        rule: &[RuleTree],
        base: &str,
        depth: usize,
    ) -> Result<TreeBuffer, ParseError> {
        let mut errors: Vec<ParseError> = Vec::new();
        for node in rule {
            let mut cloned = self.clone();
            match cloned.parse_rule_node(grammar, node, base, depth) {
                Ok(r) => {
                    self.input.set_pos(cloned.input.get_pos());

                    return Ok(r);
                }
                Err(ParseError::Many(v)) => errors.extend(v.into_vec()),
                Err(e) => errors.push(e),
            }
        }

        // at this point `errors` cannot be empty because `rule` is guaranteed to not be empty.
        // therefore the function would have either returned an `ok` early or accumulated 1 or more errors.
        if errors.len() == 1 {
            Err(errors.pop().unwrap())
        } else {
            Err(ParseError::Many(errors.into_boxed_slice()))
        }
    }

    //TODO: implement left recursion
    pub fn parse_rule_node(
        &mut self,
        grammar: &Grammar,
        node: &RuleTree,
        base: &str,
        depth: usize,
    ) -> Result<TreeBuffer, ParseError> {
        let (val, rest): (TreeBuffer, &[RuleTree]) = match node {
            RuleTree::End(action) => {
                return Ok(TreeBuffer::Rule(
                    base.to_owned(),
                    Vec::new(),
                    action.clone(),
                ))
            }
            RuleTree::Lit(lit, rest) => {
                let mut i = 0;
                for c in lit.chars() {
                    let other = self.input.current().ok_or_else(|| ParseError::Unexpected {
                        expected: lit.clone(),
                        found: "<EOF>".to_owned(),
                        pos: self.input.get_pos(),
                    })?;

                    if c != other {
                        let pos = self.input.get_pos();
                        return Err(ParseError::Unexpected {
                            expected: lit.clone(),
                            found: self.input.slice_to_string((pos - i)..pos),
                            pos,
                        });
                    }

                    self.input.advance();
                    i += 1;
                }

                (TreeBuffer::Literal(lit.clone()), rest)
            }
            RuleTree::Rul(name, rest) => {
                let val = self.parse_rule_by_name(grammar, name)?;

                (val, rest)
            }
        };

        let mut res = self.parse_rule(grammar, rest, base, depth + 1)?;

        match &mut res {
            // push the value into the beginning of the return buffer
            TreeBuffer::Rule(name, vals, ..) if name == base => {
                vals.insert(0, val);
            }
            _ => unreachable!(),
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use crate::{input::CharsInput, utils::IntoBoxs};

    use super::*;

    fn aaab_grammar() -> Grammar {
        let mut g = Grammar::empty();

        g.add_rule(
            "start",
            [RuleTree::Rul(
                "A".to_owned(),
                [RuleTree::End(RuleAction::default())].into_boxed_slice(),
            )],
        );

        g.add_rule(
            "A",
            [
                RuleTree::Lit(
                    "b".to_owned(),
                    [RuleTree::End(RuleAction::default())].into_boxed_slice(),
                ),
                RuleTree::Lit(
                    "a".to_owned(),
                    [RuleTree::Rul(
                        "A".to_owned(),
                        [RuleTree::End(RuleAction::default())].into_boxed_slice(),
                    )]
                    .into_boxed_slice(),
                ),
            ],
        );

        g
    }

    #[test]
    fn test_aaab_rule() {
        let input = "aaab";
        let expected_a_count = 3;
        let expected_b_count = 1;

        let grammar = aaab_grammar();
        let mut parser = Parser::<CharsInput>::new(input.chars());

        let result = parser.parse(&grammar);

        println!("result: {result:?}");

        if result.is_err() {
            panic!("{:?}", result.unwrap_err());
        }

        let result = result.unwrap();

        let mut a_count = 0;
        let mut b_count = 0;

        let parsed = result;
        fn count_aaab(parsed: &ParseObject, a_count: &mut usize, b_count: &mut usize) {
            match parsed {
                ParseObject::Literal(a) if a == "a" => *a_count += 1,
                ParseObject::Literal(b) if b == "b" => *b_count += 1,
                ParseObject::Literal(_) => unreachable!(),
                ParseObject::Rule(_, inner) => inner
                    .iter()
                    .map(|x| count_aaab(x, a_count, b_count))
                    .collect(),
                _ => {}
            };
        }

        count_aaab(&parsed, &mut a_count, &mut b_count);

        assert_eq!(a_count, expected_a_count);
        assert_eq!(b_count, expected_b_count);

        // fail on purpose to show stdout
        //assert!(false);
    }

    fn aaab_actions_grammar() -> Grammar {
        use crate::psi;
        use std::sync::{Arc, Mutex};

        let mut a_count = Arc::new(Mutex::new(0));
        let mut b_count = Arc::new(Mutex::new(0));

        psi! {
            start: a -> {let (a_count, b_count) = (a_count.clone(), b_count.clone()); move |_| {
                use ParseObject::*;
                Ok(List(vec![Int(*a_count.lock().unwrap()), Int(*b_count.lock().unwrap())]))
            }};

            a: "b" -> {let b_count = b_count.clone(); move |x| {
                *b_count.lock().unwrap() += 1;
                Ok(x)
            }},
               ("a" a) -> {let a_count = a_count.clone(); move |x| {
                *a_count.lock().unwrap() += 1;
                Ok(x)
            }};
        }
    }

    #[test]
    fn test_aaab_actions() {
        let source = "aaab".chars();
        let mut parser = Parser::<CharsInput>::new(source);

        let out = parser.parse(&aaab_actions_grammar()).unwrap();

        use ParseObject::*;
        
        assert_eq!(List(vec![Int(3), Int(1)]), out)

    }
}
