use super::{Input, IntoInput};
use std::{
    cell::RefCell,
    fmt::Display,
    fs::{File, OpenOptions},
    io::BufReader,
    path::{Path, PathBuf},
    rc::Rc,
};

use utf8_chars::BufReadCharsExt;

struct FileInputSource {
    source: BufReader<File>,
    buffer: Vec<char>,
}

impl FileInputSource {
    fn new(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let metadata = std::fs::metadata(&path)?;

        let source = OpenOptions::new().read(true).write(false).open(path)?;
        let buffer = Vec::with_capacity(metadata.len() as usize / 4);

        Ok(Self {
            source: BufReader::new(source),
            buffer,
        })
    }

    fn get(&mut self, pos: usize) -> Option<char> {
        while pos >= self.buffer.len() {
            let ch = self.source.read_char().ok()??;

            self.buffer.push(ch);
        }

        self.buffer.get(pos).cloned()
    }
}

#[derive(Clone)]
pub struct FileInput<'a> {
    file: &'a Path,
    buffer: Rc<RefCell<FileInputSource>>,
    pos: usize,
    row: usize,
    col: usize,
}

impl<'a> FileInput<'a> {
    pub fn new(file: &'a Path) -> Self {
        let buffer = Rc::new(RefCell::new(
            FileInputSource::new(file).expect("Could not create FileInput"),
        ));
        Self {
            file,
            buffer,
            pos: 0,
            row: 0,
            col: 0,
        }
    }
}

impl<'a> Input<'a> for FileInput<'a> {
    fn next(&mut self) -> Option<char> {
        self.buffer.borrow_mut().get(self.pos).map(|c| {
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

impl<'a> Display for FileInput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { file, row, col, .. } = self;

        f.write_fmt(format_args!("{}:{row}:{col}", file.display()))
    }
}

impl<'a> IntoInput<'a> for &'a Path {
    type Input = FileInput<'a>;

    fn into_input(self) -> Self::Input {
        Self::Input::new(self)
    }
}


impl<'a> IntoInput<'a> for &'a PathBuf {
    type Input = FileInput<'a>;

    fn into_input(self) -> Self::Input {
        Self::Input::new(self)
    }
}

