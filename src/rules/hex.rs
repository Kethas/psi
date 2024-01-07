use super::*;

declare_rules! {
    pub Hex {
        hex {
            (prefixed_hex) 
                => |v| usize::from_str_radix(
                    &v(0).downcast::<String>().unwrap(),
                    16
                ).unwrap().into_value();
        }

        prefixed_hex {
            ("0x" raw_hex) => |v| v(1);
        }

        raw_hex {
            (digit)
                => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();
            (raw_hex digit)
                => |v| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<Token>().unwrap()
                ).into_value();
        }

        digit {
            ("0")
            ("1")
            ("2")
            ("3")
            ("4")
            ("5")
            ("6")
            ("7")
            ("8")
            ("9")
            ("a")
            ("b")
            ("c")
            ("d")
            ("e")
            ("f")
            ("A")
            ("B")
            ("C")
            ("D")
            ("E")
            ("F")
        }
    }
}
