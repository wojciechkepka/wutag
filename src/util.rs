//! Utility functions used through this crate and by the main executable
use colored::{ColoredString, Colorize};
use globwalk::{DirEntry, GlobWalker, GlobWalkerBuilder};
use std::fmt::Display;
use std::path::{Path, PathBuf};

use crate::tags::Tag;
use crate::{Error, DEFAULT_MAX_DEPTH};

pub fn fmt_err<E: Display>(err: E) -> String {
    format!(
        "{}:\t{}",
        "ERROR".red().bold(),
        format!("{}", err).white().bold()
    )
}

pub fn fmt_ok<S: AsRef<str>>(msg: S) -> String {
    format!("{}:\t{}", "OK".green().bold(), msg.as_ref().white().bold())
}

pub fn fmt_path<P: AsRef<Path>>(path: P) -> String {
    format!("`{}`", path.as_ref().display().to_string().bold().blue())
}

pub fn fmt_tag(tag: &Tag) -> ColoredString {
    tag.name().bold().yellow()
}

/// Returns a GlobWalker instance with base path set to `base_path` and pattern to `pattern`. If
/// `recursive` is true the maximum depth is going to be [DEFAULT_MAX_DEPTH](DEFAULT_MAX_DEPTH)
/// otherwise `1` (only top level files).
pub fn glob_walker<S>(base_path: S, pattern: S, recursive: bool) -> Result<GlobWalker, Error>
where
    S: AsRef<str>,
{
    let mut builder = GlobWalkerBuilder::new(base_path.as_ref(), pattern.as_ref());

    if !recursive {
        builder = builder.max_depth(1);
    } else {
        builder = builder.max_depth(DEFAULT_MAX_DEPTH);
    }
    builder.build().map_err(Error::from)
}

/// Utility function that executes the function `f` on all directory entries that are Ok, by
/// default ignores all errors.
pub fn glob_ok<F>(
    pattern: &str,
    base_path: Option<PathBuf>,
    recursive: bool,
    f: F,
) -> Result<(), Error>
where
    F: Fn(&DirEntry),
{
    let base_path = if let Some(base_path) = base_path {
        base_path.to_string_lossy().to_string()
    } else {
        ".".to_string()
    };

    for entry in glob_walker(base_path.as_str(), pattern, recursive)? {
        if let Ok(entry) = entry {
            f(&entry);
        }
    }

    Ok(())
}
