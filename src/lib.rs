use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Debug,
    str::Chars,
};

pub mod prelude {
    pub use super::{
        rule, rules, IntoParseValue as _, ParseError, ParseResult, ParseValue, Rule, Rules,
        Transformer,
    };
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseValue<T>
where
    T: Clone + Debug + PartialEq,
{
    Token(String),
    List(Vec<ParseValue<T>>),
    Integer(i32),
    Float(f32),
    String(String),
    Map(HashMap<String, ParseValue<T>>),
    Value(T),
}

impl<T: Clone + Debug + PartialEq> From<Vec<ParseValue<T>>> for ParseValue<T> {
    fn from(value: Vec<ParseValue<T>>) -> Self {
        ParseValue::List(value)
    }
}

impl<T: Clone + Debug + PartialEq> From<i32> for ParseValue<T> {
    fn from(value: i32) -> Self {
        ParseValue::Integer(value)
    }
}

impl<T: Clone + Debug + PartialEq> From<f32> for ParseValue<T> {
    fn from(value: f32) -> Self {
        ParseValue::Float(value)
    }
}

impl<T: Clone + Debug + PartialEq> From<String> for ParseValue<T> {
    fn from(value: String) -> Self {
        ParseValue::String(value)
    }
}

impl<T: Clone + Debug + PartialEq> From<HashMap<String, ParseValue<T>>> for ParseValue<T> {
    fn from(value: HashMap<String, ParseValue<T>>) -> Self {
        ParseValue::Map(value)
    }
}

pub trait IntoParseValue: Clone + Debug + PartialEq {
    fn into_value(self) -> ParseValue<Self>;
}

impl<T: Clone + Debug + PartialEq> IntoParseValue for T {
    fn into_value(self) -> ParseValue<Self> {
        ParseValue::Value(self)
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

pub type ParseResult<'a, T> = Result<Option<(ParseValue<T>, Input<'a>)>, ParseError>;

pub type Transformer<T> = Box<dyn Fn(&Vec<ParseValue<T>>) -> ParseValue<T>>;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RulePart {
    Term(String),
    NonTerm(String),
    Recurse,
}

pub enum RuleTree<T: Clone + Debug + PartialEq> {
    Part {
        part: RulePart,
        nexts: Vec<RuleTree<T>>,
    },

    End {
        transformer: Option<Transformer<T>>,
    },
}

impl<T: Clone + Debug + PartialEq> RuleTree<T> {
    fn parse<'a>(
        &self,
        rules: &Rules<T>,
        current_rule: &str,
        input: Input<'a>,
        buffer: &Vec<ParseValue<T>>,
        recursive: bool,
    ) -> ParseResult<'a, T> {
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

                    let parse_value = ParseValue::Token(literal.clone());

                    let mut buffer = buffer.clone();
                    buffer.push(parse_value);

