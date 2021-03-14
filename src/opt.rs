//! Options used by the main executable
use std::{path::PathBuf, str::FromStr};

use clap::Clap;
use wutag::Error;

pub const APP_NAME: &str = "wutag";
pub const APP_VERSION: &str = "0.1.2";
pub const APP_AUTHOR: &str = "Wojciech Kępka <wojciech@wkepka.dev>";
pub const APP_ABOUT: &str = "Tool to tag and mange tags of files.";

#[derive(Clap)]
#[clap(version = APP_VERSION, author = APP_AUTHOR, about = APP_ABOUT)]
pub struct WutagOpts {
    #[clap(short, long)]
    /// When this parameter is specified the program will look for files starting from provided
    /// path, otherwise defaults to current directory.
    pub dir: Option<PathBuf>,
    #[clap(long, short)]
    /// Increase maximum recursion depth of filesystem traversal to 512. Default is 2. What this
    /// means is by deafult all subcommands that take a pattern as input will match files only 2
    /// levels deep starting from the base directory which is current working directory if `dir` is
    /// not specified.
    pub recursive: bool,
    /// If passed the output won't be colored
    #[clap(long, short)]
    pub no_color: bool,
    #[clap(subcommand)]
    pub cmd: WutagCmd,
}

#[derive(Clap)]
pub struct ListOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    #[clap(long)]
    /// Whether to show files with no tags
    pub show_missing: bool,
    /// Whether to print details like tag timestamp
    #[clap(long)]
    pub details: bool,
}

#[derive(Clap)]
pub struct SetOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    #[clap(required = true)]
    pub tags: Vec<String>,
}

#[derive(Clap)]
pub struct RmOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    pub tags: Vec<String>,
}

#[derive(Clap)]
pub struct ClearOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    #[clap(short, long)]
    /// If specified output and errors will be displayed
    pub verbose: bool,
}
#[derive(Clap)]
pub struct SearchOpts {
    #[clap(required = true)]
    pub tags: Vec<String>,
    #[clap(long)]
    /// If provided output will be raw so that it can be easily piped to other commands
    pub raw: bool,
}

#[derive(Clap)]
pub struct CpOpts {
    /// Path to the file from which to copy tags from
    pub input_path: PathBuf,
    /// A glob pattern like '*.png'.
    pub pattern: String,
}

#[derive(Clap)]
pub struct EditOpts {
    /// A glob pattern like '*.png'.
    pub pattern: String,
    /// The tag to edit
    pub tag: String,
    #[clap(long, short)]
    /// Set the color of the tag to the specified color. Accepted values are hex colors like
    /// `0x000000` or `#1F1F1F` or just plain `ff000a`. The colors are case insensitive meaning
    /// `1f1f1f` is equivalent to `1F1F1F`.
    pub color: String,
}

#[derive(Clap)]
pub enum Shell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl FromStr for Shell {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "bash" => Ok(Shell::Bash),
            "elvish" => Ok(Shell::Elvish),
            "fish" => Ok(Shell::Fish),
            "powershell" => Ok(Shell::PowerShell),
            "zsh" => Ok(Shell::Zsh),
            _ => Err(Error::InvalidShell(s.to_string())),
        }
    }
}

#[derive(Clap)]
pub struct CompletionsOpts {
    /// A shell for which to print completions. Available shells are: bash, elvish, fish,
    /// powershell, zsh
    pub shell: Shell,
}

#[derive(Clap)]
pub enum WutagCmd {
    /// Lists all tags of the files that match the provided pattern.
    List(ListOpts),
    /// Tags the files that match the given pattern with specified tags.
    Set(SetOpts),
    /// Removes the specified tags of the files that match the provided pattern.
    Rm(RmOpts),
    /// Clears all tags of the files that match the provided pattern.
    Clear(ClearOpts),
    /// Searches for files that have all of the provided `tags`.
    Search(SearchOpts),
    /// Copies tags from the specified file to files that match a pattern.
    Cp(CpOpts),
    /// Edits the tag of files that match the provided pattern.
    Edit(EditOpts),
    /// Prints completions for the specified shell to stdout.
    PrintCompletions(CompletionsOpts),
}
