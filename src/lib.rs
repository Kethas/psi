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

mod grammar {
    use crate::utils::*;
    use std::{collections::HashMap, cmp::Ordering, vec};

    #[derive(Clone, Debug)]
    pub enum RulePart {
        Empty,
        Literal(String),
        Rule(String),
    }

    #[derive(Clone, Debug)]
    pub struct RuleDef {
        pub parts: Vec<RulePart>,
    }

    #[derive(Clone, Debug)]
    pub struct RuleEntry {
        pub definitions: Vec<RuleDef>,
        pub precedence: u64,
    }

    #[derive(Clone, Debug)]
    pub struct Rules {
        rules: HashMap<String, Vec<RuleEntry>>,
    }

    /*

    analyse cycles in the rules
        cyclic rules are infinite size, therefore zero priority
        uncyclic rules are finite and can have a size (priority) calculated
        literals have a priority equal to their size
        empty is the lowest priority
    */

    impl Rules {
        pub fn new(rules: impl Into<HashMap<String, Vec<RuleEntry>>>) -> Self {
            Self {
                rules: rules.into(),
            }
        }

        pub fn flattened(self) -> FlattenedRules {
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
                    //TODO
                    continue;
                }

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
                            //TODO
                        }

                        //TODO
                    }
                }
            }

            todo!()
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    enum RuleType {
        Cyclic,
        Acyclic(usize),
    }
    use RuleType::*;

    struct FlattenedRules {
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

            for (name, ty) in &mut self.rule_types  {
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
        fn compute_size(
            &mut self,
            base: &str,
        ) -> Option<usize> {
            let mut largest_child = 0;

            for rules in self.rules.get(base) {
                for rule in rules {
                    let mut rule_size = 0;
                    for part in rule {
                        match part {
                            RulePart::Empty => {},
                            RulePart::Literal(lit) => rule_size += lit.chars().count(),
                            RulePart::Rule(name) => {
                                let size = self.get_rules_priority(name)?;
                                if size == usize::MAX {
                                    return None
                                }
                                rule_size += size
                            },
                        }
                    }

                    if rule_size > largest_child {
                        largest_child = rule_size;
                    }
                }
            };

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
                        RulePart::Literal(lit) => RuleTree::Lit(lit.clone(), [vec_to_tree(rest)].into_boxed_slice()),
                        RulePart::Rule(name) => RuleTree::Rul(name.clone(), [vec_to_tree(rest)].into_boxed_slice()),
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
                        },
                    };

                    v.push(vec_to_tree(rule))
                }
            }

            todo!("smush");

            out.into_iter().map(|(name, vec)| (name, vec.into_boxed_slice())).collect()
        }

        pub fn into_grammar(self) -> Grammar {
            Grammar::new(self.into_rule_trees())
        }

    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum RuleTree {
        // represents the end of a parsing tree
        End,
        // represents a literal to be parsed
        Lit(String, Boxs<RuleTree>),
        // represents a rule to be parsed
        Rul(String, Boxs<RuleTree>),
    }

    #[derive(Clone, Debug)]
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
}

mod rule {
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
        use crate::{
            input::CharsInput,
            utils::{IntoBoxs},
        };

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
    use crate::grammar::*;

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
    #[test]
    fn test_0() {
        println!("works!")
    }
}
