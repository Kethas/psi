use crate::{self as psi_parser};
use psi_parser::prelude::*;
use std::io::Write;

pub fn init() {
    let _ = env_logger::builder()
        .is_test(true)
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .try_init();
}
mod basic;

mod recurse;

mod rules;
