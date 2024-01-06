use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
};

use super::input::Input;
use super::result::*;

pub type Transformer = Box<dyn Fn(&Vec<ParseValue>) -> ParseValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulePart {
    Term(String),
    NonTerm(String),
    Recurse,
    Not(HashSet<String>),
}

impl RulePart {
    fn parse<'a>(
        &self,
        rules: &Rules,
        current_rule: &str,
        input: Input<'a>,
        nexts: &[RuleTree],
        buffer: &Vec<ParseValue>,
        recursive: bool,
    ) -> ParseResult<'a> {
        match self {
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

                let parse_value = Rc::new(Token::from(literal.clone()));

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
            RulePart::Not(literals) => {
                let step = literals
                    .iter()
                    .map(|str| str.chars().count())
                    .min()
                    .unwrap();

                let mut start_input = input.clone();

                for literal in literals {
                    let mut literal_chars = literal.chars();

                    let mut input = start_input.clone();

                    loop {
                        let mut i = input.clone();

                        let (row, col) = i.row_col();

                        let (i_char, l_char) = (i.next(), literal_chars.next());
                        match (i_char, l_char) {
                            (None, None) | (Some(_), None) => {
                                return Err(ParseError::UnexpectedToken {
                                    current_rule: current_rule.to_owned(),
                                    token: literal.clone(),
                                    pos: start_input.pos(),
                                    row,
                                    col,
                                })
                            }
                            (None, Some(_)) => break,
                            (Some(c0), Some(c1)) if c0 == c1 => {}
                            (Some(_), Some(_)) => break,
                        }

                        input = i;
                    }
                }

                let mut token = String::new();

                for _ in 0..step {
                    let pos = start_input.pos();
                    let (row, col) = start_input.row_col();
                    token.push(
                        start_input
                            .next()
                            .ok_or_else(|| ParseError::UnexpectedChar {
                                current_rule: current_rule.to_owned(),
                                char: None,
                                pos,
                                row,
                                col,
                            })?,
                    );
                }

                let parse_value = Rc::new(Token::from(token));

                let mut buffer = buffer.clone();
                buffer.push(parse_value);

                rules.parse_rule_trees(current_rule, nexts, input, buffer, recursive)
            }
        }
    }
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
            RuleTree::Part { part, nexts } => {
                part.parse(rules, current_rule, input, nexts, buffer, recursive)
            }
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

    fn add_namespace(self, namespace: &str) -> RuleTree {
        match self {
            RuleTree::Part { part, nexts } => {
                let part = match part {
                    RulePart::NonTerm(rule) => RulePart::NonTerm(format!("{namespace}::{rule}")),
                    part => part,
                };

                RuleTree::Part {
                    part,
                    nexts: nexts
                        .into_iter()
                        .map(|tree| tree.add_namespace(namespace))
                        .collect(),
                }
            }
            tree => tree,
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

    #[allow(dead_code)]
    pub(crate) fn rule_names(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    // Adds the given Rules to this one, optionally adding a namespace
    pub fn import(&mut self, other: Rules, name: Option<String>) {
        for (rule_name, rule_trees) in other.0.into_iter() {
            let rule_name = if let Some(namespace) = &name {
                format!("{namespace}::{rule_name}")
            } else {
                rule_name
            };

            match self.0.entry(rule_name) {
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    let rule_trees = if let Some(namespace) = &name {
                        rule_trees
                            .into_iter()
                            .map(|tree| tree.add_namespace(namespace))
                            .collect::<Vec<_>>()
                    } else {
                        rule_trees
                    };

                    o.get_mut().extend(rule_trees);

                    *o.get_mut() = Self::smush(std::mem::take(o.get_mut()));
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    let rule_trees = if let Some(namespace) = &name {
                        rule_trees
                            .into_iter()
                            .map(|tree| tree.add_namespace(namespace))
                            .collect::<Vec<_>>()
                    } else {
                        rule_trees
                    };

                    v.insert(rule_trees);
                }
            }
        }
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

    pub fn parse_proc<'a>(
        &self,
        start_rule: &str,
        input: impl Into<Input<'a>>,
    ) -> Result<ParseValue, ParseError> {
        parse(self, start_rule, input.into()).map(|x| x.0)
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
                    if let RulePart::Not(literals) = &part {
                        for t in &mut trees {
                            if let RuleTree::Part {
                                part: RulePart::Not(ls),
                                nexts: n,
                            } = t
                            {
                                ls.extend(literals.iter().cloned());

                                n.extend(nexts);

                                let nexts = std::mem::take(n);
                                *n = Self::smush(nexts);

                                return trees;
                            }
                        }
                    } else {
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
                (RulePart::Recurse, RulePart::Recurse) => Ordering::Equal,
                (RulePart::Recurse, _) => Ordering::Greater,
                (_, RulePart::Recurse) => Ordering::Less,
                (RulePart::Term(lit0), RulePart::Term(lit1)) => {
                    lit0.chars().count().cmp(&lit1.chars().count())
                }
                (RulePart::Term(_), RulePart::NonTerm(_)) => Ordering::Greater,
                (RulePart::Term(_), RulePart::Not(_)) => Ordering::Greater,
                (RulePart::NonTerm(_), RulePart::Term(_)) => Ordering::Less,
                (RulePart::NonTerm(_), RulePart::NonTerm(_)) => Ordering::Equal,
                (RulePart::NonTerm(_), RulePart::Not(_)) => Ordering::Greater,
                // Theoretically, all Nots in the same level are merged, so this is unreachable?
                (RulePart::Not(_), RulePart::Not(_)) => unreachable!(),
                (RulePart::Not(_), _) => Ordering::Less,
            },
            _ => std::cmp::Ordering::Equal,
        });

        v
    }
}

