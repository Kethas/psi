use super::{Input, IntoInput};
use std::fmt::Display;
#[derive(Clone)]
pub struct VecInput<'a> {
    chars: &'a Vec<char>,
    pos: usize,
    col: usize,
    row: usize,
}

impl<'a> VecInput<'a> {
    pub fn new(chars: &'a Vec<char>) -> Self {
        Self {
            chars,
            pos: 0,
            col: 1,
            row: 1,
        }
    }
}

impl<'a> Input<'a> for VecInput<'a> {
    fn next(&mut self) -> Option<char> {
        self.chars.get(self.pos).map(|&c| {
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

impl<'a> Display for VecInput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.chars.iter().cloned().collect::<String>())
    }
}

impl<'a> IntoInput<'a> for &'a Vec<char> {
    type Input = VecInput<'a>;

    fn into_input(self) -> Self::Input {
        Self::Input::new(self)
    }
}
