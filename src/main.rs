use anyhow::{anyhow, Context, Result};
use argh::FromArgs;
use cargo_metadata::Metadata;
use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;
use crates_io_api::SyncClient;
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
    let args = argh::from_env::<Sample>();

    let home_dir = match args.output {
        Some(output) => PathBuf::from(output),
        None => PathBuf::from("."),
    };

    if !home_dir.exists() {
        fs::create_dir_all(&home_dir)?;
    }

    std::env::set_current_dir(&home_dir)?;

    println!("Current directory: {:?}", std::env::current_dir()?);

    // add the dependency we want to the project

    let cargo_add = Command::new("cargo").arg("add").arg(&args.repo).status()?;

    if cargo_add.success() {
        println!("Added dependency: {}", args.repo);
    } else {
        println!("Probably not a Cargo project... creating a new one and adding the dependency");
        Command::new("cargo").arg("init").status()?;
        Command::new("cargo").arg("add").arg(&args.repo).status()?;
    }

    let metadata = MetadataCommand::new().exec()?;

    // filter the dependency we want
    let deps: Vec<&Package> = metadata
        .packages
        .iter()
        .filter(|package| package.name == args.repo)
        .collect();

    assert!(deps.len() == 1, "Something is cooked");

    let package = deps[0];
    let manifest_path = PathBuf::from(package.manifest_path.clone());
    let manifest_dir = manifest_path
        .parent()
        .with_context(|| anyhow!("Failed to get manifest dir"))?;
    let vcs_info = manifest_dir.join(".cargo_vcs_info.json");

    let mut repository = package.repository.to_owned().with_context(|| {
        anyhow!(
            "Failed to get repository from package: {:#?}",
            package.repository
        )
    })?;
    repository.push_str(".git");

    // read json file
    let info = fs::read_to_string(&vcs_info)?;
    let vcs_info: serde_json::Value = serde_json::from_str(&info)?;
    let sha1 = vcs_info["git"]["sha1"].as_str().unwrap();

    println!("Git repository {:#?}", repository);
    println!("Git repository {:#?}", sha1);

    // check if git is installed
    if Command::new("git").arg("--version").status().is_err() {
        return Err(anyhow::anyhow!("Git is not installed"));
    }

    let temp_dir = tempfile::tempdir()?;

    println!("Cloning repository: {}", repository);

    Command::new("git")
        .args(["clone", &repository, temp_dir.path().to_str().unwrap()])
        .output()
        .context("Failed to clone repository")?;

    Command::new("git")
        .args(["checkout", &sha1])
        .output()
        .context("Failed to checkout proper sha1")?;

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

    let output_string = home_dir.to_str().unwrap_or_default();
    let confirmation_string = format!("Copy example to {}", output_string);
    let ans = Confirm::new(&confirmation_string)
        .with_default(false)
        .with_help_message("Please confirm to continue")
        .prompt();

    match ans {
        Ok(true) => {
            println!("Copying example to {}", output_string);
            copy_dir_recursively(&path, &home_dir)?;
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
