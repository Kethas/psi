#[macro_export]
/// A RulePart -- Either a Rule (identifier) or a Literal (string literal).
macro_rules! rulepart {
    ($name:ident) => {
        RulePart::Rule(stringify!($name).to_owned())
    };

    ($lit:literal) => {{
        let literal: &str = $lit;
        RulePart::Literal(literal.to_owned())
    }};
}

pub use rulepart;

#[macro_export]
// A vector of rules -- Either an empty vector (_), any list of tokens ({},[],()), or any single rule part.
macro_rules! rule_vec {
    (_) => {{
        use $crate::grammar::*;

        Vec::<RulePart>::new()
    }};

    (($($tok:tt)*)) => {{
        use $crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};
    ([$($tok:tt)*]) => {{
        use $crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};
    ({$($tok:tt)*}) => {{
        use $crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};
    ($($tok:tt)*) => {{
        use $crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};

}

pub use rule_vec;

#[macro_export]
// The associativity -- left, right, or none/nonassoc
macro_rules! assoc {
    (left) => {{
        Associativity::Left
    }};
    (right) => {{
        Associativity::Right
    }};
    (nonassoc) => {{
        Associativity::None
    }};
    (none) => {{
        Associativity::None
    }};
}

#[macro_export]
// A rule entry.
macro_rules! rule_entry {
    (
        $(@prec $($assoc:ident)? = $prec:expr,)?
        $name:ident:
        $($tok:tt
            $(-> $action:expr)?
        ),*
        ;) => {{
            use $crate::grammar::*;


        (stringify!($name).to_owned(), RuleEntry {
            definitions: {
                let mut v = vec![];

                $({
                    let parts = rule_vec!($tok);

                    $(if true {
                        use std::sync::Arc;
                        use uuid::Uuid;
                        use $crate::parse::parsed::ParseObject;
                        use ParseObject::*;
                        use eyre;
                        use eyre::{Result, ContextCompat};
                        
                        let action = RuleAction {
                            inner: Arc::new($action),
                            id: Uuid::new_v4(),
                        };

                        v.push(RuleDef {
                            parts,
                            action,
                        });
                    } else)?
                    {
                        v.push(RuleDef::from_parts(parts));
                    }
                })*

                v
            },
            precedence: {
                let mut out = 0;

                $(if true {
                    out = $prec;
                })?

                out
            },
            associativity: {
                use Associativity::*;
                let mut out = Left;

                $($(if true {
                    out = assoc!($assoc);
                })?)?

                out
            }
        })
    }};
}
pub use rule_entry;


/// This macro can be used to generate a Psi Grammar.
/// Example:
/// ```
/// # #[macro_use] extern crate psi;
/// use psi::*;
///
/// # fn main() {
/// let grammar = psi!{
///     start: a;
///
///     a: "a",
///        (b a);
///     b: "b";
/// };
///
/// let source = "ba".chars();
/// let mut parser = Parser::<CharsInput>::new(source);
///
/// let result = parser.parse(&grammar).expect("Failed to parse.");
///
/// use psi::parse::parsed::ParseObject::*;
/// assert_eq!(result,
///     Rule("start".to_owned(),
///         vec![
///             Rule("a".to_owned(), vec![
///                 Rule("b".to_owned(), vec![
///                     Literal("b".to_owned())
///                 ]),
///                 Rule("a".to_owned(), vec![
///                     Literal("a".to_owned())
///                 ])
///             ])
///         ]
///     )
/// )
/// # }
///
/// ```
#[macro_export]
macro_rules! rules {
    (
        $(
            $(@prec $($assoc:ident)? = $prec:expr,)?
            $name:ident:
            $($tok:tt
                $(-> $action:expr)?
            ),*
        );* $(;)?
    ) => {{
        use std::collections::HashMap;
        use $crate::grammar::*;
        use $crate::psi_macro::*;


        let rules = vec![
            $(
                rule_entry!($(@prec $($assoc)? = $prec,)?
                            $name:
                            $(
                                $tok
                                $(-> $action)?
                            ),*
                            ;
            )
            ),*
        ];

        let mut map: HashMap<std::string::String, Vec<RuleEntry>> = HashMap::new();

        for (name, rule_entry) in rules {
            if map.contains_key(&name) {
                map.get_mut(&name).unwrap().push(rule_entry);
            } else {
                map.insert(name, vec![rule_entry]);
            }
        }

        Rules::new(map).into_grammar()
    }};

    // (
    //     $(
    //         $(@prec $($assoc:ident)? = $prec:expr,)?
    //         $name:ident
    //         $(| $tok:tt
    //             $(-> $action:expr)?
    //         )*
    //         ;
    //     );* $(;)?
    // ) => {{

    // }};
}

pub use rules;

//TODO: add a grammar rule to separate the Rules macro and Grammar macro instead of rules -> Grammar

#[cfg(test)]
mod tests {
    use crate::parse::parsed::ParseObject;

    #[test]
    fn test() {
        let _ = rule_vec!(_);
        let _ = rule_vec!(());
        let _ = rule_vec!(a "b" c);

        let _ = rule_entry!(
            @prec left = 13,
            name: _,
                  a,
                  [a "1"] -> |x| Ok(x);
        );

        let rules = rules! {
            start: expr;

            @prec = 0,
            expr: "()",
                  ("(" expr ")");

            @prec left = 10,
            expr: expr,
                  (expr "+" expr),
                  (expr "-" expr);
        };
    }
}
