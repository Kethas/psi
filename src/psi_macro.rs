// i give up gonna try to make this with a proc macro

use crate::grammar::*;

macro_rules! psi_rulepart {
    ($name:ident) => {{
        RulePart::Rule(stringify!($name).to_owned())
    }};
    ($lit:literal) => {{
        RulePart::Literal($lit.to_string())
    }};
}

macro_rules! _psi_rule {
    (_) => {{
        vec![RulePart::Empty]
    }};
    (($($part:tt)+,) _?) => {{
        vec![$(psi_rulepart!($part))+,]
    }};
}

macro_rules! psi_rule {
    ($($x:tt)+) => {{
        RuleDef {
            parts: _psi_rule!($($x)+)
        }
    }};
}

macro_rules! psi_rule_entry {
    (@$name:ident => $(($contents:tt))+) => {{
        (stringify!($name).to_owned(), RuleEntry {
            precedence: 0,
            definitions: vec![$(
                psi_rule!(
                    $contents
                )
            )+,]
        })
    }};
    (@$name:ident => $($contents:tt)+) => {{
        psi_rule_entry!(@$name => ($($contents)+))
    }};
}

/*fn ___() {
        raw_psi!{
            start = abab;
            abab = "abab";
        }
    }*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::*;

    #[test]
    fn test_psi_rule_part_macro() {
        assert_eq!(psi_rulepart!(expr), RulePart::Rule("expr".to_owned()));
        assert_eq!(psi_rulepart!("32"), RulePart::Literal("32".to_owned()));
    }

    #[test]
    fn test_psi_rule_macro() {
        //assert_eq!(_psi_rule!((expr)), vec![RulePart::Rule("expr".to_owned())]);
        //assert_eq!(_psi_rule!(("32")), vec![RulePart::Literal("32".to_owned())]);
    }

    #[test]
    fn test_psi_rule_entry_macro() {
        // assert_eq!(psi_rule_entry!(@expr => (expr "+" expr)), ("expr".to_owned(), RuleEntry { definitions: vec![
        //     RuleDef {
        //         parts: vec![RulePart::Rule("expr".to_owned()), RulePart::Literal("+".to_owned()), RulePart::Rule("expr".to_owned())]
        //     }
        // ], precedence: 0 }))
    }

    #[test]
    fn test_psi_macro() {}
}
