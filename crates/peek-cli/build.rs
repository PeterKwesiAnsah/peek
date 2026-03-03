use std::env;
use std::path::PathBuf;

use clap::CommandFactory;

// Re-include the CLI definition so build.rs can call Cli::command().
// clap must also be listed under [build-dependencies].
include!("src/args.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // ── Man page ──────────────────────────────────────────────────────────────
    let cmd = Cli::command();
    let man = clap_mangen::Man::new(cmd.clone());
    let mut buf: Vec<u8> = Vec::new();
    man.render(&mut buf)?;
    let man_path = out_dir.join("peek.1");
    std::fs::write(&man_path, &buf)?;
    println!("cargo:warning=man page written to {}", man_path.display());

    // ── Shell completions ─────────────────────────────────────────────────────
    for shell in [
        clap_complete::Shell::Bash,
        clap_complete::Shell::Zsh,
        clap_complete::Shell::Fish,
    ] {
        let mut cmd = Cli::command();
        clap_complete::generate_to(shell, &mut cmd, "peek", &out_dir)?;
    }
    println!(
        "cargo:warning=shell completions written to {}",
        out_dir.display()
    );

    println!("cargo:rerun-if-changed=src/args.rs");
    Ok(())
}
