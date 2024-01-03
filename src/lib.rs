use std::{
    any::Any,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
    str::Chars,
};

use derive_more::{Deref, DerefMut, Display, From};

pub mod prelude {
    pub use super::{
        rule, rules, IntoParseValue as _, ParseError, ParseResult, ParseValue, Rule, Rules, Token,
        Transformer,
    };
}

// Parsing Result Types
pub type ParseValue = Rc<dyn Any>;

pub trait IntoParseValue {
    fn into_value(self) -> ParseValue;
}

impl<T: Any> IntoParseValue for T {
    fn into_value(self) -> ParseValue {
        Rc::new(self)
    }
}

#[derive(Clone, Debug, Display, Deref, DerefMut, From, Eq, PartialEq, Hash)]
pub struct Token(String);

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ParseError {
    RuleNotFound {
        rule_name: String,
    },
    UnexpectedChar {
        current_rule: String,
        char: Option<char>,
        pos: usize,
        row: usize,
        col: usize,
    },
    MultipleErrors {
        current_rule: String,
        errors: Vec<ParseError>,
    },
}

// Long Type Aliases

pub type ParseResult<'a> = Result<Option<(ParseValue, Input<'a>)>, ParseError>;

pub type Transformer = Box<dyn Fn(&Vec<ParseValue>) -> ParseValue>;

// Input

#[derive(Clone)]
pub struct Input<'a> {
    chars: Chars<'a>,
    pos: usize,
    col: usize,
    row: usize,
}

impl<'a> Input<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            pos: 0,
            col: 1,
            row: 1,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        self.chars.next().map(|c| {
            self.pos += 1;

            if c == '\n' {
                self.row += 1;
                self.col = 1;
            }

            c
        })
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row, self.col)
    }
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value.chars())
    }
}

impl<'a> From<&'a String> for Input<'a> {
    fn from(value: &'a String) -> Self {
        Self::new(value.chars())
    }
}

impl<'a> From<Chars<'a>> for Input<'a> {
    fn from(value: Chars<'a>) -> Self {
        Self::new(value)
    }
}

// Grammar

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RulePart {
    Term(String),
    NonTerm(String),
    Recurse,
}

pub enum RuleTree {
    Part {
        part: RulePart,
        nexts: Vec<RuleTree>,
    },

    End {
        transformer: Option<Transformer>,
    },
}

impl RuleTree {
    fn parse<'a>(
        &self,
        rules: &Rules,
        current_rule: &str,
        input: Input<'a>,
        buffer: &Vec<ParseValue>,
        recursive: bool,
    ) -> ParseResult<'a> {
        match self {
            RuleTree::Part { part, nexts } => match part {
                RulePart::Term(literal) => {
                    let mut literal_chars = literal.chars();

                    let mut input = input;

                    loop {
                        let mut i = input.clone();

                        let (row, col) = i.row_col();

                        let (i_char, l_char) = (i.next(), literal_chars.next());
                        match (i_char, l_char) {
                            (None, None) => break,
                            (None, Some(_)) => {
                                return Err(ParseError::UnexpectedChar {
                                    current_rule: current_rule.to_owned(),
                                    char: None,
                                    pos: i.pos(),
                                    row,
                                    col,
                                })
                            }
                            (Some(_), None) => break,
                            (Some(c0), Some(c1)) if c0 == c1 => {}
                            (char @ Some(_), Some(_)) => {
                                return Err(ParseError::UnexpectedChar {
                                    current_rule: current_rule.to_owned(),
                                    char,
                                    pos: i.pos() - 1,
                                    row,
                                    col,
                                })
                            }
                        }

                        input = i;
                    }

                    let parse_value = Rc::new(Token(literal.clone()));

                    let mut buffer = buffer.clone();
                    buffer.push(parse_value);

                    rules.parse_rule_trees(current_rule, nexts, input, buffer, recursive)
                }
                RulePart::NonTerm(rule_name) => rules
                    .parse_rule(rule_name, input, vec![], false)
                    .and_then(|res| match res {
                        Some((parse_value, input)) => {
                            let mut buffer = buffer.clone();
                            buffer.push(parse_value);

                            rules.parse_rule_trees(current_rule, nexts, input, buffer, recursive)
                        }
                        None => Ok(None),
                    }),
                RulePart::Recurse => {
                    if recursive {
                        Ok(None)
                    } else {
                        rules
                            .parse_rule(current_rule, input, buffer.clone(), true)
                            .map(|res| {
                                res.and_then(|(parse_value, input)| {
                                    let mut buffer = buffer.clone();
                                    buffer.push(parse_value);

                                    match rules.parse_recursively(
                                        current_rule,
                                        nexts,
                                        input.clone(),
                                        &buffer,
                                    ) {
                                        None => Some((
                                            if buffer.len() == 1 {
                                                buffer.remove(0)
                                            } else {
                                                Rc::new(buffer)
                                            },
                                            input,
                                        )),
                                        res => res,
                                    }
                                })
                            })
                    }
                }
            },
            RuleTree::End { transformer } => {
                let res = match transformer {
                    Some(transformer) => transformer(buffer),
                    None => {
                        if buffer.len() == 1 {
                            buffer[0].clone()
                        } else {
                            Rc::new(buffer.clone())
                        }
                    }
                };

                Ok(Some((res, input)))
            }
        }
    }
}

