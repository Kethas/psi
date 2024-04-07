use crate::result::LineInfo;
use std::fmt::Display;
use std::path::Path;

pub trait Input<'a>: Clone + Display + 'a {
    fn next(&mut self) -> Option<char>;
    fn pos(&self) -> usize;
    fn row_col(&self) -> (usize, usize);

    fn filename(&self) -> Option<&str> {
        None
    }
    fn path(&self) -> Option<&Path> {
        None
    }

    fn line_info(&self) -> LineInfo {
        let pos = self.pos();
        let (line, column) = self.row_col();

        LineInfo { pos, line, column }
    }
}

pub trait IntoInput<'a>: 'a {
    type Input: Input<'a>;

    fn into_input(self) -> Self::Input;
}

pub mod chars;
pub mod vec;

#[cfg(feature = "file_input")]
pub mod file;
#[cfg(feature = "tcp_input")]
pub mod tcp;
