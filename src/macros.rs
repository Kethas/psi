#[allow(dead_code)]
#[macro_export]
macro_rules! rule_part {
    ($lit:literal) => {
        psi_parser::rule::RulePart::Term(String::from($lit))
    };

    ($rule:ident) => {
        psi_parser::rule::RulePart::NonTerm(stringify!($rule).to_owned())
    };

    ((! $($lit:literal)*)) => {
        psi_parser::rule::RulePart::Not([$(String::from($lit)),*].into_iter().collect())
    }


}

#[allow(dead_code)]
#[macro_export]
macro_rules! rule {
    ($name:ident: ($($tt:tt)*) $(=> $transformer:expr)?) => {{

        #[allow(unused_variables)]
        let transformer: Option<psi_parser::rule::Transformer> = None;

        $(
            let transformer: Option<psi_parser::rule::Transformer> = Some(Box::new($transformer));
        )?

        Rule {
            name: stringify!($name).to_owned(),
            parts: vec![$(psi_parser::rule_part!($tt)),*],
            transformer
        }
    }};
}

#[allow(dead_code)]
#[macro_export]
macro_rules! rules {
    (
        $(
            $rule_name:ident {
                $(
                    ($( $tt:tt )*)
                    $(=> $transformer:expr;)?
                )+
            }
        )+
    ) => {{
        let mut rules = Vec::new();

        $($(
            rules.push(rule!($rule_name: ($($tt)*) $(=> $transformer)?).into());
        )*)*

        Rules::new(rules)
    }};
}
