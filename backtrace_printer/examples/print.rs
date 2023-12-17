#![feature(error_generic_member_access)]
use std::{backtrace::Backtrace, fs::File, io};

use backtrace_printer::print_err;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("back")]
    Io(#[from] io::Error, Backtrace),
}

pub fn main() {
    let err: Error = File::open("does not exist").unwrap_err().into();
    print_err(&mut std::io::stdout(), err, &[], &[]).unwrap();
}
