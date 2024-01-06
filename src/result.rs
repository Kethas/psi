use super::input::Input;
use std::any::Any;

use derive_more::{Deref, DerefMut, Display};

pub type ParseValue = Box<dyn Any>;

pub trait IntoParseValue {
    fn into_value(self) -> ParseValue;
}

impl<T: Any> IntoParseValue for T {
    fn into_value(self) -> ParseValue {
        Box::new(self)
    }
}

#[derive(Clone, Debug, Display, Deref, DerefMut, Eq, PartialEq, Hash)]
pub struct Token(String);

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T: Into<String>> From<T> for Token {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ParseError {
    RuleNotFound {
        rule_name: String,
    },
    UnexpectedChar {
        current_rule: String,
        char: Option<char>,
        pos: usize,
        row: usize,
        col: usize,
    },
    UnexpectedToken {
        current_rule: String,
        token: String,
        pos: usize,
        row: usize,
        col: usize,
    },
    MultipleErrors {
        current_rule: String,
        errors: Vec<ParseError>,
    },
}

pub type ParseResult<'a> = Result<Option<(ParseValue, Input<'a>)>, ParseError>;
