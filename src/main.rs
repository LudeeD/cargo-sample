use anyhow::{Context, Result};
use clap::Parser;
use std::{fs, path::PathBuf, process::Command};

#[derive(Parser)]
#[command(name = "cargo-examples")]
#[command(about = "Generate a new project from a repository's examples")]
struct Cli {
    /// Repository URL to clone examples from
    #[arg(short, long)]
    repo: String,

    /// Name of the example to use as template
    #[arg(short, long)]
    example: String,

    /// Output directory name
    #[arg(short, long)]
    output: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create a temporary directory for cloning
    let temp_dir = tempfile::tempdir()?;

    // Clone the repository
    println!("Cloning repository...");
    Command::new("git")
        .args(["clone", &cli.repo, temp_dir.path().to_str().unwrap()])
        .output()
        .context("Failed to clone repository")?;

    // Find the examples directory
    let examples_dir = temp_dir.path().join("examples");
    if !examples_dir.exists() {
        anyhow::bail!("No examples directory found in repository");
    }

    // Find the specific example
    let example_dir = examples_dir.join(&cli.example);
    if !example_dir.exists() {
        anyhow::bail!("Example '{}' not found in examples directory", cli.example);
    }

    // Create output directory
    let output_dir = PathBuf::from(&cli.output);
    if output_dir.exists() {
        anyhow::bail!("Output directory already exists");
    }

    // Copy example to output directory
    println!("Generating project from example...");
    copy_dir_recursively(&example_dir, &output_dir)?;

    // Update project name in Cargo.toml if it exists
    if let Ok(cargo_toml) = fs::read_to_string(output_dir.join("Cargo.toml")) {
        let updated_toml = cargo_toml.replace(&cli.example, &cli.output);
        fs::write(output_dir.join("Cargo.toml"), updated_toml)?;
    }

    println!("Project generated successfully in '{}'", cli.output);
    Ok(())
}

fn copy_dir_recursively(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursively(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
