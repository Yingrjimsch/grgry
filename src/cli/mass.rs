use crate::{
    utils::cmd::{create_git_cmd, run_cmd_s},
    utils::helper::{prntln, MessageType},
};
use colored::Colorize;
use inquire::{validator::Validation, CustomType, InquireError};
use rayon::iter::{ParallelBridge, ParallelIterator};
use regex::Regex;
use std::{
    env::current_dir,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub fn mass(command: &str, regex: &str, reverse: bool, skip_interactive: bool, dry_run: bool) {
    process_repos(
        regex,
        reverse,
        |repo| {
            prntln(
                &format!("{} {}", "Repository found at:", repo.display()),
                MessageType::Neutral,
            );

            if skip_interactive {
                return Ok(true);
            }

            let prompt = format!("Do you want to execute {}? (y/n):", command);
            let result = CustomType::<String>::new(&prompt)
                .with_validator(|input: &String| match input.to_lowercase().as_str() {
                    "y" | "n" => Ok(Validation::Valid),
                    other => Ok(Validation::Invalid(
                        format!("Invalid argument {}. Please enter 'y' or 'n'", other)
                            .red()
                            .into(),
                    )),
                })
                .prompt();
            match result {
                Ok(choice) => Ok(choice.to_lowercase() == "y"),
                Err(err) => Err(err), // Propagate the error, which will exit the loop
            }
        },
        |repo| {
            let repo_path: std::borrow::Cow<'_, str> = repo.to_string_lossy();
            let args: Vec<&str> = ["-C", &repo_path]
                .iter()
                .copied()
                .chain(command.split_whitespace())
                .collect();

            run_cmd_s(create_git_cmd(&repo_path).args(args), dry_run, false);
        },
    );
}

pub fn process_repos<F, G>(regex: &str, reverse: bool, interactive_fn: F, execute_fn: G)
where
    F: Fn(&PathBuf) -> Result<bool, InquireError>,
    G: Fn(&PathBuf),
{
    let repos: Vec<PathBuf> = find_git_repos_parallel(None, &regex, reverse);
    for repo in repos {
        match interactive_fn(&repo) {
            Ok(true) => execute_fn(&repo),
            Ok(false) => continue,
            Err(_) => break,
        }
    }
}

//the root option is not set yet but could be included so the mass commands are not from the current but from another root dir
fn find_git_repos_parallel(root: Option<&Path>, pattern: &str, reverse: bool) -> Vec<PathBuf> {
    let root: PathBuf = match root {
        Some(path) => path.to_path_buf(),
        None => current_dir().expect("Failed to get current directory"),
    };
    let regex: Regex = Regex::new(pattern).expect("Invalid regex pattern");

    WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| {
            let path = entry.path();
            // Continue descending only if the directory does not contain a .git folder
            !path
                .parent()
                .map(|p| p.join(".git").is_dir())
                .unwrap_or(false)
        })
        .par_bridge()
        // Convert iterator to a parallel iterator
        .filter_map(|entry: Result<walkdir::DirEntry, walkdir::Error>| entry.ok())
        .filter(|entry: &walkdir::DirEntry| entry.file_type().is_dir())
        .filter(|entry: &walkdir::DirEntry| {
            let path = entry.path();
            // Check if the directory contains a .git folder
            path.join(".git").is_dir()
        })
        .filter(|entry| {
            let path_str = entry.path().to_string_lossy();
            // Check if the path matches the regex pattern
            regex.is_match(&path_str) ^ reverse
        })
        .map(|entry: walkdir::DirEntry| entry.into_path())
        .collect()
}
