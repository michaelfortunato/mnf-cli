use std::{env, path::PathBuf};

use anyhow::anyhow;
use chrono::Local;

use crate::{AppError, Result};

/// Resolve the current user's home directory.
pub fn home_dir() -> Result<PathBuf> {
    // Delegate to the `dirs` crate and convert None into `AppError`.
    dirs::home_dir().ok_or_else(|| AppError::from(anyhow!("home dir not found")))
}

/// Expand a leading `~` in a path-like string to the user's home directory.
/// - "~" becomes the home directory
/// - "~/foo/bar" becomes "$HOME/foo/bar"
/// - otherwise the input is returned as-is
pub fn expand_tilde<S: AsRef<str>>(s: S) -> Result<PathBuf> {
    let s = s.as_ref();
    if s == "~" {
        return home_dir();
    }
    if let Some(rest) = s.strip_prefix("~/") {
        return Ok(home_dir()?.join(rest));
    }
    Ok(PathBuf::from(s))
}

/// Read a dir from ENV or fall back to `$HOME/<default_rel>`. Expands "~" if present.
pub fn from_env_or_default(var: &str, default_rel: &str) -> Result<PathBuf> {
    if let Ok(val) = env::var(var) {
        expand_tilde(val)
    } else {
        Ok(home_dir()?.join(default_rel))
    }
}

/// Base notes directory. Defaults to `$HOME/notes` unless `MNF_NOTES_DIR` is set.
pub fn notes_dir() -> Result<PathBuf> {
    from_env_or_default("MNF_NOTES_DIR", "notes")
}

/// Gists directory. Defaults to `$HOME/notes/gists` unless `MNF_GIST_DIR` is set.
pub fn gists_dir() -> Result<PathBuf> {
    from_env_or_default("MNF_GIST_DIR", "notes/gist")
}

/// Scratch directory. Defaults to `$HOME/scratch` unless `MNF_SCRATCH_DIR` is set.
pub fn scratch_dir() -> Result<PathBuf> {
    from_env_or_default("MNF_SCRATCH_DIR", "scratch")
}

/// Daily notes directory: `<notes_dir>/daily`.
pub fn daily_dir() -> Result<PathBuf> {
    Ok(notes_dir()?.join("daily"))
}

/// Basename for today's daily note: `YYYY-MM-DD.typ`.
pub fn today_daily_basename() -> String {
    format!("{}.typ", Local::now().format("%Y-%m-%d"))
}

/// Full path to today's daily note: `<daily_dir>/YYYY-MM-DD.typ`.
pub fn today_daily_note() -> Result<PathBuf> {
    Ok(daily_dir()?.join(today_daily_basename()))
}
