use const_format::formatcp;

pub const SHORT_VERSION: &str = env!("CARGO_PKG_VERSION");

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

pub const BUILD_TIMESTAMP: &str = match option_env!("VERGEN_BUILD_TIMESTAMP") {
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

pub const LONG_VERSION: &str = formatcp!(
    "{ver} ({branch}@{sha}, dirty={dirty}, built={built}, rustc={rustc}, target={target})",
    ver = SHORT_VERSION,
    branch = GIT_BRANCH,
    sha = GIT_SHA,
    dirty = GIT_DIRTY,
    built = BUILD_TIMESTAMP,
    rustc = RUSTC_SEMVER,
    target = TARGET_TRIPLE,
);
