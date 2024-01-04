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
}

impl<'a, 'i> Debug for ParseStackItem<'a, 'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ParseStackItem {
            depth,
            rule,
            n,
            input,
            ..
        } = self;

        f.write_fmt(format_args!(
            "({depth}, \"{rule}\", [{n:?}], input = \"{input}\")"
        ))
    }
}

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
        input,
    }];

    let mut buffers: Vec<Vec<ParseValue>> = vec![vec![]];

    loop {
        if stack.is_empty() {
            break;
        }

        log::debug!("\n\n\n\n");
        log::debug!("STACK: {stack:#?}");
        log::debug!("BUFFERS: {buffers:?}");

        let ParseStackItem {
            depth,
            rule,
            rule_trees,
            n,
            mut input,
        } = stack.pop().unwrap();

        let rule_tree = &rule_trees[n];

        match rule_tree {
            RuleTree::Part { part, nexts } => {
                // parse...
                let start_input = input.clone();

                let success = match part {
                    RulePart::Term(literal) => {
                        log::debug!("TERM: \"{literal}\"");

                        let mut literal_chars = literal.chars();

                        let mut term_input = input.clone();

                        let success = loop {
                            let mut i = term_input.clone();

                            let (i_char, l_char) = (i.next(), literal_chars.next());
                            match (i_char, l_char) {
                                (None, None) => break true,
                                (None, Some(_)) => break false,
                                (Some(_), None) => break true,
                                (Some(c0), Some(c1)) if c0 == c1 => {}
                                (Some(_), Some(_)) => break false,
                            }

                            term_input = i;
                        };

                        if success {
                            let parse_value = Rc::new(Token::from(literal.clone()));

                            let buffer = buffers.last_mut().unwrap();
                            buffer.push(parse_value);

                            input = term_input;

                            true
                        } else {
                            false
                        }
                    }
                    RulePart::NonTerm(rule_name) => {
                        log::debug!("NON_TERM: ({rule_name})");
                        // push this rule back on the stack
                        stack.push(ParseStackItem {
                            depth,
                            rule,
                            rule_trees,
                            n,
                            input: start_input,
                        });

                        let rule_trees =
                            rules
                                .0
                                .get(rule_name)
                                .ok_or_else(|| ParseError::RuleNotFound {
                                    rule_name: rule_name.clone(),
                                })?;

                        let new_stack_item = ParseStackItem {
                            depth: depth + 1,
                            rule: rule_name,
                            rule_trees,
                            n: 0,
                            input: input.clone(),
                        };

                        stack.push(new_stack_item);

                        buffers.push(vec![]);

                        continue;
                    }
                    RulePart::Recurse => {
                        todo!()
                    }
                    RulePart::Not(_) => todo!(),
                };

                if success {
                    log::debug!("TERM SUCCESS");

                    stack.push(ParseStackItem {
                        depth,
                        rule,
                        rule_trees: nexts,
                        n: 0,
                        input,
                    });
                } else {
                    log::debug!("TERM FAIL!");

                    let mut n = n;
                    let mut rule_trees = rule_trees;
                    let mut depth = depth;
                    let mut input = start_input.clone();

                    'fail: loop {
                        log::debug!("(fail loop start)");
                        if n + 1 < rule_trees.len() {
                            // increment path

                            stack.push(ParseStackItem {
                                depth,
                                rule,
                                rule_trees,
                                n: n + 1,
                                input,
                            });
                            break 'fail;
                        } else {
                            log::debug!("REMOVING DEPTH {depth}");
                            'depth: loop {
                                if let Some(&ParseStackItem { depth: d, .. }) = stack.last() {
                                    if d == depth {
                                        stack.pop();
                                    } else {
                                        break 'depth;
                                    }
                                } else {
                                    break 'depth;
                                }
                            }

                            log::debug!("INTERMEDIATE_STACK: {stack:#?}");
                            log::debug!("INTERMEDIATE_BUFFERS: {buffers:?}");

                            if let Some(top) = stack.last().cloned() {
                                n = top.n;
                                rule_trees = top.rule_trees;
                                depth = top.depth;
                                input = top.input;
                            } else {
                                return Err(ParseError::MultipleErrors {
                                    current_rule: "?".to_owned(),
                                    errors: vec![],
                                });
                            }
                        }
                    }
                }
            }
            RuleTree::End { transformer } => {
                log::debug!("Reached end!");

                'depth: loop {
                    if let Some(&ParseStackItem { depth: d, .. }) = stack.last() {
                        if d == depth {
                            stack.pop();
                        } else {
                            break 'depth;
                        }
                    } else {
                        break 'depth;
                    }
                }

                let buffer = buffers.pop().unwrap();

                let parse_value = if let Some(transformer) = transformer {
                    transformer(&buffer)
                } else if buffer.len() == 1 {
                    buffer[0].clone()
                } else {
                    Rc::new(buffer)
                };

                if let Some(buffer) = buffers.last_mut() {
                    buffer.push(parse_value.clone());
                } else {
                    buffers.push(vec![parse_value.clone()]);
                }

                if let Some(stack_top) = stack.last().cloned() {
                    let nexts = match &stack_top.rule_trees[stack_top.n] {
                        RuleTree::Part {
                            part: RulePart::NonTerm(_),
                            nexts,
                        } => nexts,

                        _ => unreachable!(),
                    };

                    stack.push(ParseStackItem {
                        depth: stack_top.depth,
                        rule: stack_top.rule,
                        rule_trees: nexts,
                        n: 0,
                        input,
                    })
                } else {
                    log::debug!("The very end!");
                    // the very end!
                    return Ok((parse_value, input));
                }
            }
        }
    }

    unreachable!()
}
