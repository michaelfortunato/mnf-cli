use const_format::formatcp;

pub const GIT_SHA: &str = match option_env!("VERGEN_GIT_SHA") {
    Some(s) if !s.is_empty() => s,
    _ => "unknown",
};

pub const GIT_BRANCH: &str = match option_env!("VERGEN_GIT_BRANCH") {
    Some(s) if !s.is_empty() => s,
    _ => "unknown",
};

pub const GIT_DIRTY: &str = match option_env!("VERGEN_GIT_DIRTY") {
    Some(s) if !s.is_empty() => s, // typically "true" or "false"
    _ => "unknown",
};

#[allow(dead_code)]
pub const BUILD_TIMESTAMP_UTC: &str = match option_env!("BUILD_TIMESTAMP") {
    Some(s) if !s.is_empty() => s,
    _ => "unknown",
};

pub const BUILD_TIMESTAMP_LOCAL: &str = match option_env!("BUILD_TIMESTAMP_LOCAL") {
    Some(s) if !s.is_empty() => s,
    _ => "unknown",
};

pub const RUSTC_SEMVER: &str = match option_env!("VERGEN_RUSTC_SEMVER") {
    Some(s) if !s.is_empty() => s,
    _ => "unknown",
};

pub const TARGET_TRIPLE: &str = match option_env!("VERGEN_CARGO_TARGET_TRIPLE") {
    Some(s) if !s.is_empty() => s,
    _ => "unknown",
};

pub const SHORT_VERSION: &str = formatcp!(
    "{ver} ({branch}@{sha}, built-at={built})",
    ver = env!("CARGO_PKG_VERSION"),
    branch = GIT_BRANCH,
    sha = GIT_SHA,
    built = BUILD_TIMESTAMP_LOCAL,
);

pub const LONG_VERSION: &str = formatcp!(
    "{ver} ({branch}@{sha}, dirty={dirty}, built-at={built}, rustc={rustc}, target={target})",
    ver = env!("CARGO_PKG_VERSION"),
    branch = GIT_BRANCH,
    sha = GIT_SHA,
    dirty = GIT_DIRTY,
    built = BUILD_TIMESTAMP_LOCAL,
    rustc = RUSTC_SEMVER,
    target = TARGET_TRIPLE,
);
