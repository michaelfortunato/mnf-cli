use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

use std::io::Write;
use std::process::Command;

#[derive(Parser)]
#[command(version, about, long_about = None)]
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
    #[command(subcommand)]
    Gist(FileOperation),
    #[command(subcommand)]
    Note(FileOperation),
    #[command(subcommand)]
    Course(CourseCommand),
}

#[derive(Subcommand)]
enum CourseCommand {
    List,
    Add { course_name: String },
    Remove,
}

#[derive(Subcommand)]
enum FileOperation {
    /// does testing things
    List {
        rel_path: Option<PathBuf>,
        /// lists all files
        #[arg(short, long)]
        all: bool,
    },
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
    let output = tree.output()?;
    use std::io::Write;
    std::io::stdout().write_all(&output.stdout)?;
    Ok(())
}

fn list(base_dir: &Path, rel_path: &Path, all: bool) -> Result<()> {
    let mut ls = Command::new("ls");
    ls.current_dir(base_dir).arg(rel_path);
    if all {
        ls.arg("-a");
    }
    let output = ls.output()?;
    std::io::stdout().write_all(&output.stdout)?;
    Ok(())
}

fn edit(base_dir: &Path, filepath: &Path) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or(String::from("nvim"));
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

fn handle_fileop(base_dir: &Path, op: &FileOperation) -> Result<()> {
    match op {
        FileOperation::List { rel_path, all } => list(
            base_dir,
            rel_path.as_ref().unwrap_or(&PathBuf::from(".")),
            all.clone(),
        ),
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

fn main() {
    let cli = Cli::parse();
    /* */
    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    let res: Result<()> = match &cli.command {
        Commands::Gist(op) => {
            let base_dir = PathBuf::from(
                std::env::var("MNF_GIST_DIR").unwrap_or(String::from("~/notes/gists")),
            );
            handle_fileop(&base_dir, op)
        }
        Commands::Note(op) => {
            let base_dir =
                PathBuf::from(std::env::var("MNF_NOTES_DIR").unwrap_or(String::from("~/notes")));
            handle_fileop(&base_dir, op)
        }
        Commands::Course(course_command) => handle_course_command(course_command),
    };
    if let Err(e) = res {
        eprintln!("Application error: {}", e);
        std::process::exit(1);
    }

    // Continued program logic goes here...
}
