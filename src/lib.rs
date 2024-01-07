pub mod prelude {
    pub use super::result::{IntoParseValue as _, ParseError, ParseValue, Token};

    pub use super::rule::{Rule, Rules};

    pub use super::{declare_rules, rules};
}

pub mod result;

pub mod input;

pub mod rule;

pub mod macros;

pub mod rules;

#[cfg(test)]
mod tests;
