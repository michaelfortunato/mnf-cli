fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Emit git + cargo + build metadata as compile-time env vars.
    // This writes lines like: cargo:rustc-env=VERGEN_GIT_SHA=...
    //
    vergen_gitcl::Emitter::default()
        .add_instructions(&vergen_gitcl::GitclBuilder::all_git()?)?
        .add_instructions(&vergen_gitcl::CargoBuilder::all_cargo()?)?
        .emit()?;

    Ok(())
}