pub struct Rule {
    pub name: String,
    pub parts: Vec<RulePart>,
    pub transformer: Option<Transformer>,
}

impl From<Rule> for RuleTree {
    fn from(val: Rule) -> Self {
        let mut tree = RuleTree::End {
            transformer: val.transformer,
        };

        for part in val.parts.into_iter().rev() {
            let part = match part {
                RulePart::Recurse => RulePart::NonTerm(val.name.clone()),
                p => p,
            };

            tree = RuleTree::Part {
                part,
                nexts: vec![tree],
            };
        }

        if let RuleTree::Part { part, .. } = &mut tree {
            if part == &RulePart::NonTerm(val.name) {
                *part = RulePart::Recurse;
            }
        }

        tree
    }
}

pub struct Rules(HashMap<String, Vec<RuleTree>>);

impl Rules {
    pub fn new(rules: impl IntoIterator<Item = Rule>) -> Self {
        let mut map: HashMap<String, Vec<RuleTree>> = HashMap::new();

        for rule in rules {
            match map.entry(rule.name.clone()) {
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    o.get_mut().push(rule.into());
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(vec![rule.into()]);
                }
            }
        }

        Rules(
            map.into_iter()
                .map(|(rule_name, rule_trees)| (rule_name, Self::smush(rule_trees)))
                .collect(),
        )
    }

    pub fn parse_entire<'a>(
        &self,
        start_rule: &str,
        input: impl Into<Input<'a>>,
    ) -> Result<ParseValue, ParseError> {
        self.parse_rule(start_rule, input.into(), vec![], false)
            .and_then(|res| match res {
                Some((value, mut input)) => {
                    let (row, col) = input.row_col();

                    if let Some(char) = input.next() {
                        Err(ParseError::UnexpectedChar {
                            current_rule: "<parse_entire>".to_owned(),
                            char: Some(char),
                            pos: input.pos() - 1,
                            row,
                            col,
                        })
                    } else {
                        Ok(value)
                    }
                }
                None => Ok(Rc::new(Vec::<ParseValue>::new())),
            })
    }

    pub fn parse<'a>(
        &self,
        start_rule: &str,
        input: impl Into<Input<'a>>,
    ) -> Result<ParseValue, ParseError> {
        self.parse_rule(start_rule, input.into(), vec![], false)
            .map(|res| {
                res.map(|x| x.0)
                    .unwrap_or_else(|| Rc::new(Vec::<ParseValue>::new()))
            })
    }

    fn parse_rule<'a>(
        &self,
        rule_name: &str,
        input: Input<'a>,
        buffer: Vec<ParseValue>,
        recursive: bool,
    ) -> ParseResult<'a> {
        self.0
            .get(rule_name)
            .ok_or_else(|| ParseError::RuleNotFound {
                rule_name: rule_name.to_owned(),
            })
            .and_then(|rule| self.parse_rule_trees(rule_name, &rule[..], input, buffer, recursive))
    }

    fn parse_rule_trees<'a>(
        &self,
        current_rule: &str,
        rule_trees: &[RuleTree],
        input: Input<'a>,
        buffer: Vec<ParseValue>,
        recursive: bool,
    ) -> ParseResult<'a> {
        let mut errors = HashSet::new();
        for tree in rule_trees {
            match tree.parse(self, current_rule, input.clone(), &buffer, recursive) {
                res @ Ok(Some(_)) => return res,
                Err(err) => {
                    errors.insert(err);
                }
                Ok(None) => {}
            }
        }

        if errors.is_empty() {
            Ok(None)
        } else if errors.len() == 1 {
            Err(errors.drain().next().unwrap())
        } else {
            Err(ParseError::MultipleErrors {
                current_rule: current_rule.to_owned(),
                errors: errors.into_iter().collect(),
            })
        }
    }

    fn parse_recursively<'a>(
        &self,
        current_rule: &str,
        rule_trees: &[RuleTree],
        input: Input<'a>,
        buffer: &Vec<ParseValue>,
    ) -> Option<(ParseValue, Input<'a>)> {
        match self.parse_rule_trees(
            current_rule,
            rule_trees,
            input.clone(),
            buffer.clone(),
            true,
        ) {
            Err(_) => {
                if buffer.len() == 1 {
                    Some((buffer[0].clone(), input))
                } else {
                    Some((Rc::new(buffer.clone()), input))
                }
            }
            Ok(Some((v, input))) => {
                self.parse_recursively(current_rule, rule_trees, input, &vec![v])
            }
            Ok(None) => None,
        }
    }

    fn smush(trees: Vec<RuleTree>) -> Vec<RuleTree> {
        let mut v = trees
            .into_iter()
            .fold(vec![], |mut trees: Vec<RuleTree>, tree| match tree {
                RuleTree::Part { part, nexts } => {
                    for t in &mut trees {
                        match t {
                            RuleTree::Part { part: p, nexts: n } if p == &part => {
                                n.extend(nexts);

                                let nexts = std::mem::take(n);
                                *n = Self::smush(nexts);

                                return trees;
                            }
                            _ => {}
                        }
                    }

                    trees.push(RuleTree::Part {
                        part,
                        nexts: Self::smush(nexts),
                    });

                    trees
                }
                RuleTree::End { transformer } => {
                    for t in &trees {
                        if let RuleTree::End { .. } = t {
                            return trees;
                        }
                    }

                    trees.push(RuleTree::End { transformer });

                    trees
                }
            });

        v.sort_by(|a, b| match (b, a) {
            (RuleTree::Part { .. }, RuleTree::End { .. }) => Ordering::Greater,
            (RuleTree::End { .. }, RuleTree::Part { .. }) => Ordering::Less,
            (RuleTree::Part { part: p0, .. }, RuleTree::Part { part: p1, .. }) => match (p0, p1) {
                (RulePart::Recurse, _) => Ordering::Greater,
                (_, RulePart::Recurse) => Ordering::Less,
                (RulePart::Term(lit0), RulePart::Term(lit1)) => {
                    lit0.chars().count().cmp(&lit1.chars().count())
                }
                (RulePart::Term(_), RulePart::NonTerm(_)) => Ordering::Greater,
                _ => Ordering::Less,
            },
            _ => std::cmp::Ordering::Equal,
        });

        v
    }
}

