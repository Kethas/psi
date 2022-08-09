use crate::parse::parsed::*;
use crate::utils::*;
use lazy_static::lazy_static;
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
    sync::Arc,
    vec,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RulePart {
    Empty, // this can be removed, if verified it is not needed, ie can be replaced by [] or nothing
    Literal(String),
    Rule(String),
}

#[dyn_clonable::clonable]
pub trait RuleTransformer: Clone + Fn(ParseTree) -> ParseObject {}

impl<T> RuleTransformer for T where T: Clone + Fn(ParseTree) -> ParseObject {}

#[derive(Clone)]
pub struct RuleAction {
    pub inner: Arc<dyn RuleTransformer>,
    pub id: Uuid,
}

impl Default for RuleAction {
    fn default() -> Self {
        Self {
            inner: Arc::new(|x| ParseObject::ParseTree(x)),
            id: Uuid::nil(),
        }
    }
}

impl Debug for RuleAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RuleAction@{}", self.id))
    }
}

impl PartialEq for RuleAction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for RuleAction {}

impl RuleAction {
    pub fn apply(&self, input: ParseTree) -> ParseObject {
        (self.inner)(input)
    }
}

//TODO: maybe flatten this into just a vec, as it seems unecessary at this point to have
// attributes for a single definition when a group of defitnitions can be a group of one
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RuleDef {
    pub parts: Vec<RulePart>,
    pub action: RuleAction,
}

