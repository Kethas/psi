use std::str::Chars;


#[derive(Clone)]
pub struct Input<'a> {
    chars: Chars<'a>,
    pos: usize,
    col: usize,
    row: usize,
}

impl<'a> Input<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Self {
            chars,
            pos: 0,
            col: 1,
            row: 1,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<char> {
        self.chars.next().map(|c| {
            self.pos += 1;

            if c == '\n' {
                self.row += 1;
                self.col = 1;
            }

            c
        })
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row, self.col)
    }
}

impl<'a> From<&'a str> for Input<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value.chars())
    }
}

impl<'a> From<&'a String> for Input<'a> {
    fn from(value: &'a String) -> Self {
        Self::new(value.chars())
    }
}

impl<'a> From<Chars<'a>> for Input<'a> {
    fn from(value: Chars<'a>) -> Self {
        Self::new(value)
    }
}
