use chrono;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::fs::OpenOptions;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::collections::hash_map::DefaultHasher;

use clap::{Args, CommandFactory, Parser, Subcommand};
use log::*;

use clap_complete::CompleteEnv;

use clap_complete::engine::{
    ArgValueCompleter, CompletionCandidate, PathCompleter, ValueCompleter,
};
use std::process::Command;

mod constants;
mod paths;

use clap_complete::Shell;
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
            "note" => notes_dir().ok(),
            "gist" => paths::gists_dir().ok(),
            "scratch" => paths::scratch_dir().ok(),
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
    #[clap(visible_aliases = ["d", "new", "n"])]
    Daily,
    /// Open your Math Notes File at ~/notes/MATH.typ
    Math,
    /// Manage your notes
    Note(NoteArgs),
    /// Manage your gists
    Gist(GistArgs),
    /// Manage your scratch projects
    #[command(subcommand)]
    Scratch(FileOperation),
    /// Manage your courses projects
    #[command(subcommand)]
    Course(CourseCommand),
    /// Generate completion for SHELL
    Completion {
        #[arg(value_enum)]
        shell: Shell,
        /// Override the command name used in the completion script
        #[arg(long = "bin-name", hide = true)]
        bin_name: Option<String>,
    },
    /// Open your Random Principles File at ~/notes/RANDOM-PRINCIPLES.typ
    RP,
}

#[derive(Debug, Args)]
struct NoteArgs {
    #[command(subcommand)]
    op: Option<NoteOperation>,
}

#[derive(Debug, Args)]
struct GistArgs {
    #[command(subcommand)]
    op: Option<GistOperation>,
}

