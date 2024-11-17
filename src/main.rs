use anyhow::{Context, Result};
use std::io::Write;
use std::{fs, path::PathBuf, process::Command};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    assert!(
        args.len() == 2,
        "Arguments are not what I expected. Try running: cargo sample <REPO>"
    );

    assert_eq!(
        args[0], "sample",
        "It seems that the binary is not being run with cargo. Try running: cargo sample <REPO>"
    );

    let repo = &args[1];

    let temp_dir = tempfile::tempdir()?;

    println!(
        "Cloning repository {} into temp dir {}",
        repo,
        temp_dir.path().to_str().unwrap()
    );

    println!("Cloning repository...");
    Command::new("git")
        .args(["clone", repo, temp_dir.path().to_str().unwrap()])
        .output()
        .context("Failed to clone repository")?;

    // Find the examples directory
    let examples_dir = temp_dir.path().join("examples");
    assert!(
        examples_dir.exists(),
        "No examples directory found in repository"
    );

    // Present the user with a list of examples
    let examples = fs::read_dir(&examples_dir)?;

    for (i, example) in examples.enumerate() {
        let example = example?;
        println!("{}. {}", i, example.file_name().to_str().unwrap());
    }

    print!("Choose an example: ");
    std::io::stdout().flush()?;
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim().parse::<usize>().unwrap();

    let example = fs::read_dir(&examples_dir)?
        .nth(choice)
        .unwrap()?
        .file_name()
        .to_str()
        .unwrap()
        .to_string();

    let path = examples_dir.join(&example);

    // output folder is the current folder
    let output = PathBuf::from(".");

    println!("Copying the example to current folder");
    copy_dir_recursively(&path, &output)?;

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
