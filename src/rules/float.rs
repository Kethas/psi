use super::*;

declare_rules! {
    pub Float {
        #[import (Integer) as integer]
        float /* f64 */ {
            (_int) => |v| v(0).downcast::<String>().unwrap().parse::<f64>().unwrap().into_value();
            (digits "." digits)
                => |v| format!(
                    "{}.{}",
                    v(0).downcast::<String>().unwrap(),
                    v(2).downcast::<String>().unwrap()
                ).parse::<f64>().unwrap().into_value();
        }

        _int /* String */ {
            ((integer::digit)) => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();
            ((integer::digit_nonzero) digits)
                => |v| format!(
                    "{}{}",
                    v(0).downcast::<Token>().unwrap(),
                    v(1).downcast::<String>().unwrap()
                ).into_value();
        }

        digits /* String */ {
            ((integer::digit)) => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();
            (digits (integer::digit))
                => |v| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();
        }
    }
}