#[derive(Clone)]

struct ParseStackItem<'a, 'i> {
    depth: usize,
    // The current rule
    rule: &'a str,
    rule_trees: &'a [RuleTree],
    n: usize,
    input: Input<'i>,

    // primarily for debugging
    prev_path: Vec<usize>,
}

impl<'a, 'i> Debug for ParseStackItem<'a, 'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ParseStackItem {
            depth,
            rule,
            n,
            input,
            prev_path,
            ..
        } = self;

        f.write_fmt(format_args!(
            "({depth}, {rule}{prev_path:?}, [{n:?}], input = \"{input}\")"
        ))
    }
}

#[inline]
fn parse<'a>(
    rules: &Rules,
    rule: &str,
    input: Input<'a>,
) -> Result<(ParseValue, Input<'a>), ParseError> {
    let rule_trees = rules.0.get(rule).ok_or_else(|| ParseError::RuleNotFound {
        rule_name: rule.to_owned(),
    })?;
    let mut stack = vec![ParseStackItem {
        depth: 0,
        rule,
        rule_trees,
        n: 0,
        prev_path: vec![],
        input,
    }];

    let mut buffers: Vec<Vec<ParseValue>> = vec![vec![]];

    'main: loop {
        let top = stack.last().unwrap().clone();

        log::debug!("\n\n\n\n");
        log::debug!("STACK: {stack:#?}");
        log::debug!("BUFFERS: {buffers:?}");

        let rule_tree = &top.rule_trees[top.n];

        let token = match rule_tree {
            RuleTree::Part { part, nexts } => match part {
                RulePart::Term(literal) => {
                    log::debug!("LEX \"{literal}\"");

                    let mut literal_chars = literal.chars();

                    let mut term_input = top.input.clone();

                    let success = loop {
                        let mut i = term_input.clone();

                        let pos = i.pos();
                        let (row, col) = i.row_col();

                        let (i_char, l_char) = (i.next(), literal_chars.next());
                        match (i_char, l_char) {
                            (None, None) => break Ok(()),
                            (None, Some(_)) => {
                                break Err(ParseError::UnexpectedChar {
                                    current_rule: top.rule.to_owned(),
                                    char: None,
                                    pos,
                                    row,
                                    col,
                                })
                            }
                            (Some(_), None) => break Ok(()),
                            (Some(c0), Some(c1)) if c0 == c1 => {}
                            (Some(c), Some(_)) => {
                                break Err(ParseError::UnexpectedChar {
                                    current_rule: top.rule.to_owned(),
                                    char: Some(c),
                                    pos,
                                    row,
                                    col,
                                })
                            }
                        }

                        term_input = i;
                    };

                    success.map(|_| (Token::from(literal), term_input, nexts))
                }
                RulePart::Not(literals) => {
                    log::debug!(
                        "LEX {:?}",
                        literals
                            .iter()
                            .map(|s| format!("\"{s}\""))
                            .collect::<Vec<_>>()
                    );

                    let mut success = Ok(());

                    'literals: for literal in literals {
                        let mut literal_chars = literal.chars();
                        let mut term_input = top.input.clone();

                        loop {
                            let mut i = term_input.clone();

                            let pos = i.pos();
                            let (row, col) = i.row_col();

                            let (i_char, l_char) = (i.next(), literal_chars.next());
                            match (i_char, l_char) {
                                (None, None) => {
                                    success = Err(ParseError::UnexpectedToken {
                                        current_rule: top.rule.to_owned(),
                                        token: literal.clone(),
                                        pos,
                                        row,
                                        col,
                                    });
                                    break 'literals;
                                }
                                (None, Some(_)) => {
                                    break;
                                }
                                (Some(_), None) => {
                                    success = Err(ParseError::UnexpectedToken {
                                        current_rule: top.rule.to_owned(),
                                        token: literal.clone(),
                                        pos,
                                        row,
                                        col,
                                    });
                                    break 'literals;
                                }
                                (Some(c0), Some(c1)) if c0 == c1 => {}
                                (Some(_), Some(_)) => {
                                    break;
                                }
                            }

                            term_input = i;
                        }
                    }

                    let step_size = literals.iter().map(|s| s.chars().count()).min().unwrap();

                    let mut input = top.input.clone();
                    let mut token = String::new();

                    for _ in 0..step_size {
                        let pos = input.pos();
                        let (row, col) = input.row_col();

                        if let Some(ch) = input.next() {
                            token.push(ch);
                        } else {
                            success = Err(ParseError::UnexpectedChar {
                                current_rule: top.rule.to_owned(),
                                char: None,
                                pos,
                                row,
                                col,
                            });
                            break;
                        }
                    }

                    success.map(|_| (Token::from(token), input, nexts))
                }

                RulePart::NonTerm(rule) => {
                    log::debug!("PUSH RULE {rule} ONTO STACK");

                    stack.push(ParseStackItem {
                        depth: top.depth + 1,
                        rule,
                        rule_trees: rules.0.get(rule).ok_or_else(|| ParseError::RuleNotFound {
                            rule_name: rule.clone(),
                        })?,
                        n: 0,
                        input: top.input,
                        prev_path: vec![],
                    });

                    buffers.push(Vec::new());

                    continue 'main;
                }

                RulePart::Recurse => todo!(),
            },
            RuleTree::End { transformer } => {
                log::debug!("END");

                let mut buffer = buffers.pop().unwrap();

                let parse_value = if let Some(transformer) = transformer {
                    transformer(&buffer)
                } else if buffer.len() == 1 {
                    buffer.remove(0)
                } else {
                    buffer.into_value()
                };

                if top.depth == 0 {
                    // End on depth == 0, return value
                    log::debug!("END ON DEPTH 0, FINISHED!");

                    return Ok((parse_value, top.input));
                } else {
                    log::debug!("REMOVE DEPTH {}", top.depth);
                    'depth: loop {
                        if let Some(&ParseStackItem { depth: d, .. }) = stack.last() {
                            if d == top.depth {
                                stack.pop();
                            } else {
                                break 'depth;
                            }
                        } else {
                            break 'depth;
                        }
                    }

                    let mut new_top = stack.last().unwrap().clone();

                    match &new_top.rule_trees[new_top.n] {
                        RuleTree::Part {
                            part: RulePart::NonTerm(_),
                            nexts,
                        } => {
                            new_top.prev_path.push(new_top.n);
                            new_top.n = 0;
                            new_top.input = top.input;
                            new_top.rule_trees = nexts;
                            stack.push(new_top);

                            let buffer = if buffers.is_empty() {
                                buffers.push(vec![]);
                                &mut buffers[0]
                            } else {
                                buffers.last_mut().unwrap()
                            };

                            buffer.push(parse_value);
                        }

                        RuleTree::Part { part, .. } => {
                            log::debug!("NON NONTERM PART: {part:?}");
                            unreachable!()
                        }

                        _ => unreachable!(),
                    }
                }

                continue 'main;
            }
        };

        match token {
            Ok((token, input, nexts)) => {
                log::debug!("LEX SUCCESS, PUSH NEXT ONTO STACK");

                let mut prev_path = top.prev_path;

                prev_path.push(top.n);

                stack.push(ParseStackItem {
                    depth: top.depth,
                    rule: top.rule,
                    rule_trees: nexts,
                    n: 0,
                    input,
                    prev_path,
                });

                buffers.last_mut().unwrap().push(token.into_value());
            }
            Err(error) => fail(rules, &mut stack, &mut buffers, error)?,
        }
    }
}

