use super::{Input, IntoInput};
use std::{
    cell::RefCell,
    fmt::Display,
    io::BufReader,
    marker::PhantomData,
    net::{SocketAddr, TcpStream},
    rc::Rc,
};

use utf8_chars::BufReadCharsExt;

struct TcpInputSource {
    source: BufReader<TcpStream>,
    buffer: Vec<char>,
}

impl TcpInputSource {
    fn new(source: TcpStream) -> Self {
        let buffer = Vec::new();

        Self {
            source: BufReader::new(source),
            buffer,
        }
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
pub struct TcpInput<'a> {
    address: SocketAddr,
    buffer: Rc<RefCell<TcpInputSource>>,
    pos: usize,
    row: usize,
    col: usize,

    _phantom: PhantomData<&'a TcpStream>,
}

impl<'a> TcpInput<'a> {
    pub fn new(tcp_stream: TcpStream) -> Self {
        let address = tcp_stream
            .local_addr()
            .expect("Couldn't get TcpStream IP address");
        let buffer = Rc::new(RefCell::new(TcpInputSource::new(tcp_stream)));
        Self {
            address,
            buffer,
            pos: 0,
            row: 0,
            col: 0,
            _phantom: PhantomData,
        }
    }
}

impl<'a> Input<'a> for TcpInput<'a> {
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

impl<'a> Display for TcpInput<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            address, row, col, ..
        } = self;

        f.write_fmt(format_args!("{address}:{row}:{col}"))
    }
}

impl<'a> IntoInput<'a> for TcpStream {
    type Input = TcpInput<'a>;

    fn into_input(self) -> Self::Input {
        Self::Input::new(self)
    }
}
