use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    str::Chars,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ParseValue {
    Token(String),
    List(Vec<ParseValue>),
    Integer(i32),
    Float(f32),
    String(String),
    Map(HashMap<String, ParseValue>),
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
    },
    MultipleErrors {
        current_rule: String,
        errors: Vec<ParseError>,
    },
}

pub type ParseResult<'a> = Result<Option<(ParseValue, Input<'a>)>, ParseError>;

pub type Transformer = Box<dyn Fn(&Vec<ParseValue>) -> ParseValue>;

#[derive(Clone)]
pub struct Input<'a> {
    source: &'a str,
    chars: Chars<'a>,
    pos: usize,
}

impl<'a> Input<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars(),
            pos: 0,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        self.chars.next().map(|c| {
            self.pos += 1;
            c
        })
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn source(&self) -> &'a str {
        self.source
    }
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

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

                        let (i_char, l_char) = (i.next(), literal_chars.next());
                        match (i_char, l_char) {
                            (None, None) => break,
                            (None, Some(_)) => {
                                return Err(ParseError::UnexpectedChar {
                                    current_rule: current_rule.to_owned(),
                                    char: None,
                                    pos: i.pos(),
                                })
                            }
                            (Some(_), None) => break,
                            (Some(c0), Some(c1)) if c0 == c1 => {}
                            (char @ Some(_), Some(_)) => {
                                return Err(ParseError::UnexpectedChar {
                                    current_rule: current_rule.to_owned(),
                                    char,
                                    pos: i.pos() - 1,
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
                .map(|(rule_name, rule_trees)| (rule_name, smush(rule_trees)))
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
                    if let Some(char) = input.next() {
                        Err(ParseError::UnexpectedChar {
                            current_rule: "<parse_entire>".to_owned(),
                            char: Some(char),
                            pos: input.pos() - 1,
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
    ) -> Result<ParseValue, ParseError> {
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
        buffer: Vec<ParseValue>,
        recursive: bool,
    ) -> ParseResult<'a> {
        self.0
            .get(rule_name)
            .ok_or_else(|| ParseError::RuleNotFound {
                rule_name: rule_name.to_owned(),
            })
            .and_then(|rule| parse_rule_trees(self, rule_name, &rule[..], input, buffer, recursive))
    }
}

fn parse_rule_trees<'a>(
    rules: &Rules,
    current_rule: &str,
    rule_trees: &[RuleTree],
    input: Input<'a>,
    buffer: Vec<ParseValue>,
    recursive: bool,
) -> ParseResult<'a> {
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

fn parse_recursively<'a>(
    rules: &Rules,
    current_rule: &str,
    rule_trees: &[RuleTree],
    input: Input<'a>,
    buffer: &Vec<ParseValue>,
) -> Option<(ParseValue, Input<'a>)> {
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
        RulePart::Term(String::from($lit))
    };

    ($rule:ident) => {
        RulePart::NonTerm(stringify!($rule).to_owned())
    };
}

#[allow(dead_code)]
#[macro_export]
macro_rules! rule {
    ($name:ident: ($($tt:tt)*) $(=> $transformer:expr)?) => {{
        #[allow(unused_variables)]
        let transformer: Option<Box<dyn Fn(&Vec<ParseValue>) -> ParseValue>> = None;

        $(
            let transformer: Option<Box<dyn Fn(&Vec<ParseValue>) -> ParseValue>> = Some(Box::new($transformer));
        )?

        Rule {
            name: stringify!($name).to_owned(),
            parts: vec![$(rule_part!($tt)),*],
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
    use super::*;

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

                        ParseValue::List(list)
                    } else {
                        ParseValue::List(vec![v[0].clone(), rest])
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
                pos: 0
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
                        (ParseValue::Integer(a), ParseValue::Integer(b)) => ParseValue::Integer(a + b),
                        _ => unreachable!()
                    }
                };
            }

            factor {
                (int)
                ("(" ws expr ws ")") => |v| v[2].clone();
                (factor ws "*" ws factor) => |v| {
                    match (&v[0], &v[4]) {
                        (ParseValue::Integer(a), ParseValue::Integer(b)) => ParseValue::Integer(a * b),
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
                    ParseValue::Token(digit) => ParseValue::String(digit.clone()),
                    _ => unreachable!()
                };

                (_int digit_nonzero) => |v| match (&v[0], &v[1]) {
                    (ParseValue::String(int), ParseValue::Token(digit)) => ParseValue::String(format!("{int}{digit}")),
                    _ => unreachable!()
                };

                (_int "0") => |v| match &v[0] {
                    ParseValue::String(int) => ParseValue::String(format!("{int}0")),
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
