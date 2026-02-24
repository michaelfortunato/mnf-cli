use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Args, CommandFactory, Parser, Subcommand};

use clap_complete::CompleteEnv;

use clap_complete::engine::{
    ArgValueCompleter, CompletionCandidate, PathCompleter, ValueCompleter,
};
use clap_complete::Shell;
use std::process::Command;

mod constants;
mod paths;

use paths::notes_dir;

fn completion_words() -> Vec<OsString> {
    let mut args: Vec<OsString> = std::env::args_os().collect();
    let Some(double_dash) = args.iter().position(|a| a == "--") else {
        // Not being called from the completion engine
        args.drain(0..1); // Strip `argv[0]`
        return args;
    };
    args.drain(0..=double_dash);
    args
}

fn completion_base_dir() -> Option<PathBuf> {
    let words = completion_words();

    // `words` is the shell's view of the command line, including the command itself as the first
    // word.
    let mut iter = words.iter().skip(1);
    let mut skip_next = false;

    while let Some(word) = iter.next() {
        if skip_next {
            skip_next = false;
            continue;
        }

        if word == "--config" || word == "-c" {
            skip_next = true;
            continue;
        }

        let s = word.to_string_lossy();
        if s == "--debug" || (s.starts_with("-d") && s.chars().skip(1).all(|c| c == 'd')) {
            continue;
        }
        if s.starts_with('-') {
            continue;
        }

        return match s.as_ref() {
            "list" | "l" | "ls" | "tree" | "edit" | "add" => notes_dir().ok(),
            _ => None,
        };
    }

    None
}

fn complete_any_in_base(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(base_dir) = completion_base_dir() else {
        return PathCompleter::any().complete(current);
    };
    PathCompleter::any().current_dir(base_dir).complete(current)
}

fn complete_file_in_base(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(base_dir) = completion_base_dir() else {
        return PathCompleter::file().complete(current);
    };
    PathCompleter::file()
        .current_dir(base_dir)
        .complete(current)
}

#[derive(Parser)]
#[command(
    name = "mnf",
    about = "My Personal CLI",
    version = constants::SHORT_VERSION,
    long_version = constants::LONG_VERSION,
    propagate_version = true,

)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Open or create today's daily note in ~/notes/daily
    #[clap(visible_aliases = ["d"])]
    Daily,
    /// Open your Math Notes File at ~/notes/MATH.typ
    Math,
    /// Open your Random Principles File at ~/notes/RANDOM-PRINCIPLES.typ
    RP,
    /// List files in your notes directory
    #[clap(visible_aliases = ["l", "ls"])]
    List(ListArgs),
    /// Shows a tree of your notes directory
    Tree {
        #[arg(add = ArgValueCompleter::new(complete_any_in_base))]
        rel_path: Option<PathBuf>,
        /// the depth
        #[arg(short = 'L', long)]
        depth: Option<u8>,
    },
    /// Edit a note file
    #[clap(visible_aliases = ["add"])]
    Edit {
        /// the filepath
        #[arg(add = ArgValueCompleter::new(complete_file_in_base))]
        filepath: PathBuf,
    },
    /// Generate completion for SHELL
    Completion {
        #[arg(value_enum)]
        shell: Shell,
        /// Override the command name used in the completion script
        #[arg(long = "bin-name", hide = true)]
        bin_name: Option<String>,
    },
}

#[derive(Debug, Args)]
struct ListArgs {
    #[arg(add = ArgValueCompleter::new(complete_any_in_base))]
    rel_path: Option<PathBuf>,
    /// lists all files
    #[arg(short, long)]
    all: bool,
    /// list files in long format
    #[arg(short = 'l')]
    long: bool,
}

use anyhow::Error as AnyhowError;
use thiserror::Error;

use crate::paths::today_daily_note;
#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    // A catch-all variant that wraps any error from anyhow
    #[error(transparent)]
    Other(#[from] AnyhowError),
}

pub type Result<T> = std::result::Result<T, AppError>;

fn tree(base_dir: &Path, rel_path: &Path, depth: Option<u8>) -> Result<()> {
    let mut tree = Command::new("tree");
    tree.current_dir(base_dir).arg(rel_path);
    if let Some(depth) = depth {
        tree.arg("-L");
        tree.arg(depth.to_string());
    };
    tree.arg("-C");
    let output = tree.output()?;
    use std::io::Write;
    std::io::stdout().write_all(&output.stdout)?;
    Ok(())
}

