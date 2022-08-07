use std::ops::Range;

use crate::utils::IntoBoxs;

pub trait Input: Clone {
    fn get(&self, index: usize) -> Option<char>;
    fn get_pos(&self) -> usize;
    fn set_pos(&mut self, pos: usize);
    fn slice(&self, range: Range<usize>) -> &[char];

    fn slice_to_string(&self, range: Range<usize>) -> String {
        self.slice(range).into_iter().copied().collect()
    }

    fn current(&self) -> Option<char> {
        self.get(self.get_pos())
    }
    fn next(&mut self) -> Option<char> {
        self.advance();
        self.current()
    }

    fn advance_by(&mut self, n: usize) {
        self.set_pos(self.get_pos() + n)
    }

    fn retreat_by(&mut self, n: usize) {
        self.set_pos(self.get_pos() - n)
    }

    fn advance(&mut self) {
        self.advance_by(1);
    }

    fn retreat(&mut self) {
        self.retreat_by(1);
    }
}

#[derive(Clone, Debug)]
pub struct CharsInput {
    inner: Box<[char]>,
    pos: usize,
}

impl Input for CharsInput {
    fn get(&self, index: usize) -> Option<char> {
        self.inner.get(index).copied()
    }

    fn get_pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }

    fn slice(&self, range: Range<usize>) -> &[char] {
        &self.inner[range]
    }
}

impl<I: IntoBoxs<char>> From<I> for CharsInput {
    fn from(chars: I) -> Self {
        Self {
            inner: chars.into_boxed_slice(),
            pos: 0,
        }
    }
}

mod new {
    use crate::utils::Boxs;
    use std::fmt::Debug;
    use std::{io::Read, ops::Range, sync::Arc};

    pub trait Source {
        fn get_char(&self, index: usize) -> Option<char>;
        fn slice_chars(&self, range: Range<usize>) -> Boxs<char>;
        fn name(&self) -> String;

        fn slice_to_string(&self, range: Range<usize>) -> String {
            self.slice_chars(range).into_iter().collect()
        }
    }

    impl Source for String {
        fn get_char(&self, index: usize) -> Option<char> {
            self.chars().nth(index)
        }

        fn slice_chars(&self, range: Range<usize>) -> Boxs<char> {
            let (start, end) = (range.start, range.end);
            self.chars().skip(start).take(end - start).collect()
        }

        fn name(&self) -> String {
            format!("<?>")
        }
    }

    impl Source for Boxs<char> {
        fn get_char(&self, index: usize) -> Option<char> {
            self.get(index).copied()
        }

        fn slice_chars(&self, range: Range<usize>) -> Boxs<char> {
            let (start, end) = (range.start, range.end);
            self.into_iter()
                .copied()
                .skip(start)
                .take(end - start)
                .collect()
        }

        fn name(&self) -> String {
            format!("<?>")
        }
    }

    #[derive(Clone)]
    pub struct Input<'a> {
        source: Arc<dyn Source + 'a>,
        pub pos: usize,
    }

    impl Debug for Input<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_fmt(format_args!("{}@{}", self.source.name(), self.pos))
        }
    }

    impl<'a> Input<'a> {
        pub fn new<S: Source + 'a>(source: impl Into<S>) -> Self {
            Self {
                source: Arc::new(source.into()),
                pos: 0,
            }
        }

        pub fn current_char(&self) -> Option<char> {
            self.get_char(self.pos)
        }

        
    }

    impl<'a> Source for Input<'a> {
        fn get_char(&self, index: usize) -> Option<char> {
            self.source.get_char(index)
        }

        fn slice_chars(&self, range: Range<usize>) -> Boxs<char> {
            self.source.slice_chars(range)
        }

        fn name(&self) -> String {
            format!("{:?}", self)
        }
    }
}