#[derive(Subcommand)]
enum CourseCommand {
    List,
    Add { course_name: String },
    Remove,
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

#[derive(Subcommand)]
enum FileOperation {
    /// does testing things
    #[clap(alias = "ls")]
    List(ListArgs),
    /// Shows tree
    Tree {
        #[arg(add = ArgValueCompleter::new(complete_any_in_base))]
        rel_path: Option<PathBuf>,
        #[arg(short, long)]
        /// the depth
        #[arg(short = 'L', long)]
        depth: Option<u8>,
    },
    /// Edit a file
    #[clap(visible_aliases = ["add"])]
    Edit {
        /// the filepath
        #[arg(add = ArgValueCompleter::new(complete_file_in_base))]
        filepath: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum GistOperation {
    /// does testing things
    #[clap(alias = "ls")]
    List(ListArgs),
    /// Shows tree
    Tree {
        #[arg(add = ArgValueCompleter::new(complete_any_in_base))]
        rel_path: Option<PathBuf>,
        #[arg(short, long)]
        /// the depth
        #[arg(short = 'L', long)]
        depth: Option<u8>,
    },
    /// Edit a file
    #[clap(visible_aliases = ["add"])]
    Edit {
        /// the filepath
        #[arg(add = ArgValueCompleter::new(complete_file_in_base))]
        filepath: PathBuf,
    },
    /// Treat an unrecognized subcommand as a filename to edit
    #[command(external_subcommand)]
    External(Vec<OsString>),
}

#[derive(Debug, Subcommand)]
enum NoteOperation {
    /// does testing things
    #[clap(alias = "ls")]
    List(ListArgs),
    /// Shows tree
    Tree {
        #[arg(add = ArgValueCompleter::new(complete_any_in_base))]
        rel_path: Option<PathBuf>,
        #[arg(short, long)]
        /// the depth
        #[arg(short = 'L', long)]
        depth: Option<u8>,
    },
    /// Edit a file
    #[clap(visible_aliases = ["add"])]
    Edit {
        /// the filepath
        #[arg(add = ArgValueCompleter::new(complete_file_in_base))]
        filepath: PathBuf,
    },
    /// Treat an unrecognized subcommand as a filename to edit
    #[command(external_subcommand)]
    External(Vec<OsString>),
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

const CUTE_ADJECTIVES: &[&str] = &[
    "brave", "bright", "bubbly", "calm", "cheerful", "clever", "cozy", "curious", "dreamy",
    "eager", "fuzzy", "gentle", "happy", "jolly", "kind", "mellow", "nimble", "peppy", "plucky",
    "quiet", "shiny", "sleepy", "snappy", "sparkly", "sunny", "tiny", "witty", "zippy",
];

const CUTE_NOUNS: &[&str] = &[
    "acorn", "alpaca", "badger", "cactus", "comet", "corgi", "dolphin", "dragon", "fox", "gecko",
    "hedgehog", "koala", "lantern", "lemur", "marmot", "mushroom", "narwhal", "otter", "pebble",
    "penguin", "puffin", "raccoon", "salamander", "sparrow", "squid", "teacup", "turtle",
];

fn make_cute_typ_basename() -> String {
    let now = chrono::Local::now();
    let stamp = now.format("%Y-%m-%d-%H%M%S").to_string();

    let seed = now
        .timestamp_nanos_opt()
        .unwrap_or_else(|| now.timestamp_millis());
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let h = hasher.finish();

    let adj = CUTE_ADJECTIVES[(h as usize) % CUTE_ADJECTIVES.len()];
    let noun = CUTE_NOUNS[((h >> 32) as usize) % CUTE_NOUNS.len()];

    format!("{stamp}-{adj}-{noun}.typ")
}

fn slugify_stem(input: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in input.chars() {
        let ch = ch.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

fn sanitize_ext(input: &str) -> Option<String> {
    let cleaned: String = input
        .chars()
        .map(|c| c.to_ascii_lowercase())
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn sanitize_requested_filename(raw: &str) -> Result<PathBuf> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(AppError::from(anyhow::anyhow!("missing filename")));
    }
    if raw.contains('/') || raw.contains('\\') {
        return Err(AppError::from(anyhow::anyhow!(
            "filenames cannot contain path separators; use the `edit` subcommand for subdirectories"
        )));
    }

    let p = Path::new(raw);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(raw);
    let ext = p.extension().and_then(|e| e.to_str());

    let mut stem = slugify_stem(stem);
    if stem.is_empty() {
        stem = "untitled".to_string();
    }

    let ext = ext.and_then(sanitize_ext).unwrap_or_else(|| "typ".to_string());
    Ok(PathBuf::from(format!("{stem}.{ext}")))
}

fn pick_available_name(base_dir: &Path, suggested: &str) -> String {
    let suggested_path = base_dir.join(suggested);
    if !suggested_path.exists() {
        return suggested.to_string();
    }

    let (stem, ext) = suggested
        .rsplit_once('.')
        .map(|(s, e)| (s, e))
        .unwrap_or((suggested, "typ"));

    for i in 2..=1000u32 {
        let candidate = format!("{stem}-{i}.{ext}");
        if !base_dir.join(&candidate).exists() {
            return candidate;
        }
    }

    suggested.to_string()
}

fn open_new_typ_in_dir(base_dir: &Path) -> Result<()> {
    fs::create_dir_all(base_dir)?;
    let suggested = make_cute_typ_basename();
    let name = pick_available_name(base_dir, &suggested);
    edit(base_dir, Path::new(&name))
}

fn open_named_in_dir(base_dir: &Path, raw_name: &str) -> Result<()> {
    fs::create_dir_all(base_dir)?;
    let filepath = sanitize_requested_filename(raw_name)?;
    edit(base_dir, &filepath)
}

fn open_external_name_in_dir(base_dir: &Path, words: &[OsString]) -> Result<()> {
    let raw = words
        .iter()
        .map(|w| w.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    open_named_in_dir(base_dir, &raw)
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

fn handle_gistop(base_dir: &Path, op: &GistOperation) -> Result<()> {
    match op {
        GistOperation::List(list_args) => list(base_dir, list_args),
        GistOperation::Tree { rel_path, depth } => tree(
            base_dir,
            rel_path.as_ref().unwrap_or(&PathBuf::from(".")),
            depth.clone(),
        ),
        GistOperation::Edit { filepath } => edit(base_dir, &filepath),
        GistOperation::External(words) => open_external_name_in_dir(base_dir, words),
    }
}

fn handle_noteop(base_dir: &Path, op: &NoteOperation) -> Result<()> {
    match op {
        NoteOperation::List(list_args) => list(base_dir, list_args),
        NoteOperation::Tree { rel_path, depth } => tree(
            base_dir,
            rel_path.as_ref().unwrap_or(&PathBuf::from(".")),
            depth.clone(),
        ),
        NoteOperation::Edit { filepath } => edit(base_dir, &filepath),
        NoteOperation::External(words) => open_external_name_in_dir(base_dir, words),
    }
}

fn handle_course_command(_course_command: &CourseCommand) -> Result<()> {
    todo!("Implement course command");
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
        Commands::Gist(GistArgs { op }) => {
            let base_dir = gists_dir()?;
            match op {
                Some(op) => handle_gistop(&base_dir, op),
                None => open_new_typ_in_dir(&base_dir),
            }
        }
        Commands::Note(NoteArgs { op }) => {
            let base_dir = notes_dir()?;
            match op {
                Some(op) => handle_noteop(&base_dir, op),
                None => open_new_typ_in_dir(&base_dir),
            }
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
        Commands::Completion { shell, bin_name } => handle_completion(shell, bin_name.as_deref()),
    };
    if let Err(e) = res {
        eprintln!("Application error: {}", e);
    }
    // Continued program logic goes here...
    Ok(())
}
