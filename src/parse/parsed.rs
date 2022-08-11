use eyre::ContextCompat;
use eyre::{eyre, Context};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    ops::Index,
    sync::Arc,
};
use eyre::Result;

use crate::grammar::RuleAction;

#[derive(Clone, PartialEq, Eq)]
pub enum ParseTree {
    End,
    Literal(String),
    Rule(String, Vec<ParseTree>),
}

impl Debug for ParseTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseTree::End => f.write_str("_"),
            ParseTree::Literal(l) => f.write_fmt(format_args!("\"{l}\"")),
            ParseTree::Rule(name, inner) => {
                f.write_fmt(format_args!("@{name}"))?;
                f.debug_list().entries(inner).finish()
            }
        }
    }
}

impl Display for ParseTree {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseTree::End => Result::Ok(()),
            ParseTree::Literal(l) => f.write_str(l),
            ParseTree::Rule(_, inner) => inner.iter().map(|x| Display::fmt(x, f)).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseObject {
    Nothing,
    Literal(String),
    Rule(String, Vec<ParseObject>),
    Object(HashMap<String, ParseObject>),
    List(Vec<ParseObject>),
    Str(String),
    Int(i64),
    Float(f64),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParseObjectType {
    Nothing,
    Literal,
    Rule,
    Object,
    List,
    Str,
    Int,
    Float
}

impl Display for ParseObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}

impl ParseObject {
    fn expected_type<T>(&self, expected: &'static str) -> eyre::Result<T> {
        Err(eyre!(
            "Object was not of type {expected}, but of type {}.",
            self.stringified_type()
        ))
    }


    fn get_type(&self) -> ParseObjectType {
        use ParseObjectType::*;
        match self {
            ParseObject::Nothing => Nothing,
            ParseObject::Literal(_) => Literal,
            ParseObject::Rule(_, _) => Rule,
            ParseObject::Object(_) => Object,
            ParseObject::List(_) => List,
            ParseObject::Str(_) => Str,
            ParseObject::Int(_) => Int,
            ParseObject::Float(_) => Float,
        }
    }

    pub fn stringified_type(&self) -> String {
        self.get_type().to_string()
    }

    pub fn index(&self, i: usize) -> eyre::Result<&ParseObject> {
        let ty = self.stringified_type();
        if ty == "list" {
            self.as_list()?
                .get(i)
                .wrap_err(format!("Index {i} out of bounds."))
        } else if ty == "rule" {
            self.as_rule()?
                .1
                .get(i)
                .wrap_err(format!("Index {i} out of bounds."))
        } else {
            Err(eyre!("Object was of type {ty}, not list or rule."))
        }
    }

    pub fn as_rule(&self) -> eyre::Result<(&str, &[ParseObject])> {
        match self {
            ParseObject::Rule(n, v) => Ok((n, v)),
            _ => self.expected_type("rule"),
        }
    }

    pub fn into_rule(self) -> eyre::Result<(String, Vec<ParseObject>)> {
        match self {
            ParseObject::Rule(n, v) => Ok((n, v)),
            _ => self.expected_type("rule"),
        }
    }

    pub fn as_literal(&self) -> eyre::Result<&str> {
        match self {
            ParseObject::Literal(l) => Ok(l),
            _ => self.expected_type("literal"),
        }
    }

    pub fn into_literal(self) -> eyre::Result<String> {
        match self {
            ParseObject::Literal(l) => Ok(l),
            _ => self.expected_type("literal"),
        }
    }

    pub fn as_object(&self) -> eyre::Result<&HashMap<String, ParseObject>> {
        match self {
            ParseObject::Object(obj) => Ok(obj),
            _ => self.expected_type("object"),
        }
    }

    pub fn into_object(self) -> eyre::Result<HashMap<String, ParseObject>> {
        match self {
            ParseObject::Object(obj) => Ok(obj),
            _ => self.expected_type("object"),
        }
    }

    pub fn as_list(&self) -> eyre::Result<&Vec<ParseObject>> {
        match self {
            ParseObject::List(list) => Ok(list),
            _ => self.expected_type("list"),
        }
    }

    pub fn into_list(self) -> eyre::Result<Vec<ParseObject>> {
        match self {
            ParseObject::List(list) => Ok(list),
            _ => self.expected_type("list"),
        }
    }

