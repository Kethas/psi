#[allow(dead_code)]
#[macro_export]
macro_rules! rule_part {
    ($lit:literal) => {
        $crate::rule::RulePart::Term(std::string::String::from($lit))
    };

    ($rule:ident) => {
        $crate::rule::RulePart::NonTerm(stringify!($rule).to_owned())
    };

    (($rule:path)) => {{

        $crate::rule::RulePart::NonTerm(stringify!($rule).to_owned())
    }};

    ((! $($lit:literal)*)) => {
        $crate::rule::RulePart::Not([$(std::string::String::from($lit)),*].into_iter().collect())
    }


}

#[allow(dead_code)]
#[macro_export]
macro_rules! rule {
    ($name:ident: ($($tt:tt)*) $(=> $transformer:expr)?) => {{

        #[allow(unused_variables)]
        let transformer: Option<$crate::rule::Transformer> = None;

        $(
            let transformer: Option<$crate::rule::Transformer> = Some(std::rc::Rc::new($transformer));
        )?

        Rule {
            name: stringify!($name).to_owned(),
            parts: vec![$($crate::rule_part!($tt)),*],
            transformer
        }
    }};
}

#[allow(dead_code)]
#[macro_export]
macro_rules! rules {
    (
        $(#[import ($rules_expr:expr) $(as $rules_name:ident)?])*
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

        let rules = $crate::rule::Rules::new(rules);

        $(
            let mut rules = rules;
            let rules_name: Option<std::string::String> = None$(.or(Some(stringify!($rules_name).to_owned())))?;
            rules.import(Into::<$crate::rule::Rules>::into($rules_expr), rules_name);
        )*

        rules
    }};
}

#[macro_export]
macro_rules! declare_rules {
    ($visibility:vis $name:ident { $($tt:tt)* }) => {
        $visibility struct $name;

        impl From<$name> for $crate::rule::Rules {
            fn from(_: $name) -> Self {
                rules! {
                    $($tt)*
                }
            }
        }

        impl $name {
            pub fn parse<'a>(
                &self,
                start_rule: &str,
                input: impl Into<$crate::input::Input<'a>>,
            ) -> Result<$crate::result::ParseValue, $crate::result::ParseError> {
                $crate::rule::Rules::from(Self).parse(start_rule, input)
            }

            pub fn parse_entire<'a>(
                &self,
                start_rule: &str,
                input: impl Into<$crate::input::Input<'a>>,
            ) -> Result<$crate::result::ParseValue, $crate::result::ParseError> {
                $crate::rule::Rules::from(Self).parse_entire(start_rule, input)
            }
        }

    };
}
