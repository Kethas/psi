use super::*;

declare_rules! {
    pub Identifier {
        #[import (Integer) as int]
        #[import (Alpha) as alpha]

        identifier {
            (identifier_start)
                => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();
            (identifier identifier_continue)
                => |v| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();
        }

        identifier_start {
            ((alpha::alpha))
            ("_")
        }

        identifier_continue {
            (identifier_start)
            ((int::digit))
        }
    }
}