impl RuleDef {
    pub fn from_parts(parts: Vec<RulePart>) -> Self {
        Self {
            parts,
            action: RuleAction::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Associativity {
    #[default]
    Left,
    Right,
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleEntry {
    pub definitions: Vec<RuleDef>,
    pub precedence: u64,
    pub associativity: Associativity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    rules: HashMap<String, Vec<RuleEntry>>,
}

/*

analyse cycles in the rules
    cyclic rules are infinite size, therefore zero priority
    uncyclic rules are finite and can have a size (priority) calculated
    literals have a priority equal to their size
    empty is the lowest priority

TODO: proper priotisation
*/

impl Rules {
    pub fn new(rules: impl Into<HashMap<String, Vec<RuleEntry>>>) -> Self {
        Self {
            rules: rules.into(),
        }
    }

    pub(crate) fn flattened(self) -> FlattenedRules {
        let mut rules: HashMap<String, HashMap<u64, Vec<RuleEntry>>> = HashMap::new();

        for (name, rule_entries) in self.rules.clone() {
            let precedences = match rules.get_mut(&name) {
                Some(precedences) => precedences,
                None => {
                    rules.insert(name.clone(), HashMap::new());
                    rules.get_mut(&name).unwrap()
                }
            };

            for rule_entry in rule_entries {
                let precedence = match precedences.get_mut(&rule_entry.precedence) {
                    Some(precedence) => precedence,
                    None => {
                        precedences.insert(rule_entry.precedence, Vec::new());
                        precedences.get_mut(&rule_entry.precedence).unwrap()
                    }
                };

                precedence.push(rule_entry)
            }
        }

        fn get_next_prec(prec: u64, iter: impl IntoIterator<Item = u64>) -> u64 {
            let mut v = iter.into_iter().filter(|p| p < &prec).collect::<Vec<_>>();
            v.sort();

            v.pop().unwrap_or(0)
        }

        let mut out: HashMap<String, Vec<Vec<RulePart>>> = HashMap::new();

        for (name, precedences) in rules {
            if precedences.len() == 0 {
                continue;
            } else if precedences.len() == 1 {
                let v = match out.get_mut(&name) {
                    Some(v) => v,
                    None => {
                        out.insert(name.clone(), Vec::new());
                        out.get_mut(&name).unwrap()
                    }
                };

                for (_, entries) in precedences {
                    for entry in entries {
                        for rule in entry.definitions {
                            v.push(rule.parts);
                        }
                    }
                }

                continue;
            }

            // rule: rule_max
            let highest_precedence = precedences.keys().copied().fold(0, u64::max);
            let v = match out.get_mut(&name) {
                Some(v) => v,
                None => {
                    out.insert(name.clone(), Vec::new());
                    out.get_mut(&name).unwrap()
                }
            };
            v.push(vec![
                RulePart::Rule(format!("{name}@{highest_precedence}")),
                RulePart::Empty,
            ]);

            // left assoc
            // rule_x: rule_x-1 rule_x...
            // right assoc would be `rule_x: rule_x... rule_x-1` but i haveent implemented left recursion yet
            // nonassoc would be `rule_x: rule_x-1...`
            let all_precedences = precedences.keys().copied().collect::<Vec<_>>();
            for (precedence, entries) in precedences {
                let prec_name = format!("{name}@{precedence}");
                let next_prec = format!(
                    "{name}@{}",
                    get_next_prec(precedence, all_precedences.clone())
                );

                let v = match out.get_mut(&prec_name) {
                    Some(v) => v,
                    None => {
                        out.insert(prec_name.clone(), Vec::new());
                        out.get_mut(&prec_name).unwrap()
                    }
                };

                for entry in entries {
                    let assoc = entry.associativity;

                    for def in entry.definitions {
                        if def.parts.is_empty() {
                            v.push(vec![RulePart::Empty]);
                            continue;
                        }

                        let len = def.parts.len();

                        v.push(
                            def.parts
                                .into_iter()
                                .enumerate()
                                .map(assoc_applier(len, assoc, name.clone(), prec_name.clone(), next_prec.clone()))
                                .collect(),
                        )
                    }
                }
            }
        }

        FlattenedRules::from_rules(out)
    }

    pub fn into_grammar(self) -> Grammar {
        self.flattened().into_grammar()
    }
}

// TODO: actually test that the implementation is correct and not a brain fart - maybe l and r are switched
fn assoc_applier(
    len: usize,
    assoc: Associativity,
    name: String,
    prec_name: String,
    next_prec: String,
) -> Box<dyn Fn((usize, RulePart)) -> RulePart> {
    Box::new(move |(n, rule_part)| match assoc {
        // rule_x: rule_x-1 rule_x...
        Associativity::Left if n == 0 => match rule_part {
            RulePart::Rule(r) if r == name => RulePart::Rule(next_prec.clone()),
            x => x,
        },
        Associativity::Left => match rule_part { 
            RulePart::Rule(r) if r == name => RulePart::Rule(prec_name.clone()),
            x => x,
        },

        // rule_x: rule_x... rule_x-1
        Associativity::Right if n == len => match rule_part {
            RulePart::Rule(r) if r == name => RulePart::Rule(next_prec.clone()),
            x => x,
        },
        Associativity::Right => match rule_part {
            RulePart::Rule(r) if r == name => RulePart::Rule(prec_name.clone()),
            x => x,
        },
        
        // rule_x: rule_x-1...
        Associativity::None => match rule_part {
            RulePart::Rule(r) if r == name => RulePart::Rule(next_prec.clone()),
            x => x,
        },
    })
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum RuleType {
    Cyclic,
    Acyclic(usize),
}
use uuid::Uuid;
use RuleType::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FlattenedRules {
    rules: HashMap<String, Vec<Vec<RulePart>>>,
    rule_types: HashMap<String, RuleType>,
}

impl FlattenedRules {
    pub fn new(
        rules: impl Into<HashMap<String, Vec<Vec<RulePart>>>>,
        rule_types: impl Into<HashMap<String, RuleType>>,
    ) -> Self {
        Self {
            rules: rules.into(),
            rule_types: rule_types.into(),
        }
    }

    pub fn from_rules(rules: impl Into<HashMap<String, Vec<Vec<RulePart>>>>) -> Self {
        let mut rules = Self::new(rules, []);
        rules.analyse_cycles();
        rules.compute_sizes();
        rules
    }

    pub fn analyse_cycles(&mut self) {
        fn analyse_cycle(
            all_rules: &HashMap<String, Vec<Vec<RulePart>>>,
            current: &str,
            name: &str,
            history: &mut Vec<String>,
        ) -> bool {
            history.push(current.to_owned());

            for rules in all_rules.get(current) {
                for rule in rules {
                    for rule_part in rule {
                        match rule_part {
                            RulePart::Rule(n) if n == name => return true,
                            RulePart::Rule(n) if !history.contains(n) => {
                                if analyse_cycle(all_rules, n, name, history) {
                                    return true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            false
        }

        for (rule_name, rules) in &self.rules {
            let cyclic = analyse_cycle(&self.rules, rule_name, rule_name, &mut Vec::new());

            self.rule_types.insert(
                rule_name.to_owned(),
                match cyclic {
                    true => Cyclic,
                    false => Acyclic(usize::MAX),
                },
            );
        }
    }

    // if a child is zero, then it is zero, otherwise it is the highest child
    // a rule is the sum of its parts
    pub fn compute_sizes(&mut self) {
        let mut last_state = self.rule_types.values().copied().collect::<Vec<_>>();

        loop {
            for rule in self.rule_types.keys().cloned().collect::<Vec<_>>() {
                match self.rule_types.get(&rule).copied() {
                    Some(Acyclic(usize::MAX)) => {
                        if let Some(size) = self.compute_size(&rule) {
                            self.rule_types.insert(rule.clone(), Acyclic(size));
                        }
                    }
                    None => unreachable!(),
                    _ => {}
                }
            }

            let state = self.rule_types.values().copied().collect::<Vec<_>>();

            // if there was no change, there will be no further changes -- the system is in deadlock
            if last_state == state {
                break;
            }

            last_state = state;
        }

        for (name, ty) in &mut self.rule_types {
            match ty {
                Acyclic(x) if *x == usize::MAX => {
                    println!("{name} is unset acyclic. resetting to zero.");
                    *x = 0;
                }
                _ => {}
            }
        }
    }

    // if a child is zero, then it is zero, otherwise it is the highest child
    // a rule is the sum of its parts
    // base is guaranteed to be Acyclic
    fn compute_size(&mut self, base: &str) -> Option<usize> {
        let mut largest_child = 0;

        for rules in self.rules.get(base) {
            for rule in rules {
                let mut rule_size = 0;
                for part in rule {
                    match part {
                        RulePart::Empty => {}
                        RulePart::Literal(lit) => rule_size += lit.chars().count(),
                        RulePart::Rule(name) => {
                            let size = self.get_rules_priority(name)?;
                            if size == usize::MAX {
                                return None;
                            }
                            rule_size += size
                        }
                    }
                }

                if rule_size > largest_child {
                    largest_child = rule_size;
                }
            }
        }

        Some(largest_child)
    }

    pub fn get_rules_priority(&self, name: &str) -> Option<usize> {
        self.rule_types.get(name).map(|x| match x {
            Cyclic => 0,
            Acyclic(x) => *x,
        })
    }

    pub fn get_rule_part_priority(&self, rule_part: &RulePart) -> Option<usize> {
        match rule_part {
            RulePart::Empty => Some(0),
            RulePart::Literal(lit) => Some(lit.chars().count()),
            RulePart::Rule(name) => self.get_rules_priority(name),
        }
    }

    pub fn get_rule_priority(&self, rule: &[RulePart]) -> Option<usize> {
        let mut size = 0;

        for part in rule {
            size += self.get_rule_part_priority(part)?;
        }

        Some(size)
    }

    pub fn cmp_rule(&self, r1: &[RulePart], r2: &[RulePart]) -> Option<Ordering> {
        let a = self.get_rule_priority(r1)?;
        let b = self.get_rule_priority(r2)?;

        a.partial_cmp(&b)
    }

    pub fn cmp_rule_part(&self, rp1: &RulePart, rp2: &RulePart) -> Option<Ordering> {
        match (rp1, rp2) {
            // for two literals compare their priority
            (rp1 @ RulePart::Literal(_), rp2 @ RulePart::Literal(_)) => Some(
                self.get_rule_part_priority(rp1)?
                    .cmp(&self.get_rule_part_priority(rp2)?),
            ),
            // a literal is more important than a rule or an empty
            (rp1 @ RulePart::Literal(_), rp2) => Some(Ordering::Greater),

            // a rule and another rule: compare their priorities
            (RulePart::Rule(r1), RulePart::Rule(r2)) => Some(
                self.get_rules_priority(r1)?
                    .cmp(&self.get_rules_priority(r2)?),
            ),

            // a rule is higher than an empty
            (RulePart::Rule(_), RulePart::Empty) => Some(Ordering::Greater),

            // an empty is equal to an empty
            (RulePart::Empty, RulePart::Empty) => Some(Ordering::Equal),

            // if no thingy yet then  a `cmp` b = b `cmp` a
            (a, b) => self.cmp_rule_part(b, a),
        }
    }

    pub fn cmp_rule_tree(&self, rt1: &RuleTree, rt2: &RuleTree) -> Option<Ordering> {
        match (rt1, rt2) {
            // for two literals compare their length
            (RuleTree::Lit(lit1, ..), RuleTree::Lit(lit2, ..)) => {
                Some(lit1.chars().count().cmp(&lit2.chars().count()))
            }
            // a literal is more important than a rule or an end
            (RuleTree::Lit(..), _) => Some(Ordering::Greater),
            (_, RuleTree::Lit(..)) => Some(Ordering::Less),

            // a rule and another rule: compare their priorities
            (RuleTree::Rul(r1, ..), RuleTree::Rul(r2, ..)) => Some(
                self.get_rules_priority(r1)?
                    .cmp(&self.get_rules_priority(r2)?),
            ),

            // a rule is higher than an end
            (RuleTree::Rul(..), RuleTree::End) => Some(Ordering::Greater),

            // an end is equal to an end
            (RuleTree::End, RuleTree::End) => Some(Ordering::Equal),

            // an end is less than anything else
            (RuleTree::End, _) => Some(Ordering::Less),
            // if no thingy yet then  a `cmp` b = b `cmp` a
            //(a, b) => self.cmp_rule_tree(b, a),
        }
    }

    pub fn into_rule_trees(self) -> HashMap<String, Boxs<RuleTree>> {
        let mut out = HashMap::<String, Vec<_>>::new();

        fn vec_to_tree(rule: &[RulePart]) -> RuleTree {
            if rule.is_empty() {
                return RuleTree::End;
            } else {
                let first = &rule[0];
                let rest = &rule[1..];

                match first {
                    RulePart::Empty => RuleTree::End,
                    RulePart::Literal(lit) => {
                        RuleTree::Lit(lit.clone(), [vec_to_tree(rest)].into_boxed_slice())
                    }
                    RulePart::Rule(name) => {
                        RuleTree::Rul(name.clone(), [vec_to_tree(rest)].into_boxed_slice())
                    }
                }
            }
        }

        for (name, rules) in &self.rules {
            for rule in rules {
                let v = match out.get_mut(name) {
                    Some(v) => v,
                    None => {
                        out.insert(name.to_owned(), vec![]);
                        out.get_mut(name).unwrap()
                    }
                };

                v.push(vec_to_tree(rule))
            }
        }

        fn smush(
            cmp: &impl Fn(&RuleTree, &RuleTree) -> Ordering,
            nodes: &mut Vec<RuleTree>,
        ) -> Boxs<RuleTree> {
            let mut out = Vec::new();

            'node: for node in nodes.iter_mut() {
                if out.is_empty() {
                    out.push(node.clone());
                    continue 'node;
                }

                for i in 0..out.len() {
                    if node.equivalent(&out[i]) {
                        let out_node = &mut out[i];

                        match out_node {
                            RuleTree::End => {}
                            RuleTree::Lit(_, rest) | RuleTree::Rul(_, rest) => {
                                let mut bs: Boxs<RuleTree> = "a"
                                    .repeat(rest.len())
                                    .chars()
                                    .map(|_| RuleTree::End)
                                    .into_boxed_slice();
                                bs.swap_with_slice(rest);

                                let mut v = bs.into_vec();

                                // merge node chilren into out_children
                                match node {
                                    RuleTree::Lit(_, rest) | RuleTree::Rul(_, rest) => {
                                        let mut bs: Boxs<RuleTree> = "a"
                                            .repeat(rest.len())
                                            .chars()
                                            .map(|_| RuleTree::End)
                                            .into_boxed_slice();
                                        bs.swap_with_slice(rest);
                                        v.extend(bs.into_vec().drain(..));
                                    }
                                    _ => unreachable!(),
                                }

                                // smuuuush
                                let bs = smush(cmp, &mut v);
                                // put the smushed trees back into the out_node.rest
                                *rest = bs;
                            }
                        }

                        continue 'node;
                    }

                    match cmp(node, &out[i]) {
                        Ordering::Greater => {
                            out.insert(i, node.clone());
                            continue 'node;
                        }

                        _ => {}
                    }
                }
                // here it means node wasnt added to the rest
                out.push(node.clone());
            }

            out.into_boxed_slice()
        }

        let mut o = HashMap::new();

        for (name, mut bs) in out {
            o.insert(
                name,
                smush(&|a, b| self.cmp_rule_tree(a, b).unwrap(), &mut bs),
            );
        }

        o
    }

    pub fn into_grammar(self) -> Grammar {
        Grammar::new(self.into_rule_trees())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleTree {
    // these have kinda dumb names, maybe they should be renamed

    // represents the end of a parsing tree
    End,
    // represents a literal to be parsed
    Lit(String, Boxs<RuleTree>),
    // represents a rule to be parsed
    Rul(String, Boxs<RuleTree>),
}

impl RuleTree {
    pub fn equivalent(&self, other: &RuleTree) -> bool {
        match (self, other) {
            (RuleTree::End, RuleTree::End) => true,
            (RuleTree::Lit(a, _), RuleTree::Lit(b, _)) => a == b,
            (RuleTree::Rul(a, _), RuleTree::Rul(b, _)) => a == b,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Grammar {
    rules: HashMap<String, Boxs<RuleTree>>,
}

impl Grammar {
    pub fn new(rules: impl Into<HashMap<String, Boxs<RuleTree>>>) -> Self {
        Self {
            rules: rules.into(),
        }
    }

    pub fn empty() -> Self {
        Self::new(HashMap::new())
    }

    pub fn get_rule(&self, k: &str) -> Option<&[RuleTree]> {
        self.rules.get(k).map(|v| v.as_ref())
    }

    pub fn add_rule(&mut self, k: impl Into<String>, tree: impl IntoBoxs<RuleTree>) {
        self.rules.insert(k.into(), tree.into_boxed_slice());
    }
}

impl Debug for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rule_names = self.rules.keys().cloned().collect::<Vec<_>>();
        rule_names.sort();

        let mut first = true;

        for rule_name in rule_names {
            if !first {
                f.write_str("\n")?;
            } else {
                first = false;
            }

            f.write_str(&rule_name)?;
            f.write_str(": ");
            Debug::fmt(self.rules.get(&rule_name).unwrap(), f)?;
            f.write_str(";\n")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aaab_rules() -> Rules {
        use RulePart::*;
        Rules::new([
            (
                "start".to_owned(),
                vec![RuleEntry {
                    definitions: vec![RuleDef {
                        parts: vec![Rule("A".to_owned())],
                        ..Default::default()
                    }],
                    precedence: 0,
                    ..Default::default()
                }],
            ),
            (
                "A".to_owned(),
                vec![RuleEntry {
                    definitions: vec![
                        RuleDef {
                            parts: vec![Literal("b".to_owned())],
                            ..Default::default()
                        },
                        RuleDef {
                            parts: vec![Literal("a".to_owned()), Rule("A".to_owned())],
                            ..Default::default()
                        },
                    ],
                    precedence: 0,
                    ..Default::default()
                }],
            ),
        ])
    }

    fn aaab_flattened_rules() -> FlattenedRules {
        use RulePart::*;
        use RuleType::*;
        FlattenedRules::new(
            [
                ("start".to_owned(), vec![vec![Rule("A".to_owned())]]),
                (
                    "A".to_owned(),
                    vec![
                        vec![Literal("b".to_owned())],
                        vec![Literal("a".to_owned()), Rule("A".to_owned())],
                    ],
                ),
            ],
            [("start".to_owned(), Acyclic(0)), ("A".to_owned(), Cyclic)],
        )
    }

    #[test]
    fn test_flattening_0() {
        let rules = aaab_rules();
        let flattened_rules = aaab_flattened_rules();

        assert_eq!(flattened_rules, rules.flattened());
    }
}
