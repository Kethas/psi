use super::*;
use std::{collections::HashMap, error::Error, fmt::Display};

#[derive(Debug)]
pub enum XmlParseError {
    IllegalTagName { tag: String },
    NonMatchingTagNames { start_tag: String, end_tag: String },
}

impl Error for XmlParseError {}

impl Display for XmlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XmlParseError::IllegalTagName { tag } => {
                f.write_fmt(format_args!("Illegal tag name: <{tag}>"))
            }
            XmlParseError::NonMatchingTagNames { start_tag, end_tag } => f.write_fmt(format_args!(
                "Opening and closing tags don't match: <{start_tag}> </{end_tag}>"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Xml {
    Node(String, HashMap<String, String>, Vec<Xml>),
    Text(String),
}

impl Display for Xml {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            match self {
                Xml::Node(tag_name, attributes, children) => {
                    let attributes = {
                        let mut s = String::new();

                        for (k, v) in attributes {
                            s.push_str(&format!(
                                "{k}='{}' ",
                                v.replace('&', "&amp;").replace('\'', "&apos;")
                            ));
                        }

                        s
                    };

                    if children.is_empty() {
                        f.write_fmt(format_args!("<{tag_name} {attributes}/>"))
                    } else {
                        f.write_fmt(format_args!("<{tag_name} {attributes}>"))?;
                        for child in children {
                            let str = child.to_string().replace('\n', "\n\t");
                            f.write_fmt(format_args!("\t{str}"))?;
                        }
                        f.write_fmt(format_args!("</{tag_name}>"))
                    }
                }
                Xml::Text(text) => f.write_str(text),
            }
        } else {
            match self {
                Xml::Node(tag_name, attributes, children) => {
                    let attributes = {
                        let mut s = String::new();

                        for (k, v) in attributes {
                            s.push_str(&format!(
                                "{k}='{}' ",
                                v.replace('&', "&amp;").replace('\'', "&apos;")
                            ));
                        }

                        s
                    };

                    if children.is_empty() {
                        f.write_fmt(format_args!("<{tag_name} {attributes}/>"))
                    } else {
                        f.write_fmt(format_args!("<{tag_name} {attributes}>"))?;
                        for child in children {
                            Display::fmt(child, f)?;
                        }
                        f.write_fmt(format_args!("</{tag_name}>"))
                    }
                }
                Xml::Text(text) => f.write_str(text),
            }
        }
    }
}

