#![feature(error_generic_member_access)]

use std::io::Write;

use btparse::Frame;
use colored::Colorize;
use regex::Regex;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BacktraceParsing(#[from] btparse::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// filter an `std::backtrace::Backtrace` based on file name and function symbol block lists
/// NOTE: this uses a debug-output-based parser `btparse` to understand an `std` Backtrace.
///
/// # Errors
///
/// This function will return an error if parsing a backtrace fails
pub fn filter(
    bt: &std::backtrace::Backtrace,
    name_blocklist: &[Regex],
    file_blocklist: &[Regex],
) -> Result<Vec<Frame>> {
    let bt_parsed = btparse::deserialize(bt)?;
    Ok(bt_parsed
        .frames
        .into_iter()
        .filter(|x| {
            if matches!(&x.file, Some(f) if !file_blocklist.is_empty() && file_blocklist.iter().any(|l| {
              l.is_match(f)
            })) {
                return false;
            }

            if !name_blocklist.is_empty() && name_blocklist.iter().any(|l| l.is_match(&x.function)) {
                return false;
            }
            true
        })
        .collect::<Vec<_>>())
}

/// Print a backtrace given an error.
///
/// # Errors
///
/// This function will return an error if backtrace extraction fails
pub fn print_err<W: Write>(
    writer: &mut W,
    err: impl std::error::Error,
    name_blocklist: &[Regex],
    file_blocklist: &[Regex],
) -> Result<()> {
    if let Some(bt) = backtrace(&err) {
        let frames = filter(bt, name_blocklist, file_blocklist)?;
        print_frames(writer, &frames)?;
    }
    Ok(())
}

/// Print frames that were extracted and parsed from an `std::backtrace::Backtrace`
pub fn print_frames<W: Write>(writer: &mut W, frames: &Vec<Frame>) -> Result<()> {
    for frame in frames {
        writeln!(
            writer,
            "{}{}",
            frame.file.as_ref().map_or("<no file>", |file| file),
            frame
                .line
                .as_ref()
                .map_or(String::new(), |ln| format!(":{ln}")),
        )?;
        writeln!(writer, "\t{}\n", frame.function.yellow(),)?;
    }
    Ok(())
}

/// Extract a stacktrace from an error. Uses unstable features to extract
/// because apparently, there's no clear approved API for this (yet)
pub fn backtrace(err: &impl std::error::Error) -> Option<&std::backtrace::Backtrace> {
    std::error::request_ref::<std::backtrace::Backtrace>(err)
}

#[cfg(test)]
mod tests {
    use std::backtrace::Backtrace;
    use std::fs::File;
    use std::io::{self, BufWriter};

    use super::*;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("back")]
        Io(#[from] io::Error, Backtrace),
    }

    #[test]
    pub fn test_write() {
        std::env::set_var("RUST_BACKTRACE", "1");
        let mut buf = BufWriter::new(Vec::new());

        let err: Error = File::open("does not exist").unwrap_err().into();
        print_err(&mut buf, err, &[], &[]).unwrap();
        // Do writing here.

        let bytes = buf.into_inner().unwrap();
        let string = String::from_utf8_lossy(bytes.as_slice());
        let replacer = Regex::new("/rustc/.*?/").unwrap();
        let out = replacer.replace_all(&string, "");
        insta::assert_display_snapshot!(out);
    }
}
