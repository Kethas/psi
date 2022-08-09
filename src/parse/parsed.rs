use std::{
    collections::HashMap,
    fmt::{Debug, Display}, sync::Arc,
};

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

impl ParseObject {
    pub fn as_object(&self) -> Option<&HashMap<String, ParseObject>> {
        match self {
            ParseObject::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn into_object(self) -> Option<HashMap<String, ParseObject>> {
        match self {
            ParseObject::Object(obj) => Some(obj),
            _ => None
        }
    }

    pub fn as_list(&self) -> Option<&Vec<ParseObject>> {
        match self {
            ParseObject::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<Vec<ParseObject>> {
        match self {
            ParseObject::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            ParseObject::Str(str) => Some(str),
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            ParseObject::Str(str) => Some(str),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            ParseObject::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn into_int(&self) -> Option<i64> {
        match self {
            ParseObject::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            ParseObject::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn into_float(&self) -> Option<f64> {
        match self {
            ParseObject::Float(f) => Some(*f),
            _ => None,
        }
    }
}

impl From<ParseTree> for ParseObject
{
    fn from(x: ParseTree) -> Self {
        match x {
            ParseTree::End => Self::Nothing,
            ParseTree::Literal(x) => Self::Literal(x) ,
            ParseTree::Rule(n, v) => Self::Rule(n, v.into_iter().map(ParseObject::from).collect()),
        }
    }
}

impl<'a> From<HashMap<&'a str, ParseObject>> for ParseObject
{
    fn from(x: HashMap<&'a str, ParseObject>) -> Self {
        ParseObject::Object(x.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
    }
}

impl From<Vec<ParseObject>> for ParseObject
{
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
            },
            ParseObject::Object(map) => {
                f.debug_set().entries(map.iter().map(|(k, v)| format!("{k}: {v}"))).finish()
            },
            ParseObject::List(v) => {
                f.debug_list().entries(v.iter().map(|x| x.to_string())).finish()
            },
            ParseObject::Str(s) => f.write_fmt(format_args!("\"{s}\"")),
            ParseObject::Int(i) => f.write_fmt(format_args!("{i}")),
            ParseObject::Float(n) => f.write_fmt(format_args!("{n}")),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeBuffer {
    Literal(String),
    Rule(String, Vec<TreeBuffer>, RuleAction),
}

impl TreeBuffer {
    pub fn transfrom(self) -> ParseObject {
        match self {
            TreeBuffer::Literal(lit) => ParseObject::Literal(lit),
            TreeBuffer::Rule(name, inner, action) => {
                action.apply(ParseObject::Rule(name, inner.into_iter().map(|x| x.transfrom()).collect()))
            },
        }
    }
}