declare_rules! {
    pub XmlRules {
        #[import (rules::Whitespace) as ws]
        #[import (rules::Identifier) as id]

        start {
            ((ws::ws_ml) xml (ws::ws_ml)) => |v| v(1);
        }

        xml {
            (node)
            (text) => |v| Xml::Text(*v(0).downcast::<String>().unwrap()).into_value();
        }

        text {
            ((! "<" "&")) => |v| v(0).downcast::<Token>().unwrap().to_string().into_value();
            (escape) => |v| [*v(0).downcast::<char>().unwrap()].into_iter().collect::<String>().into_value();
            (text escape) => |v| {
                let mut text = v(0).downcast::<String>().unwrap();
                text.push(*v(1).downcast::<char>().unwrap());

                text
            };
            (text (! "<" "&")) => |v| {
                let mut text = v(0).downcast::<String>().unwrap();
                text.push_str(v(1).downcast::<Token>().unwrap().as_str());

                text
            };
        }

        escape {
            ("&lt;") => |_| '<'.into_value();
            ("&gt;") => |_| '>'.into_value();
            ("&quot;") => |_| '"'.into_value();
            ("&apos;") => |_| '\''.into_value();
            ("&amp;") => |_| '&'.into_value();
        }

        node {
            ("<" (id::identifier) (ws::ws_ml) attributes (ws::ws_ml) ">"
             node_inner
             "</" (id::identifier) (ws::ws_ml) ">" )
             => |v| {
                let tag_name = *v(1).downcast::<String>().unwrap();
                let tag_name_end = *v(8).downcast::<String>().unwrap();

                if tag_name != tag_name_end {
                    return XmlParseError::NonMatchingTagNames { start_tag: tag_name, end_tag: tag_name_end }.into_error()
                }

                if tag_name.eq_ignore_ascii_case("xml") {
                    return XmlParseError::IllegalTagName { tag: tag_name }.into_error()
                }

                let attributes = *v(3).downcast::<HashMap<String, String>>().unwrap();
                let children = *v(6).downcast::<Vec<Xml>>().unwrap();


                Xml::Node(tag_name, attributes, children).into_value()
             };

             ("<" (id::identifier) (ws::ws_ml) attributes (ws::ws_ml) ">"
             "</" (id::identifier) (ws::ws_ml) ">" )
             => |v| {
                let tag_name = *v(1).downcast::<String>().unwrap();
                let tag_name_end = *v(7).downcast::<String>().unwrap();

                if tag_name != tag_name_end {
                    return XmlParseError::NonMatchingTagNames { start_tag: tag_name, end_tag: tag_name_end }.into_error()
                }

                if tag_name.eq_ignore_ascii_case("xml") {
                    return XmlParseError::IllegalTagName { tag: tag_name }.into_error()
                }

                let attributes = *v(3).downcast::<HashMap<String, String>>().unwrap();


                Xml::Node(tag_name, attributes, Vec::new()).into_value()
             };


             ("<" (id::identifier) (ws::ws_ml) attributes (ws::ws_ml) "/>")

             => |v| {
                let tag_name = *v(1).downcast::<String>().unwrap();

                if tag_name.eq_ignore_ascii_case("xml") {
                    return XmlParseError::IllegalTagName { tag: tag_name }.into_error()
                }

                let attributes = *v(3).downcast::<HashMap<String, String>>().unwrap();


                Xml::Node(tag_name, attributes, Vec::new()).into_value()
             };
        }

        node_inner {
            (xml) => |v| vec![*v(0).downcast::<Xml>().unwrap()].into_value();
            (node_inner xml) => |v| {
                let mut vec = v(0).downcast::<Vec<Xml>>().unwrap();
                vec.push(*v(1).downcast::<Xml>().unwrap());

                vec
            };
        }

        attributes {
            () => |_| HashMap::<String, String>::new().into_value();
            (attributes (ws::ws_ml) attribute) => |v| {
                let mut map = v(0).downcast::<HashMap<String, String>>().unwrap();
                let (k, v) = *v(2).downcast::<(String, String)>().unwrap();
                map.insert(k, v);

                map
            };
        }

        attribute {
            ((id::identifier) (ws::ws_ml) "=" (ws::ws_ml) (id::identifier))
                => |v| (*v(0).downcast::<String>().unwrap(), *v(4).downcast::<String>().unwrap()).into_value();
            ((id::identifier) (ws::ws_ml) "=" (ws::ws_ml) (string))
                => |v| (*v(0).downcast::<String>().unwrap(), *v(4).downcast::<String>().unwrap()).into_value();

        }


        string {
            ("\"" d_string_inner "\"") => |v| v(1);
            ("'" s_string_inner "'") => |v| v(1);
        }

        d_string_inner {
            () => |_| String::new().into_value();
            (d_string_inner escape) => |v| {
                let mut str = v(0).downcast::<String>().unwrap();
                str.push(*v(1).downcast::<char>().unwrap());

                str
            };
            (d_string_inner (! "\"" "&"))  => |v| {
                let mut str = v(0).downcast::<String>().unwrap();
                str.push_str(v(1).downcast::<Token>().unwrap().as_str());

                str
            };
        }

        s_string_inner {
            () => |_| String::new().into_value();
            (s_string_inner escape) => |v| {
                let mut str = v(0).downcast::<String>().unwrap();
                str.push(*v(1).downcast::<char>().unwrap());

                str
            };
            (s_string_inner (! "'" "&"))  => |v| {
                let mut str = v(0).downcast::<String>().unwrap();
                str.push_str(v(1).downcast::<Token>().unwrap().as_str());

                str
            };
        }
    }
}
