fn main() -> Result<(), Box<dyn std::error::Error>> {
    let now = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={now}");
    let s = chrono::Local::now()
        .format("%Y-%m-%d %I:%M %p %Z")
        .to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP_LOCAL={s}");
    // Emit git + cargo + build metadata as compile-time env vars.
    // This writes lines like: cargo:rustc-env=VERGEN_GIT_SHA=...
    vergen_gitcl::Emitter::default()
        .add_instructions(&vergen_gitcl::GitclBuilder::all_git()?)?
        .add_instructions(&vergen_gitcl::CargoBuilder::all_cargo()?)?
        .emit()?;

    Ok(())
}
