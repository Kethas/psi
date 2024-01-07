use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
};

use super::input::Input;
use super::result::*;

pub type ParseBuffer<'a> = &'a mut dyn FnMut(usize) -> ParseValue;

pub type Transformer = Rc<dyn Fn(ParseBuffer) -> ParseValue>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulePart {
    Term(String),
    NonTerm(String),
    Recurse,
    Not(HashSet<String>),
}

#[derive(Clone)]
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
    fn length(&self) -> usize {
        match self {
            RuleTree::Part { nexts, .. } => {
                1 + nexts.iter().map(|tree| tree.length()).max().unwrap_or(0)
            }
            RuleTree::End { .. } => 1,
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

#[derive(Clone)]
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

#[derive(Clone)]
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
        parse(self, start_rule, input.into()).and_then(|(value, mut input)| {
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
        })
    }

    pub fn parse<'a>(
        &self,
        start_rule: &str,
        input: impl Into<Input<'a>>,
    ) -> Result<ParseValue, ParseError> {
        parse(self, start_rule, input.into()).map(|x| x.0)
    }

    pub fn parse_proc<'a>(
        &self,
        start_rule: &str,
        input: impl Into<Input<'a>>,
    ) -> Result<ParseValue, ParseError> {
        parse(self, start_rule, input.into()).map(|x| x.0)
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
                (RulePart::Recurse, RulePart::Recurse) => unreachable!(),
                (RulePart::Recurse, _) => Ordering::Greater,
                (_, RulePart::Recurse) => Ordering::Less,
                (RulePart::Term(lit0), RulePart::Term(lit1)) => {
                    match lit0.chars().count().cmp(&lit1.chars().count()) {
                        Ordering::Equal => b.length().cmp(&a.length()),
                        ord => ord,
                    }
                }
                (RulePart::Term(_), RulePart::NonTerm(_)) => Ordering::Greater,
                (RulePart::Term(_), RulePart::Not(_)) => Ordering::Greater,
                (RulePart::NonTerm(_), RulePart::Term(_)) => Ordering::Less,
                (RulePart::NonTerm(_), RulePart::NonTerm(_)) => b.length().cmp(&a.length()),
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

        let prev_path = if prev_path.is_empty() {
            "".to_owned()
        } else {
            format!("{prev_path:?}")
        };

        f.write_fmt(format_args!(
            "({depth}, {rule}{prev_path}, [{n:?}], input = \"{input}\")"
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

                RulePart::Recurse => {
                    log::debug!("RECURSE");

                    stack.push(ParseStackItem {
                        depth: top.depth + 1,
                        rule,
                        rule_trees: rules.0.get(top.rule).ok_or_else(|| {
                            ParseError::RuleNotFound {
                                rule_name: top.rule.to_owned(),
                            }
                        })?,
                        n: 1,
                        input: top.input,
                        prev_path: vec![],
                    });

                    buffers.push(Vec::new());

                    continue 'main;
                }
            },
            RuleTree::End { transformer } => {
                log::debug!("END");

                let mut buffer = buffers.pop().unwrap();

                let parse_value = if let Some(transformer) = transformer {
                    let mut buffer = buffer.into_iter().map(Some).collect::<Vec<_>>();

                    let mut parse_buffer = move |index| {
                        let value: &mut Option<ParseValue> = &mut buffer[index];
                        value.take().unwrap()
                    };

                    transformer(&mut parse_buffer)
                } else if buffer.len() == 1 {
                    buffer.remove(0)
                } else {
                    buffer.into_value()
                };

                if let Some(res) = end(&mut stack, &mut buffers, parse_value, None) {
                    return Ok(res);
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
            Err(error) => {
                if let Some(result) = fail(&mut stack, &mut buffers, error)? {
                    return Ok(result);
                }
            }
        }
    }
}

#[inline]
fn fail<'a>(
    stack: &mut Vec<ParseStackItem<'_, 'a>>,
    buffers: &mut Vec<Vec<ParseValue>>,
    error: ParseError,
) -> Result<Option<(ParseValue, Input<'a>)>, ParseError> {
    //let errors = vec![error];

    // ab: ("a" "b")
    // "ac"
    // [0, ab, 0, "ab"]      [[]]
    // [0, ab[0], 0, "b"]    [["a"]]
    // inc n, can't inc, delete top, delete last item in last buffer
    // inc n, can't inc, delete top, delete last item in last buffer
    //

    log::debug!("ENTER FAIL");

    let mut last_buffer: Option<Vec<ParseValue>> = None;
    let mut last_input = None;

    'fail: loop {
        log::debug!("\n\n\n\n");
        log::debug!("FAIL LOOP START");
        log::debug!("STACK: {stack:#?}");
        log::debug!("BUFFERS: {buffers:?}");

        // if stack.is_empty() {
        //     return Err(error)
        // }

        let top = stack.last_mut().unwrap();

        if let RuleTree::Part {
            part: RulePart::Recurse,
            ..
        } = &top.rule_trees[top.n]
        {
            log::debug!("Reached Recurse on fail");
            let parse_value = last_buffer.unwrap().remove(0);

            buffers.pop();

            if let Some(res) = end(stack, buffers, parse_value, last_input) {
                return Ok(Some(res));
            }

            break 'fail;
        }

        log::debug!(
            "top.n = {}, top.rule_trees.len() = {}",
            top.n,
            top.rule_trees.len()
        );

        last_input = Some(top.input.clone());

        if top.n + 1 < top.rule_trees.len() {
            top.n += 1;
            break 'fail;
        } else {
            let old_top = stack.pop().unwrap();

            if let Some(top) = stack.last_mut() {
                if top.depth == old_top.depth {
                    last_buffer = buffers.last_mut().unwrap().pop().map(|v| vec![v]);
                } else {
                    last_buffer = buffers.pop();
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

    Ok(None)
}

fn end<'a>(
    stack: &mut Vec<ParseStackItem<'_, 'a>>,
    buffers: &mut Vec<Vec<ParseValue>>,
    parse_value: ParseValue,
    override_input: Option<Input<'a>>,
) -> Option<(ParseValue, Input<'a>)> {
    let top = stack.last().unwrap().clone();

    let input = override_input.unwrap_or_else(|| top.input.clone());

    if top.depth == 0 {
        // End on depth == 0, return value
        log::debug!("END ON DEPTH 0, FINISHED!");

        return Some((parse_value, input));
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
                new_top.input = input;
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

            RuleTree::Part {
                part: RulePart::Recurse,
                nexts,
            } => {
                new_top.depth += 1;
                new_top.prev_path.push(new_top.n);
                new_top.n = 0;
                new_top.input = input;
                new_top.rule_trees = nexts;
                stack.push(new_top);

                let buffer = if buffers.is_empty() {
                    unreachable!();
                } else {
                    buffers.insert(buffers.len() - 1, Vec::new());
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

    None
}
