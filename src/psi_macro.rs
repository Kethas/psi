macro_rules! rulepart {
    ($name:ident) => {
        RulePart::Rule(stringify!($name).to_owned())
    };

    ($lit:literal) => {{
        let literal: &str = $lit;
        RulePart::Literal(literal.to_owned())
    }};
}

macro_rules! rule_vec {
    (_) => {{
        use crate::grammar::*;

        Vec::<RulePart>::new()
    }};
    
    (($($tok:tt)*)) => {{
        use crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};
    ([$($tok:tt)*]) => {{
        use crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};
    ({$($tok:tt)*}) => {{
        use crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};
    ($($tok:tt)*) => {{
        use crate::grammar::*;

        let v: Vec<RulePart> = vec![$(rulepart!($tok)),*];
        v
    }};

}

macro_rules! assoc {
    (left) => {{
        use crate::grammar::Associativity::*;
        Left
    }};
    (right) => {{
        use crate::grammar::Associativity::*;
        Right
    }};
    (nonassoc) => {{
        use crate::grammar::Associativity::*;
        None
    }};
    (none) => {{
        use crate::grammar::Associativity::*;
        None
    }};
}

macro_rules! rule_entry {
    (
        $(@prec $($assoc:ident)? = $prec:expr,)?
        $name:ident:
        $($tok:tt
            $(-> $action:expr)?
        ),*
        ;) => {{
            use crate::grammar::*;

        (stringify!($name).to_owned(), RuleEntry {
            definitions: {
                let mut v = vec![];

                $({
                    let parts = rule_vec!($tok);

                    $(if true {
                        use std::sync::Arc;
                        use uuid::Uuid;
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

    (
        $(@prec $($assoc:ident)? = $prec:expr,)?    
            $name:ident
        $(| $tok:tt
            $(-> $action:expr)?
        )* 
        ;
    ) => {{
        rule_entry!(
            $(@prec $($assoc)? = $prec)?,
            $name: $($tok $(-> $action)?),* ;
        )
    }};
}

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
        use crate::grammar::{RuleEntry, Rules};

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

        let mut map: HashMap<String, Vec<RuleEntry>> = HashMap::new();

        for (name, rule_entry) in rules {
            if map.contains_key(&name) {
                map.get_mut(&name).unwrap().push(rule_entry);
            } else {
                map.insert(name, vec![rule_entry]);
            }
        }

        Rules::new(map)
    
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

macro_rules! psi {
    ($($tt:tt)*) => {{
        rules! {
            $($tt)*
        }.into_grammar()
    }}
}

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
                  [a "1"] -> |pt| ParseObject::ParseTree(pt);
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
