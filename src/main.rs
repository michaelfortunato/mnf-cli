use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use clap::{Args, Parser, Subcommand};
use log::*;

use std::io::Write;
use std::process::Command;

mod paths;

use paths::notes_dir;

#[derive(Parser)]
#[command(name="mnf", version, about, long_about = None)]
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
    #[clap(visible_aliases = ["d", "new", "n"])]
    Daily,
    /// Open your Math Notes File at ~/notes/MATH.typ
    Math,
    /// Manage your notes
    #[command(subcommand)]
    Note(FileOperation),
    /// Managee your gists
    #[command(subcommand)]
    Gist(FileOperation),
    /// Manage your scratch projects
    #[command(subcommand)]
    Scratch(FileOperation),
    /// Manage your courses projects
    #[command(subcommand)]
    Course(CourseCommand),
    /// Open your Random Principles File at ~/notes/RANDOM-PRINCIPLES.typ
    RP,
}

#[derive(Subcommand)]
enum CourseCommand {
    List,
    Add { course_name: String },
    Remove,
}

#[derive(Debug, Args)]
struct ListArgs {
    rel_path: Option<PathBuf>,
    /// lists all files
    #[arg(short, long)]
    all: bool,
    /// list files in long format
    #[arg(short = 'l')]
    long: bool,
}

#[derive(Subcommand)]
enum FileOperation {
    /// does testing things
    #[clap(alias = "ls")]
    List(ListArgs),
    /// Shows tree
    Tree {
        rel_path: Option<PathBuf>,
        #[arg(short, long)]
        /// the depth
        #[arg(short = 'L', long)]
        depth: Option<u8>,
    },
    /// Edit a file
    Edit {
        /// the filepath
        filepath: PathBuf,
    },
}

use anyhow::Error as AnyhowError;
use thiserror::Error;

use crate::paths::today_daily_note;
use crate::paths::{gists_dir, scratch_dir};
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

    if !note.exists() {
        OpenOptions::new().create(true).write(true).open(&note)?;
    }

    let file = note
        .file_name()
        .ok_or_else(|| AppError::from(anyhow::anyhow!("daily note path has no filename")))?;
    // Reuse your editor helper; open relative to the directory
    edit(dir, std::path::Path::new(file))
}

fn handle_fileop(base_dir: &Path, op: &FileOperation) -> Result<()> {
    match op {
        FileOperation::List(list_args) => list(base_dir, list_args),
        FileOperation::Tree { rel_path, depth } => tree(
            base_dir,
            rel_path.as_ref().unwrap_or(&PathBuf::from(".")),
            depth.clone(),
        ),

        FileOperation::Edit { filepath } => edit(base_dir, &filepath),
    }
}

fn handle_course_command(course_command: &CourseCommand) -> Result<()> {
    if true {
        println!("GOOD!");
    };
    Ok(())
}

fn install_logger() {
    env_logger::init();
}

fn main() -> Result<()> {
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
        Commands::Gist(op) => {
            let gists_dir = gists_dir()?;
            handle_fileop(&gists_dir, op)
        }
        Commands::Note(op) => {
            let note_dir = notes_dir()?;
            handle_fileop(&note_dir, op)
        }
        Commands::Course(course_command) => handle_course_command(course_command),
        Commands::Scratch(op) => {
            let scratch_dir = scratch_dir()?;
            handle_fileop(&scratch_dir, op)
        }
        Commands::RP => {
            let notes_dir = notes_dir()?;
            let op = FileOperation::Edit {
                filepath: PathBuf::from_str("RANDOM-PRINCIPLES.typ").unwrap(),
            };
            handle_fileop(&notes_dir, &op)
        }
        Commands::Math => {
            let notes_dir = notes_dir()?;
            let op = FileOperation::Edit {
                filepath: PathBuf::from_str("MATH.typ").unwrap(),
            };
            handle_fileop(&notes_dir, &op)
        }
        Commands::Daily => open_or_create_today_note(),
    };
    if let Err(e) = res {
        eprintln!("Application error: {}", e);
    }
    // Continued program logic goes here...
    Ok(())
}
