mod utils {
    pub type Boxs<T> = Box<[T]>;

    pub trait IntoBoxs<T> {
        fn into_boxed_slice(self) -> Boxs<T>;
    }

    impl<I, T> IntoBoxs<T> for I
    where
        I: IntoIterator<Item = T>,
    {
        fn into_boxed_slice(self) -> Boxs<T> {
            Box::from_iter(self.into_iter())
        }
    }

    pub trait Void: Sized {
        fn void(self) -> () {
            ()
        }
    }

    impl<T> Void for T {}

    pub trait RefVoid {
        fn void(&self) -> () {
            ()
        }
    }

    impl<T> RefVoid for T {}
}

//TODO: expand input types and optimize their cloning efficiency
mod input {
    use std::ops::Range;

    use crate::utils::IntoBoxs;

    pub trait Input: Clone {
        fn get(&self, index: usize) -> Option<char>;
        fn get_pos(&self) -> usize;
        fn set_pos(&mut self, pos: usize);
        fn slice(&self, range: Range<usize>) -> &[char];

        fn slice_to_string(&self, range: Range<usize>) -> String {
            self.slice(range).into_iter().copied().collect()
        }

        fn current(&self) -> Option<char> {
            self.get(self.get_pos())
        }
        fn next(&mut self) -> Option<char> {
            self.advance();
            self.current()
        }

        fn advance_by(&mut self, n: usize) {
            self.set_pos(self.get_pos() + n)
        }

        fn retreat_by(&mut self, n: usize) {
            self.set_pos(self.get_pos() - n)
        }

        fn advance(&mut self) {
            self.advance_by(1);
        }

        fn retreat(&mut self) {
            self.retreat_by(1);
        }
    }

    #[derive(Clone, Debug)]
    pub struct CharsInput {
        inner: Box<[char]>,
        pos: usize,
    }

    impl Input for CharsInput {
        fn get(&self, index: usize) -> Option<char> {
            self.inner.get(index).copied()
        }

        fn get_pos(&self) -> usize {
            self.pos
        }

        fn set_pos(&mut self, pos: usize) {
            self.pos = pos;
        }

        fn slice(&self, range: Range<usize>) -> &[char] {
            &self.inner[range]
        }
    }

    impl<I: IntoBoxs<char>> From<I> for CharsInput {
        fn from(chars: I) -> Self {
            Self {
                inner: chars.into_boxed_slice(),
                pos: 0,
            }
        }
    }
}

//TODO: split into grammar and rules, maybe grammar > rules with pub use Rules
mod grammar {
    use crate::utils::*;
    use std::{cmp::Ordering, collections::HashMap, vec};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum RulePart {
        Empty, // this can be removed, if verified it is not needed, ie can be replaced by [] or nothing
        Literal(String),
        Rule(String),
    }

