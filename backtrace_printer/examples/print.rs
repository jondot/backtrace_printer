use std::{
    backtrace::{self, Backtrace},
    env,
};

use backtrace_printer::print_backtrace;

fn bar() -> Backtrace {
    backtrace::Backtrace::capture()
}
fn foo() -> Backtrace {
    bar()
}

pub fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let bt = foo();
    print_backtrace(&mut std::io::stdout(), &bt, &[], &[]).unwrap();
}
