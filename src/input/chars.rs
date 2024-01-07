use super::{Input, IntoInput};
use std::{fmt::Display, str::Chars};

#[derive(Clone)]
pub struct CharsInput<'a> {
    chars: Chars<'a>,
    pos: usize,
    col: usize,
    row: usize,
}

impl<'a> CharsInput<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            pos: 0,
            col: 1,
            row: 1,
        }
    }
}

impl<'a> Input<'a> for CharsInput<'a> {
    fn next(&mut self) -> Option<char> {
        self.chars.next().map(|c| {
            self.pos += 1;

            if c == '\n' {
                self.row += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }

            c
        })
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn row_col(&self) -> (usize, usize) {
        (self.row, self.col)
    }
}

impl<'a> Display for CharsInput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.chars.as_str())
    }
}

impl<'a> IntoInput<'a> for &'a str {
    type Input = CharsInput<'a>;

    fn into_input(self) -> Self::Input {
        Self::Input::new(self.chars())
    }
}

impl<'a> IntoInput<'a> for &'a String {
    type Input = CharsInput<'a>;

    fn into_input(self) -> Self::Input {
        Self::Input::new(self.chars())
    }
}

impl<'a> IntoInput<'a> for Chars<'a> {
    type Input = CharsInput<'a>;

    fn into_input(self) -> Self::Input {
        Self::Input::new(self)
    }
}
