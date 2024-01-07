use std::{
    cell::RefCell,
    fmt::Display,
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
    rc::Rc,
};

use utf8_chars::BufReadCharsExt;

pub trait Input<'a>: Clone + Display + 'a {
    fn next(&mut self) -> Option<char>;
    fn pos(&self) -> usize;
    fn row_col(&self) -> (usize, usize);
}

pub trait IntoInput<'a>: 'a {
    type Input: Input<'a>;

    fn into_input(self) -> Self::Input;
}

pub mod chars;
pub mod vec;

pub mod file;
pub mod tcp;