                    parse_rule_trees(rules, current_rule, nexts, input, buffer, recursive)
                }
                RulePart::NonTerm(rule_name) => rules
                    .parse_rule(rule_name, input, vec![], false)
                    .and_then(|res| match res {
                        Some((parse_value, input)) => {
                            let mut buffer = buffer.clone();
                            buffer.push(parse_value);

                            parse_rule_trees(rules, current_rule, nexts, input, buffer, recursive)
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

                                    match parse_recursively(
                                        rules,
                                        current_rule,
                                        nexts,
                                        input.clone(),
                                        &buffer,
                                    ) {
                                        None => Some((
                                            if buffer.len() == 1 {
                                                buffer.remove(0)
                                            } else {
                                                ParseValue::List(buffer)
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
                            ParseValue::List(buffer.clone())
                        }
                    }
                };

                Ok(Some((res, input)))
            }
        }
    }
}

pub struct Rule<T: Clone + Debug + PartialEq> {
    pub name: String,
    pub parts: Vec<RulePart>,
    pub transformer: Option<Transformer<T>>,
}

impl<T: Clone + Debug + PartialEq> From<Rule<T>> for RuleTree<T> {
    fn from(val: Rule<T>) -> Self {
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

pub struct Rules<T: Clone + Debug + PartialEq>(HashMap<String, Vec<RuleTree<T>>>);

impl<T: Clone + Debug + PartialEq> Rules<T> {
    pub fn new(rules: impl IntoIterator<Item = Rule<T>>) -> Self {
        let mut map: HashMap<String, Vec<RuleTree<T>>> = HashMap::new();

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
                .map(|(rule_name, rule_trees)| (rule_name, smush(rule_trees)))
                .collect(),
        )
    }

    pub fn parse_entire<'a>(
        &self,
        start_rule: &str,
        input: impl Into<Input<'a>>,
    ) -> Result<ParseValue<T>, ParseError> {
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
                None => Ok(ParseValue::List(Vec::new())),
            })
    }

    pub fn parse<'a>(
        &self,
        start_rule: &str,
        input: impl Into<Input<'a>>,
    ) -> Result<ParseValue<T>, ParseError> {
        self.parse_rule(start_rule, input.into(), vec![], false)
            .map(|res| {
                res.map(|x| x.0)
                    .unwrap_or_else(|| ParseValue::List(Vec::new()))
            })
    }

    fn parse_rule<'a>(
        &self,
        rule_name: &str,
        input: Input<'a>,
        buffer: Vec<ParseValue<T>>,
        recursive: bool,
    ) -> ParseResult<'a, T> {
        self.0
            .get(rule_name)
            .ok_or_else(|| ParseError::RuleNotFound {
                rule_name: rule_name.to_owned(),
            })
            .and_then(|rule| parse_rule_trees(self, rule_name, &rule[..], input, buffer, recursive))
    }
}

fn parse_rule_trees<'a, T: Clone + Debug + PartialEq>(
    rules: &Rules<T>,
    current_rule: &str,
    rule_trees: &[RuleTree<T>],
    input: Input<'a>,
    buffer: Vec<ParseValue<T>>,
    recursive: bool,
) -> ParseResult<'a, T> {
    let mut errors = HashSet::new();
    for tree in rule_trees {
        match tree.parse(rules, current_rule, input.clone(), &buffer, recursive) {
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

fn parse_recursively<'a, T: Clone + Debug + PartialEq>(
    rules: &Rules<T>,
    current_rule: &str,
    rule_trees: &[RuleTree<T>],
    input: Input<'a>,
    buffer: &Vec<ParseValue<T>>,
) -> Option<(ParseValue<T>, Input<'a>)> {
    match parse_rule_trees(
        rules,
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
                Some((ParseValue::List(buffer.clone()), input))
            }
        }
        Ok(Some((v, input))) => parse_recursively(rules, current_rule, rule_trees, input, &vec![v]),
        Ok(None) => None,
    }
}