    //TODO: maybe flatten this into just a vec, as it seems unecessary at this point to have
    // attributes for a single definition when a group of defitnitions can be a group of one
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RuleDef {
        pub parts: Vec<RulePart>,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
    pub enum Associativity {
        #[default]
        Left,
        Right,
        None,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RuleEntry {
        pub definitions: Vec<RuleDef>,
        pub precedence: u64,
        // pub associativity: Associativity,
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
            let mut rules: HashMap<String, HashMap<u64, Vec<Vec<RulePart>>>> = HashMap::new();

            for (name, rule_entries) in self.rules {
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

                    for def in rule_entry.definitions {
                        precedence.push(def.parts);
                    }
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

                    for (_, rule) in precedences {
                        v.extend(rule);
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
                for (precedence, rules) in precedences {
                    let prec_name = format!("{name}@{precedence}");
                    let next_prec = format!(
                        "{name}@{}",
                        get_next_prec(precedence, all_precedences.clone())
                    );

                    for rule in rules {
                        let mut out_rule = Vec::new();

                        if rule.is_empty() {
                            out_rule.push(RulePart::Empty)
                        } else {
                            out_rule = rule;
                            if let Some(part) = out_rule.first_mut() {
                                match part {
                                    RulePart::Rule(n) if n == &name => *n = next_prec.clone(),
                                    _ => {}
                                }

                                drop(part);

                                for part in &mut out_rule[1..] {
                                    match part {
                                        RulePart::Rule(n) if n == &name => *n = prec_name.clone(),
                                        _ => {}
                                    }
                                }
                            }
                        }

                        let v = match out.get_mut(&prec_name) {
                            Some(v) => v,
                            None => {
                                out.insert(prec_name.clone(), Vec::new());
                                out.get_mut(&prec_name).unwrap()
                            }
                        };

                        v.push(out_rule);
                    }
                }
            }

            FlattenedRules::from_rules(out)
        }

        pub fn into_grammar(self) -> Grammar {
            self.flattened().into_grammar()
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum RuleType {
        Cyclic,
        Acyclic(usize),
    }
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
                    // here it means node wasnt added to the rest
                    out.push(node.clone());

                    for i in 0..out.len() {
                        if node.equivalent(&out[i]) {
                            let out_node = &mut out[i];

                            match out_node {
                                RuleTree::End => {}
                                RuleTree::Lit(_, rest) | RuleTree::Rul(_, rest) => {
                                    let mut bs: Boxs<RuleTree> = "a".repeat(rest.len()).chars().map(|_| RuleTree::End).into_boxed_slice();
                                    bs.swap_with_slice(rest);

                                    let mut v = bs.into_vec();

                                    // merge node chilren into out_children
                                    match node {
                                        RuleTree::Lit(_, rest) | RuleTree::Rul(_, rest) => {
                                            let mut bs: Boxs<RuleTree> = "a".repeat(rest.len()).chars().map(|_| RuleTree::End).into_boxed_slice();
                                            bs.swap_with_slice(rest);
                                            v.extend(bs.into_vec().drain(..));
                                        },
                                        _ => unreachable!()
                                    }


                                    // smuuuush
                                    let bs = smush(cmp, &mut v);
                                    // put the smushed trees back into the out_node.rest
                                    *rest = bs;
                                },
                            }

                            continue 'node;
                        }

                        match cmp(node, &out[i]) {
                            Ordering::Greater => {
                                out.insert(i, node.clone());
                                continue 'node
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
                
                o.insert(name, smush(&|a, b| self.cmp_rule_tree(a, b).unwrap(), &mut bs));
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

    #[derive(Clone, Debug, PartialEq, Eq)]
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
                        }],
                        precedence: 0,
                    }],
                ),
                (
                    "A".to_owned(),
                    vec![RuleEntry {
                        definitions: vec![
                            RuleDef {
                                parts: vec![Literal("b".to_owned())],
                            },
                            RuleDef {
                                parts: vec![Literal("a".to_owned()), Rule("A".to_owned())],
                            },
                        ],
                        precedence: 0,
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
}

//TODO: make a non-recursive parsing function
mod parse {
    use crate::{grammar::*, input::Input, utils::Boxs};

    #[derive(Clone, Debug)]
    pub enum Parsed {
        End,
        Literal(String),
        Rule(String, Vec<Parsed>),
    }

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

    pub type ParseResult = Result<Parsed, ParseError>;

    #[derive(Clone, Debug)]
    pub struct Parser<I: Input> {
        input: I,
    }

    #[derive(Clone, Copy, Debug)]
    enum ParseFlag {
        ReturnToBasePos,
        NoSelf,
    }

    impl<I: Input> Parser<I> {
        pub fn new(input: impl Into<I>) -> Self {
            Self {
                input: input.into(),
            }
        }

        pub fn parse(&mut self, grammar: &Grammar) -> ParseResult {
            self.parse_rule_by_name(grammar, "start")
        }

        pub fn parse_rule_by_name(&mut self, grammar: &Grammar, rule_name: &str) -> ParseResult {
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
        ) -> ParseResult {
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
        ) -> ParseResult {
            let (val, rest): (Parsed, &[RuleTree]) = match node {
                RuleTree::End => return Ok(Parsed::Rule(base.to_owned(), Vec::new())),
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

                    (Parsed::Literal(lit.clone()), rest)
                }
                RuleTree::Rul(name, rest) => {
                    let val = self.parse_rule_by_name(grammar, name)?;

                    (val, rest)
                }
            };

            let mut res = self.parse_rule(grammar, rest, base, depth + 1)?;

            match &mut res {
                // push the value into the beginning of the return buffer
                Parsed::Rule(name, vals) if name == base => {
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
                    [RuleTree::End].into_boxed_slice(),
                )],
            );

            g.add_rule(
                "A",
                [
                    RuleTree::Lit("b".to_owned(), [RuleTree::End].into_boxed_slice()),
                    RuleTree::Lit(
                        "a".to_owned(),
                        [RuleTree::Rul(
                            "A".to_owned(),
                            [RuleTree::End].into_boxed_slice(),
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
            fn count_aaab(parsed: &Parsed, a_count: &mut usize, b_count: &mut usize) {
                match parsed {
                    Parsed::End => {}
                    Parsed::Literal(a) if a == "a" => *a_count += 1,
                    Parsed::Literal(b) if b == "b" => *b_count += 1,
                    Parsed::Literal(_) => unreachable!(),
                    Parsed::Rule(_, inner) => inner
                        .iter()
                        .map(|x| count_aaab(x, a_count, b_count))
                        .collect(),
                };
            }

            count_aaab(&parsed, &mut a_count, &mut b_count);

            assert_eq!(a_count, expected_a_count);
            assert_eq!(b_count, expected_b_count);

            // fail on purpose to show stdout
            //assert!(false);
        }
    }
}

mod psi_macro {
    /*use crate::grammar::*;

    macro_rules! raw_psi {
        () => {{
            let mut g = Grammar::empty();

            {}

            g
        }};
    }

    macro_rules! raw_rule {
        ($lit:literal, ) => {};

        ($rulename:ident) => {};
    }

    */

    /*fn ___() {
        raw_psi!{
            start = abab;
            abab = "abab";
        }
    }*/

    #[cfg(test)]
    mod tests {
        #[test]
        pub fn test_psi_macro() {}
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{grammar::{Grammar, RuleDef, RuleEntry, RulePart, Rules}, input::CharsInput, parse::Parser};

    fn compile_expr_grammar() -> Grammar {
        let rules = Rules::new([
            (
                "start".to_owned(),
                vec![RuleEntry {
                    definitions: vec![RuleDef {
                        parts: vec![RulePart::Rule("expr".to_owned())],
                    }],
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
                            },
                            RuleDef {
                                parts: vec![RulePart::Rule("expr".to_owned())],
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
                            },
                            RuleDef {
                                parts: vec![
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal("-".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                ],
                            },
                            RuleDef {
                                parts: vec![RulePart::Rule("expr".to_owned())],
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
                            },
                            RuleDef {
                                parts: vec![
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal("/".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                ],
                            },
                            RuleDef {
                                parts: vec![RulePart::Rule("expr".to_owned())],
                            },
                        ],
                        precedence: 10,
                    },
                    RuleEntry {
                        definitions: vec![
                            RuleDef {
                                parts: vec![RulePart::Literal("x".to_owned())],
                            },
                            RuleDef {
                                parts: vec![
                                    RulePart::Literal("(".to_owned()),
                                    RulePart::Rule("expr".to_owned()),
                                    RulePart::Literal(")".to_owned()),
                                ],
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

    #[test]
    fn test_expr_0() {
        let compiled_grammar = compile_expr_grammar();
        //let expected_grammar = expr_grammar();


        println!("Compiled Grammar:\n{:#?}", compiled_grammar);

        //assert_eq!(expected_grammar, compiled_grammar);


        let input = "x+x*x+x".chars();
        //let result = 1 + 3 * 8 + 2;

        let mut parser = Parser::<CharsInput>::new(input);

        let result = parser.parse(&compiled_grammar);

        if let Err(e) = result {
            eprintln!("ParseError:\n{:#?}", e);
            panic!("Error encountered while parsing.");
        }

        let result = result.unwrap();
        
        println!("Parsed:\n{:#?}", result);

        // fail on purpose to see output
        //assert!(false);

        // great success! wawaweewa!
    }
}
