use anyhow::{Context, Result};
use argh::FromArgs;
use inquire::Confirm;
use inquire::Select;
use std::{fs, path::PathBuf, process::Command};

#[derive(FromArgs)]
/// Always sample before you buy
struct Sample {
    // Cargo puts the command name invoked into the first argument,
    // so we don't want this argument to show up in the usage text.
    #[argh(positional, hidden_help)]
    _command: String,
    /// the repository to sample from
    #[argh(positional)]
    repo: String,

    /// the optional folder to output the results
    #[argh(positional)]
    output: Option<String>,
}

fn main() -> Result<()> {
    let demo = argh::from_env::<Sample>();

    // check if git is installed
    if Command::new("git").arg("--version").status().is_err() {
        return Err(anyhow::anyhow!("Git is not installed"));
    }

    let repo = demo.repo;

    let temp_dir = tempfile::tempdir()?;

    println!("Cloning repository: {}", repo);

    Command::new("git")
        .args(["clone", &repo, temp_dir.path().to_str().unwrap()])
        .output()
        .context("Failed to clone repository")?;

    // Find the examples directory
    let examples_dir = temp_dir.path().join("examples");

    if !examples_dir.exists() {
        return Err(anyhow::anyhow!("No examples directory found in repository"));
    }

    // Present the user with a list of examples
    let examples: Vec<String> = fs::read_dir(&examples_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name = entry.file_name();
            file_name.to_str()?.to_string().into()
        })
        .collect();

    let ans = Select::new("Which example to use?", examples).prompt();

    let ans = ans.context("Failed to select example")?;

    let path = examples_dir.join(&ans);

    // output is either the output folder or the current folder
    let output = match demo.output {
        Some(output) => PathBuf::from(output),
        None => PathBuf::from("."),
    };

    let output_string = output.to_str().unwrap_or_default();
    let confirmation_string = format!("Copy example to {}", output_string);
    let ans = Confirm::new(&confirmation_string)
        .with_default(false)
        .with_help_message("Please confirm to continue")
        .prompt();

    match ans {
        Ok(true) => {
            println!("Copying example to {}", output_string);
            copy_dir_recursively(&path, &output)?;
        }
        Ok(false) => {
            println!("That's too bad...");
            return Err(anyhow::anyhow!("User did not confirm"));
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Something went wrong: {}", e));
        }
    }

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