// Macros

#[allow(dead_code)]
#[macro_export]
macro_rules! rule_part {
    ($lit:literal) => {
        psi_parser::RulePart::Term(String::from($lit))
    };

    ($rule:ident) => {
        psi_parser::RulePart::NonTerm(stringify!($rule).to_owned())
    };
}

#[allow(dead_code)]
#[macro_export]
macro_rules! rule {
    ($name:ident: ($($tt:tt)*) $(=> $transformer:expr)?) => {{

        #[allow(unused_variables)]
        let transformer: Option<Transformer> = None;

        $(
            let transformer: Option<Transformer> = Some(Box::new($transformer));
        )?

        Rule {
            name: stringify!($name).to_owned(),
            parts: vec![$(psi_parser::rule_part!($tt)),*],
            transformer
        }
    }};
}

#[allow(dead_code)]
#[macro_export]
macro_rules! rules {
    (
        $(
            $rule_name:ident {
                $(
                    ($( $tt:tt )*)
                    $(=> $transformer:expr;)?
                )+
            }
        )+
    ) => {{
        let mut rules = Vec::new();

        $($(
            rules.push(rule!($rule_name: ($($tt)*) $(=> $transformer)?).into());
        )*)*

        Rules::new(rules)
    }};
}

#[cfg(test)]
mod tests {
    use crate::{self as psi_parser};
    use psi_parser::prelude::*;

