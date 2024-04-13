use std::path::Path;
use std::{any::Any, error::Error, fmt::Display};
use std::cmp::Ordering;

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

#[derive(Debug)]
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
    TransformerError {
        current_rule: String,
        pos: usize,
        row: usize,
        col: usize,
        error: Box<dyn Error>,
    },
}

impl Error for ParseError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            ParseError::TransformerError { error, .. } => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::RuleNotFound { rule_name } => {
                f.write_fmt(format_args!("Rule '{rule_name}' not found"))
            }
            ParseError::UnexpectedChar {
                current_rule,
                char: Some(char),
                pos,
                row,
                col,
            } => f.write_fmt(format_args!(
                "Unexpected char at position {pos} (row {row}, column {col}) while parsing rule '{current_rule}': '{char}'"
            )),
            ParseError::UnexpectedChar {
                current_rule,
                char: None,
                pos,
                row,
                col,
            } => f.write_fmt(format_args!(
                "Unexpected end of input at position {pos} (row {row}, column {col}) while parsing rule '{current_rule}'"
            )),
            ParseError::UnexpectedToken {
                current_rule,
                token,
                pos,
                row,
                col,
            } => f.write_fmt(format_args!(
                "Unexpected token at position {pos} (row {row}, column {col}) while parsing rule '{current_rule}': \"{token}\""
            )),
            ParseError::TransformerError {
                current_rule,
                pos,
                row,
                col,
                error,
            } => f.write_fmt(format_args!("Error while transforming rule '{current_rule}' at position {pos} (row {row}, column {col}): {error}")),
        }
    }
}

pub trait IntoParseError {
    fn into_error(self) -> ParseValue;
}

impl<E: Error + 'static> IntoParseError for E {
    fn into_error(self) -> ParseValue {
        // Make sure it's dynamic
        let error: Box<dyn Error> = Box::new(self);
        // Double boxed
        Box::new(error)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct LineInfo {
    pub pos: usize,
    pub line: usize,
    pub column: usize,
}

impl LineInfo {
    pub fn zero() -> Self {
        Self {
            pos: 0,
            line: 1,
            column: 1,
        }
    }
}

impl Default for LineInfo {
    fn default() -> Self {
        Self::zero()
    }
}

impl PartialOrd for LineInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.pos.cmp(&other.pos))
    }
}

impl Ord for LineInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos.cmp(&other.pos)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Span {
    pub start: LineInfo,
    pub end: LineInfo,
}

impl PartialOrd for Span {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Span {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.start.cmp(&other.start) {
            Ordering::Equal => {
                self.end.cmp(&other.end)
            }
            ord => ord
        }
    }
}