#[inline]
fn fail(
    rules: &Rules,
    stack: &mut Vec<ParseStackItem>,
    buffers: &mut Vec<Vec<ParseValue>>,
    error: ParseError,
) -> Result<(), ParseError> {
    //let errors = vec![error];

    // ab: ("a" "b")
    // "ac"
    // [0, ab, 0, "ab"]      [[]]
    // [0, ab[0], 0, "b"]    [["a"]]
    // inc n, can't inc, delete top, delete last item in last buffer
    // inc n, can't inc, delete top, delete last item in last buffer
    //

    log::debug!("ENTER FAIL");

    'fail: loop {
        log::debug!("\n\n\n\n");
        log::debug!("FAIL LOOP START");
        log::debug!("STACK: {stack:#?}");
        log::debug!("BUFFERS: {buffers:?}");

        // if stack.is_empty() {
        //     return Err(error)
        // }

        let top = stack.last_mut().unwrap();

        log::debug!(
            "top.n = {}, top.rule_trees.len() = {}",
            top.n,
            top.rule_trees.len()
        );

        if top.n + 1 < top.rule_trees.len() {
            top.n += 1;
            break 'fail;
        } else {
            let old_top = stack.pop().unwrap();

            if let Some(top) = stack.last_mut() {
                if top.depth == old_top.depth {
                    buffers.last_mut().unwrap().pop();
                } else {
                    buffers.pop();
                }
            } else {
                return Err(error);
                // return Err(ParseError::MultipleErrors {
                //     current_rule: old_top.rule.to_owned(),
                //     errors,
                // });
            }
        }
    }

    Ok(())
}
