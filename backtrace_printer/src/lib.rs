use std::{backtrace::Backtrace, io::Write};

use btparse_stable::Frame;
use colored::Colorize;
use regex::Regex;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    BacktraceParsing(#[from] btparse_stable::Error),

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
    let bt_parsed = btparse_stable::deserialize(bt)?;
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

/// Print a backtrace
///
/// # Errors
///
/// This function will return an error if backtrace extraction fails
pub fn print_backtrace<W: Write>(
    writer: &mut W,
    bt: &Backtrace,
    name_blocklist: &[Regex],
    file_blocklist: &[Regex],
) -> Result<()> {
    let frames = filter(bt, name_blocklist, file_blocklist)?;
    print_frames(writer, &frames)?;
    Ok(())
}
/// Print frames that were extracted and parsed from an `std::backtrace::Backtrace`
///
/// # Errors
/// Returns error if IO fails
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
