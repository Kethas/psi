use crate::declare_rules;

use super::*;

declare_rules! {
    pub Integer {
        start /* isize */ { (integer) }

        integer /* isize */ {
            (unsigned) => |v| (*v(0).downcast::<usize>().unwrap() as isize).into_value();
            ("+" unsigned) => |v| (*v(1).downcast::<usize>().unwrap() as isize).into_value();
            ("-" unsigned) => |v| (-(*v(1).downcast::<usize>().unwrap() as isize)).into_value();
        }

        unsigned /* usize */ {
            ("0") => |_| 0_usize.into_value();
            (_int)
                => |v| v(0).downcast::<String>().unwrap().parse::<usize>().unwrap().into_value();
        }

        _int {
            (digit_nonzero)
                => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();
            (_int digit)
                => |v| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();
        }

        digit {
            (digit_nonzero)
            ("0")
        }

        digit_nonzero {
            ("1")
            ("2")
            ("3")
            ("4")
            ("5")
            ("6")
            ("7")
            ("8")
            ("9")
        }
    }
}