fn list(
    base_dir: &Path,
    ListArgs {
        all,
        rel_path,
        long,
    }: &ListArgs,
) -> Result<()> {
    let mut ls = Command::new("ls");
    ls.current_dir(base_dir);
    if let Some(rel_path) = rel_path {
        ls.arg(rel_path);
    };
    if all.to_owned() {
        ls.arg("-a");
    }
    if long.to_owned() {
        ls.arg("-l");
    };
    ls.arg("-G");
    ls.arg("--color=always");
    let output = ls.output()?;
    std::io::stdout().write_all(&output.stdout)?;
    Ok(())
}

fn edit(base_dir: &Path, filepath: &Path) -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or(String::from("nvim"));
    let mut editor = Command::new(editor);
    editor.current_dir(base_dir).arg(filepath);
    let mut child = editor.spawn()?;
    let status = child.wait()?;
    if !status.success() {
        return Err(AppError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not edit file",
        )));
    }
    Ok(())
}

fn open_or_create_today_note() -> Result<()> {
    let note = today_daily_note()?;

    let dir = note
        .parent()
        .ok_or_else(|| AppError::from(anyhow::anyhow!("daily note path has no parent")))?;
    fs::create_dir_all(dir)?;

    match OpenOptions::new().write(true).create_new(true).open(&note) {
        Ok(mut f) => {
            // Typst heading at the top (pick your style)
            // As a heading:
            writeln!(f, "= {}", chrono::Local::now().format("%A, %B %e, %Y"))?;
            // Or as a comment:
            // writeln!(f, "// {}", chrono::Local::now().format("%A, %B %e, %Y"))?;
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => false,
        Err(e) => return Err(e.into()),
    };

    let file = note
        .file_name()
        .ok_or_else(|| AppError::from(anyhow::anyhow!("daily note path has no filename")))?;
    // Reuse your editor helper; open relative to the directory
    edit(dir, std::path::Path::new(file))
}

fn handle_completion(shell: &Shell, bin_name: Option<&str>) -> Result<()> {
    use clap_complete::env::{Bash, Elvish, EnvCompleter, Fish, Powershell, Zsh};

    let cmd = Cli::command();
    let name = cmd.get_name();
    let bin_name = bin_name.unwrap_or(name);

    let mut buf = Vec::new();
    match shell {
        Shell::Bash => Bash.write_registration("COMPLETE", name, bin_name, bin_name, &mut buf)?,
        Shell::Elvish => {
            Elvish.write_registration("COMPLETE", name, bin_name, bin_name, &mut buf)?
        }
        Shell::Fish => Fish.write_registration("COMPLETE", name, bin_name, bin_name, &mut buf)?,
        Shell::PowerShell => {
            Powershell.write_registration("COMPLETE", name, bin_name, bin_name, &mut buf)?
        }
        Shell::Zsh => Zsh.write_registration("COMPLETE", name, bin_name, bin_name, &mut buf)?,
        _ => {
            return Err(AppError::from(anyhow::anyhow!(
                "unsupported shell for dynamic completion script"
            )));
        }
    }
    std::io::stdout().write_all(&buf)?;
    Ok(())
}

fn install_logger() {
    env_logger::init();
}

fn main() -> Result<()> {
    // Shell completion integration that supports dynamic completions (`ArgValueCompleter`).
    // Must run before writing anything to stdout.
    CompleteEnv::with_factory(|| Cli::command()).complete();

    install_logger();
    let cli = Cli::parse();
    /* */
    // You can check the value provided by positional arguments, or option arguments

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    let command = &cli.command;
    let res: Result<()> = match command {
        Commands::RP => {
            let notes_dir = notes_dir()?;
            edit(&notes_dir, Path::new("RANDOM-PRINCIPLES.typ"))
        }
        Commands::Math => {
            let notes_dir = notes_dir()?;
            edit(&notes_dir, Path::new("MATH.typ"))
        }
        Commands::Daily => open_or_create_today_note(),
        Commands::List(list_args) => {
            let notes_dir = notes_dir()?;
            list(&notes_dir, list_args)
        }
        Commands::Tree { rel_path, depth } => {
            let notes_dir = notes_dir()?;
            tree(
                &notes_dir,
                rel_path.as_ref().unwrap_or(&PathBuf::from(".")),
                depth.clone(),
            )
        }
        Commands::Edit { filepath } => {
            let notes_dir = notes_dir()?;
            edit(&notes_dir, filepath)
        }
        Commands::Completion { shell, bin_name } => handle_completion(shell, bin_name.as_deref()),
    };
    if let Err(e) = res {
        eprintln!("Application error: {}", e);
    }
    // Continued program logic goes here...
    Ok(())
}