    #[test]
    fn hello_world() {
        let rules = rules! {
            start {
                (hello_world)
            }
            hello_world {
                ("hello" " " "world")
            }
        };

        let input = "hello world";

        let result = rules.parse("start", input);

        let result = result
            .expect("Could not parse")
            .downcast_ref::<Vec<ParseValue>>()
            .expect("Result should be a Vec")
            .iter()
            .map(|x| x.downcast_ref::<Token>().expect("Should be Token").clone())
            .collect::<Vec<_>>();

        assert_eq!(
            vec![
                Token("hello".to_owned()),
                Token(" ".to_owned()),
                Token("world".to_owned())
            ],
            result
        );
    }

    #[test]
    fn aab() {
        let rules = rules! {
            start { (aab) }
            aab {
                ("b")
                ("a" aab) => |v| {
                    if let Some(mut list) = v[1].downcast_ref::<Vec<Token>>().cloned() {
                        list.insert(0, v[0].clone().downcast_ref::<Token>().unwrap().clone());

                        list.into_value()
                    } else {
                        v
                            .iter()
                            .map(|token| token.downcast_ref::<Token>().unwrap().clone())
                            .collect::<Vec<Token>>()
                            .into_value()
                    }
                };
            }
        };

        let input0 = "b";
        let input1 = "ab";
        let input2 = "aab";
        let input3 = "aaab";
        let input4 = "c";

        assert_eq!(
            Some(Token("b".to_owned())),
            rules
                .parse("start", input0)
                .expect("Should be parsed")
                .downcast_ref()
                .cloned()
        );

        assert_eq!(
            Some(vec![Token("a".to_owned()), Token("b".to_owned())]),
            rules
                .parse("start", input1)
                .expect("Should be parsed")
                .downcast_ref()
                .cloned()
        );

        assert_eq!(
            Some(vec![
                Token("a".to_owned()),
                Token("a".to_owned()),
                Token("b".to_owned())
            ]),
            rules
                .parse("start", input2)
                .expect("Should be parsed")
                .downcast_ref()
                .cloned()
        );

        assert_eq!(
            Some(vec![
                Token("a".to_owned()),
                Token("a".to_owned()),
                Token("a".to_owned()),
                Token("b".to_owned())
            ]),
            rules
                .parse("start", input3)
                .expect("Should be parsed")
                .downcast_ref()
                .cloned()
        );

        assert_eq!(
            Some(ParseError::UnexpectedChar {
                current_rule: "aab".to_owned(),
                char: Some('c'),
                pos: 0,
                row: 1,
                col: 1,
            }),
            rules.parse("start", input4).err()
        );
    }

    #[test]
    fn calculator() {
        let rules = rules! {
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
                    match (v[0].downcast_ref::<i32>(), v[4].downcast_ref::<i32>()) {
                        (Some(a), Some(b)) => (a + b).into_value(),
                        _ => unreachable!()
                    }
                };
            }

            factor {
                (int)
                ("(" ws expr ws ")") => |v| v[2].clone();
                (factor ws "*" ws factor) => |v| {
                    match (v[0].downcast_ref::<i32>(), v[4].downcast_ref::<i32>()) {
                        (Some(a), Some(b)) => (a * b).into_value(),
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

            int {
                ("0") => |_| 0.into_value();
                (_int) => |v| match v[0].downcast_ref::<String>() {
                    Some(s) => s.parse::<i32>().unwrap().into_value(),
                    _ => unreachable!(),
                };
            }

            _int {
                (digit_nonzero) => |v| match v[0].downcast_ref::<Token>() {
                    Some(digit) => digit.to_string().into_value(),
                    _ => unreachable!()
                };

                (_int digit_nonzero) => |v| match (v[0].downcast_ref::<String>(), v[1].downcast_ref::<Token>()) {
                    (Some(int), Some(digit)) => format!("{int}{digit}").into_value(),
                    _ => unreachable!()
                };

                (_int "0") => |v| match v[0].downcast_ref::<String>() {
                    Some(int) => format!("{int}0").into_value(),
                    _ => unreachable!()
                };
            }
        };

        let input = "       12 * 5 + 16 * 2     ";

        let expected_result = 12 * 5 + 16 * 2;

        let result = rules.parse("start", input);

        assert_eq!(
            Some(expected_result),
            result
                .expect("Should be parsed")
                .downcast_ref::<i32>()
                .cloned()
        )
    }
}
