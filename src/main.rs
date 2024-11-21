use anyhow::{anyhow, Context, Result};
use argh::FromArgs;
use cargo_metadata::MetadataCommand;
use cargo_metadata::Package;
use inquire::Confirm;
use inquire::Select;
use std::{fs, path::PathBuf, process::Command};
use toml_edit::Document;
use toml_edit::Item;

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

    let output = std::env::current_dir()?;
    println!("Current directory: {:?}", output);

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
        .args(["checkout", sha1])
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
        } else if entry.file_name() == "Cargo.toml" {
            merge_cargo_toml(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn merge_cargo_toml(src_path: &PathBuf, dst_path: &PathBuf) -> Result<()> {
    // Read both Cargo.toml files
    let dst_content = fs::read_to_string(dst_path)?;
    let src_content = fs::read_to_string(src_path)?;

    // Parse both files
    let dst_doc = dst_content.parse::<Document>()?;
    let mut src_doc = src_content.parse::<Document>()?;

    // Get the package name from the destination (original) Cargo.toml
    if let Some(dst_package) = dst_doc.get("package") {
        if let Some(dst_name) = dst_package.get("name") {
            // Replace the package name in the source Cargo.toml
            if let Some(src_package) = src_doc.as_table_mut().get_mut("package") {
                if let Item::Table(table) = src_package {
                    table.insert("name", dst_name.clone());
                }
            }
        }
    }

    // Merge dependencies, prioritizing destination dependencies
    if let Some(src_deps) = src_doc.as_table_mut().get_mut("dependencies") {
        if let Item::Table(src_deps_table) = src_deps {
            // Remove any dependencies that exist in destination
            if let Some(dst_deps) = dst_doc.get("dependencies") {
                if let Item::Table(dst_deps_table) = dst_deps {
                    for key in dst_deps_table.iter().map(|(k, _)| k) {
                        src_deps_table.remove(key);
                    }
                }
            }

            // Add back the destination dependencies
            if let Some(dst_deps) = dst_doc.get("dependencies") {
                if let Item::Table(dst_deps_table) = dst_deps {
                    for (key, value) in dst_deps_table.iter() {
                        src_deps_table.insert(key, value.clone());
                    }
                }
            }
        }
    }

    // Write the modified source content to the destination
    fs::write(dst_path, src_doc.to_string())?;

    Ok(())
}