    pub fn as_string(&self) -> eyre::Result<&str> {
        match self {
            ParseObject::Str(str) => Ok(str),
            _ => self.expected_type("str"),
        }
    }

    pub fn into_string(self) -> eyre::Result<String> {
        match self {
            ParseObject::Str(str) => Ok(str),
            _ => self.expected_type("str"),
        }
    }

    pub fn as_int(&self) -> eyre::Result<i64> {
        match self {
            ParseObject::Int(i) => Ok(*i),
            _ => self.expected_type("int"),
        }
    }

    pub fn into_int(self) -> eyre::Result<i64> {
        match self {
            ParseObject::Int(i) => Ok(i),
            _ => self.expected_type("int"),
        }
    }

    pub fn as_float(&self) -> eyre::Result<f64> {
        match self {
            ParseObject::Float(f) => Ok(*f),
            _ => self.expected_type("float"),
        }
    }

    pub fn into_float(self) -> eyre::Result<f64> {
        match self {
            ParseObject::Float(f) => Ok(f),
            _ => self.expected_type("float"),
        }
    }
}

impl From<ParseTree> for ParseObject {
    fn from(x: ParseTree) -> Self {
        match x {
            ParseTree::End => Self::Nothing,
            ParseTree::Literal(x) => Self::Literal(x),
            ParseTree::Rule(n, v) => Self::Rule(n, v.into_iter().map(ParseObject::from).collect()),
        }
    }
}

impl<'a> From<HashMap<&'a str, ParseObject>> for ParseObject {
    fn from(x: HashMap<&'a str, ParseObject>) -> Self {
        ParseObject::Object(x.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }
}

impl From<Vec<ParseObject>> for ParseObject {
    fn from(x: Vec<ParseObject>) -> Self {
        ParseObject::List(x)
    }
}

impl<'a> From<&'a str> for ParseObject {
    fn from(x: &'a str) -> Self {
        ParseObject::Str(x.to_owned())
    }
}

impl From<i64> for ParseObject {
    fn from(x: i64) -> Self {
        ParseObject::Int(x)
    }
}

impl From<f64> for ParseObject {
    fn from(x: f64) -> Self {
        ParseObject::Float(x)
    }
}

impl Display for ParseObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseObject::Nothing => f.write_str("nothing"),
            ParseObject::Literal(lit) => f.write_str(lit),
            ParseObject::Rule(_, inner) => {
                for obj in inner {
                    Display::fmt(obj, f)?;
                }

                Ok(())
            }
            ParseObject::Object(map) => f
                .debug_set()
                .entries(map.iter().map(|(k, v)| format!("{k}: {v}")))
                .finish(),
            ParseObject::List(v) => f
                .debug_list()
                .entries(v.iter().map(|x| x.to_string()))
                .finish(),
            ParseObject::Str(s) => f.write_fmt(format_args!("\"{s}\"")),
            ParseObject::Int(i) => f.write_fmt(format_args!("{i}")),
            ParseObject::Float(n) => f.write_fmt(format_args!("{n}")),
        }
    }
}

impl Index<usize> for ParseObject {
    type Output = ParseObject;

    fn index(&self, index: usize) -> &Self::Output {
        self.index(index).unwrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeBuffer {
    Literal(String),
    Rule(String, Vec<TreeBuffer>, RuleAction),
}

impl TreeBuffer {
    pub fn transfrom(self) -> Result<ParseObject> {
        match self {
            TreeBuffer::Literal(lit) => Ok(ParseObject::Literal(lit)),
            TreeBuffer::Rule(name, inner, action) if !action.id.is_nil() => {
                let transformed_inner = inner
                    .into_iter()
                    .map(|x| x.transfrom())
                    .collect::<Result<Vec<_>>>()?;

                println!("transformed inners for {name}/{action:?}");
                for (i, o) in transformed_inner.iter().enumerate() {
                    println!("[{i}]  {o:?}");
                }

                let out = action.apply(ParseObject::Rule(name.clone(), transformed_inner))?;

                println!("transformed {name}/{action:?}\n\tinto {out:?}");

                Ok(out)
            }
            TreeBuffer::Rule(name, inner, action) => {
                let transformed_inner = inner
                    .into_iter()
                    .map(|x| x.transfrom())
                    .collect::<Result<Vec<_>>>()?;

                action.apply(ParseObject::Rule(name.clone(), transformed_inner))
            }
        }
    }
}