fn smush<T: Clone + Debug + PartialEq>(trees: Vec<RuleTree<T>>) -> Vec<RuleTree<T>> {
    let mut v = trees
        .into_iter()
        .fold(vec![], |mut trees: Vec<RuleTree<T>>, tree| match tree {
            RuleTree::Part { part, nexts } => {
                for t in &mut trees {
                    match t {
                        RuleTree::Part { part: p, nexts: n } if p == &part => {
                            n.extend(nexts);

                            let nexts = std::mem::take(n);
                            *n = smush(nexts);

                            return trees;
                        }
                        _ => {}
                    }
                }

                trees.push(RuleTree::Part {
                    part,
                    nexts: smush(nexts),
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
        let transformer: Option<Box<dyn Fn(&Vec<ParseValue<()>>) -> ParseValue<()>>> = None;

        $(
            let transformer: Option<Box<dyn Fn(&Vec<ParseValue<()>>) -> ParseValue<()>>> = Some(Box::new($transformer));
        )?

        Rule {
            name: stringify!($name).to_owned(),
            parts: vec![$(psi_parser::rule_part!($tt)),*],
            transformer
        }
    }};

    (#[type = $type:ty] $name:ident: ($($tt:tt)*) $(=> $transformer:expr)?) => {{
        #[allow(unused_variables)]
        let transformer: Option<Box<dyn Fn(&Vec<ParseValue<$type>>) -> ParseValue<$type>>> = None;

        $(
            let transformer: Option<Box<dyn Fn(&Vec<ParseValue<$type>>) -> ParseValue<$type>>> = Some(Box::new($transformer));
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

        Rules::<()>::new(rules)
    }};

    (
        #[type = $type:ty]

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
            rules.push(rule!(#[type = $type] $rule_name: ($($tt)*) $(=> $transformer)?).into());
        )*)*

        Rules::<$type>::new(rules)
    }};
}

#[cfg(test)]
mod tests {
    use crate as psi_parser;
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

        assert_eq!(
            Ok(ParseValue::List(vec![
                ParseValue::Token("hello".to_owned()),
                ParseValue::Token(" ".to_owned()),
                ParseValue::Token("world".to_owned())
            ])),
            result
        );
    }

    #[test]
    fn aab() {
        let rules = rules! {
            start { (aab) }
            aab { ("b")
                 ("a" aab) => |v| {
                    let rest = v[1].clone();

                    if let ParseValue::List(mut list) = rest {
                        list.insert(0, v[0].clone());

                        list.into()
                    } else {
                        vec![v[0].clone(), rest].into()
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
            Ok(ParseValue::Token("b".to_owned())),
            rules.parse("start", input0)
        );

        assert_eq!(
            Ok(ParseValue::List(vec![
                ParseValue::Token("a".to_owned()),
                ParseValue::Token("b".to_owned())
            ])),
            rules.parse("start", input1)
        );

        assert_eq!(
            Ok(ParseValue::List(vec![
                ParseValue::Token("a".to_owned()),
                ParseValue::Token("a".to_owned()),
                ParseValue::Token("b".to_owned())
            ])),
            rules.parse("start", input2)
        );

        assert_eq!(
            Ok(ParseValue::List(vec![
                ParseValue::Token("a".to_owned()),
                ParseValue::Token("a".to_owned()),
                ParseValue::Token("a".to_owned()),
                ParseValue::Token("b".to_owned())
            ])),
            rules.parse("start", input3)
        );

        assert_eq!(
            Err(ParseError::UnexpectedChar {
                current_rule: "aab".to_owned(),
                char: Some('c'),
                pos: 0,
                row: 1,
                col: 1,
            }),
            rules.parse("start", input4)
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
                    match (&v[0], &v[4]) {
                        (ParseValue::Integer(a), ParseValue::Integer(b)) => (a + b).into(),
                        _ => unreachable!()
                    }
                };
            }

            factor {
                (int)
                ("(" ws expr ws ")") => |v| v[2].clone();
                (factor ws "*" ws factor) => |v| {
                    match (&v[0], &v[4]) {
                        (ParseValue::Integer(a), ParseValue::Integer(b)) => (a * b).into(),
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
                ("0") => |_| ParseValue::Integer(0);
                (_int) => |v| match &v[0] {
                    ParseValue::String(s) => ParseValue::Integer(s.parse().unwrap()),
                    _ => unreachable!(),
                };
            }

            _int {
                (digit_nonzero) => |v| match &v[0] {
                    ParseValue::Token(digit) => digit.clone().into(),
                    _ => unreachable!()
                };

                (_int digit_nonzero) => |v| match (&v[0], &v[1]) {
                    (ParseValue::String(int), ParseValue::Token(digit)) => format!("{int}{digit}").into(),
                    _ => unreachable!()
                };

                (_int "0") => |v| match &v[0] {
                    ParseValue::String(int) => format!("{int}0").into(),
                    _ => unreachable!()
                };
            }
        };

        let input = "       12 * 5 + 16 * 2     ";

        let expected_result = 12 * 5 + 16 * 2;

        let result = rules.parse("start", input);

        assert_eq!(Ok(ParseValue::Integer(expected_result)), result)
    }
}
