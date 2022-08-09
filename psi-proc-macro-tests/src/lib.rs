#[cfg(test)]
mod tests {
    use psi_proc_macro::*;
    use psi::grammar::*;

    #[test]
    fn test_psi_rulepart_macro() {
        assert_eq!(psi_rulepart!(expr), RulePart::Rule("expr".to_owned()));
        assert_eq!(
            psi_rulepart!("123123"),
            RulePart::Literal("123123".to_owned())
        );
        assert_ne!(psi_rulepart!("123123"), RulePart::Rule("expr".to_owned()));
    }

    #[test]
    fn test_psi_rule_macro() {
        assert_eq!(psi_rule!(_), RuleDef {
            parts: vec![RulePart::Empty]
        });
        assert_eq!(psi_rule!(expr "+" expr), RuleDef {
            parts: vec![RulePart::Rule("expr".to_owned()), RulePart::Literal("+".to_owned()), RulePart::Rule("expr".to_owned())]
        });
    }
}
