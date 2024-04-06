use super::*;

declare_rules! {
    // Name ends in Rules because otherwise std::string::String is shadowed
    pub StringRules {
        #[import (Hex) as hex]

        string {
            ("\"" string_inner "\"") => |v, _| v(1);
            ("\"\"") => |_, _| String::new().into_value();
        }

        string_inner {
            (string_char)
            (string_inner string_char)
                => |v, _| format!(
                    "{}{}",
                    v(0).downcast::<String>().unwrap(),
                    v(1).downcast::<String>().unwrap()
                ).into_value();
        }

        string_char {
            (escape)
            ((! "\""))
                => |v, _| v(0).downcast::<Token>().unwrap().to_string().into_value();
        }

        escape {
            ("\\\"") => |_, _| "\"".to_owned().into_value();
            ("\\\\") => |_, _| "\\".to_owned().into_value();
            ("\\n") => |_, _| "\n".to_owned().into_value();
            ("\\r") => |_, _| "\r".to_owned().into_value();
            ("\\t") => |_, _| "\t".to_owned().into_value();
            ("\\0") => |_, _| "\0".to_owned().into_value();
            ("\\x" (hex::digit) (hex::digit)) => |v, _| {
                let digit1 = v(1).downcast::<Token>().unwrap().to_string();
                let digit2 = v(2).downcast::<Token>().unwrap().to_string();

                let number = format!("{digit1}{digit2}");
                let char = u8::from_str_radix(&number, 16).unwrap();

                if char <= 0x7F {
                    [(char as char)].into_iter().collect::<String>().into_value()
                } else {
                    panic!("Illegal escape code: \\x{digit1}{digit2}");
                }

            };
            ("\\u{" (hex::raw_hex) "}") => |v, _| {
                let hex = v(1).downcast::<String>().unwrap();
                if hex.chars().count() > 6 {
                    panic!("Illegal escape code: \\u{{{hex}}}");
                } else {
                    let unicode_char = u32::from_str_radix(&hex, 16).unwrap();
                    let unicode_char = char::from_u32(unicode_char).unwrap();

                    [unicode_char].into_iter().collect::<String>().into_value()
                }
            };
        }
    }
}
