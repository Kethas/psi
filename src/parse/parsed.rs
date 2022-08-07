use std::{
    collections::HashMap,
    fmt::{Debug, Display}, hash::Hash,
};

#[derive(Clone)]
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

pub enum ParseObject {
    ParseTree(ParseTree),
    Object(HashMap<String, ParseObject>),
    List(Vec<ParseObject>),
    String(String),
    Int(i64),
    Float(f64),
}

impl ParseObject {
    pub fn as_parse_tree(&self) -> Option<&ParseTree> {
        match self {
            ParseObject::ParseTree(tree) => Some(tree),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, ParseObject>> {
        match self {
            ParseObject::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Vec<ParseObject>> {
        match self {
            ParseObject::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            ParseObject::String(str) => Some(str),
            _ => None,
        }
    }


    pub fn as_int(&self) -> Option<i64> {
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
}

impl From<ParseTree> for ParseObject
{
    fn from(x: ParseTree) -> Self {
        ParseObject::ParseTree(x)
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
        ParseObject::String(x.to_owned())
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