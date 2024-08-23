mod commands;
mod config;
mod git_providers;
mod github;
mod gitlab;

use clap::Parser;
use colored::Colorize;
use commands::Commands;
use config::{Config, Profile};
use git_providers::GitProvider;
use git_providers::Repo;
use github::Github;
use gitlab::Gitlab;
use inquire::validator::Validation;
use inquire::Confirm;
use inquire::{error::InquireError, required, CustomType, Select, Text};
use rayon::prelude::*;
use rayon::str;
use regex::Regex;
use std::collections::HashMap;
use std::path;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use tokio::time::error::Elapsed;
use walkdir::WalkDir;

const TEST: bool = false;
const VERSION: &str = "1.0.0";

#[derive(Parser)]
#[command(name = "grgry")]
#[command(about = "A CLI tool for git en mass", long_about = None)]
#[command(version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let mut config = Config::new();
    let cli = Cli::parse();

    match &cli.command {
        Commands::Clone {
            directory,
            user,
            branch,
            regex_args,
        } => {
            let (regex, reverse) = regex_args.get_regex_args(".*");
            clone(
                directory,
                *user,
                branch.to_string(),
                &regex,
                reverse,
                config,
            )
            .await;
        }
        Commands::Quick {
            message,
            force,
            regex_args,
            skip_interactive,
        } => {
            let (regex, reverse) = regex_args.get_regex_args("false");
            quick(message, *force, &regex, reverse, *skip_interactive, config);
        }
        Commands::Mass {
            command,
            regex_args,
            skip_interactive,
        } => {
            let (regex, reverse) = regex_args.get_regex_args(".*");
            mass(command, &regex, reverse, *skip_interactive)
        }
        Commands::Profile { sub } => match &sub {
            commands::ProfileCommands::Activate => activate_profile_prompt(&mut config),
            commands::ProfileCommands::Add => add_profile_prompt(&mut config),
            commands::ProfileCommands::Delete => delete_profile_prompt(&mut config),
        },
    }
}

fn delete_profile_prompt(config: &mut Config) {
    let profiles_cloned = do_clone(&config.profiles);
    let profile_keys: Vec<&str> = profiles_cloned.keys().map(|key| key.as_str()).collect();
    let profile_to_delete_key: Result<&str, InquireError> =
        Select::new("Which profile do you want to delete?", profile_keys).prompt();

    match profile_to_delete_key {
        Ok(choice) => {
            config.delete_profile(choice);
            config.save_config();
            println!("{}", serde_json::to_string_pretty(&config).unwrap());
        }
        Err(_) => println!("{}", "There was an error, please try again".red()),
    }
}

fn add_profile_prompt(config: &mut Config) {
    let profile_name = Text::new("profile name:")
        .with_validator(required!("This field is required"))
        //TODO: would be nice to validate if the profile already exists but does not work due to my rust incapabilities
        // .with_validator(
        //     move |name: &str| {
        //     if choices.iter().any(|e| name.contains(e)) {
        //         Ok(Validation::Invalid("not exists".into()))
        //     } else {
        //         Ok(Validation::Valid)
        //     }
        // })
        .with_help_message("Optional notes")
        .prompt();
    match profile_name {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let target_base_path = Text::new("target base path:")
        .with_validator(required!("This field is required"))
        .with_help_message("Optional notes")
        .with_default("")
        .prompt();
    match target_base_path {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let user_name = Text::new("user name:")
        .with_help_message("Optional notes")
        .with_default("")
        .prompt();
    match user_name {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let user_email = Text::new("user email:")
        .with_help_message("Optional notes")
        .with_default("")
        .prompt();
    match user_email {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let providers: Vec<&str> = vec!["github", "gitlab"];
    let provider: Result<&str, InquireError> = Select::new("choose provider", providers).prompt();
    match provider {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let pull_options: Vec<&str> = vec!["ssh", "https"];
    let pulloption: Result<&str, InquireError> =
        Select::new("choose pull option", pull_options).prompt();
    match pulloption {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let base_address = Text::new("base address:")
        .with_validator(required!("This field is required"))
        .with_help_message("Optional notes")
        .prompt();
    match base_address {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let token = Text::new("token:")
        .with_help_message("Optional notes")
        .with_default("")
        .prompt();
    match token {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let activate = Confirm::new("Do you want to activate the profile?")
        .with_default(false)
        .prompt();
    match activate {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let profile_name = profile_name.unwrap();
    config.profiles.insert(
        profile_name.clone(),
        Profile {
            active: false,
            pulloption: String::from(pulloption.unwrap()),
            username: user_name.unwrap(),
            targetbasepath: target_base_path.unwrap(),
            email: user_email.unwrap(),
            baseaddress: base_address.unwrap(),
            provider: String::from(provider.unwrap()),
            token: token.unwrap(),
        },
    );
    match activate {
        Ok(true) => {
            config.activate_profile(&profile_name.clone());
        }
        _ => {}
    }
    config.save_config();
    println!("{}", serde_json::to_string_pretty(&config).unwrap());
}

fn do_clone<K: Clone, V: Clone>(data: &HashMap<K, V>) -> HashMap<K, V> {
    data.clone()
}

fn activate_profile_prompt(config: &mut Config) {
    let profiles_cloned = do_clone(&config.profiles);
    let profile_keys: Vec<&str> = profiles_cloned.keys().map(|key| key.as_str()).collect();

    let profile_to_activate_key: Result<&str, InquireError> =
        Select::new("Choose profile to activate:", profile_keys).prompt();
    match profile_to_activate_key {
        Ok(choice) => {
            config.activate_profile(choice);
            config.save_config();
            println!("{} {}", "Activated profile is:".green(), choice.green());
        }
        Err(_) => println!("{}", "Active profile could not be changed! Make sure there you have a profile configured with grgry profile add.".red()),
    }
}

fn mass(command: &str, regex: &str, reverse: bool, skip_interactive: bool) {
    let repos = find_git_repos_parallel(None, &regex, reverse);
    for repo in repos {
        loop {
            println!(
                "{} {}",
                "Repo has been found at:".green(),
                repo.clone().into_os_string().into_string().unwrap().green()
            );
            let allow_mass: Result<String, InquireError> = if !skip_interactive {
                CustomType::<String>::new(&format!(
                    "Do you want to execute {}? (y)es/(n)o:",
                    command
                ))
                .with_validator(&|input: &String| match input.to_lowercase().as_str() {
                    "y" | "n" => Ok(Validation::Valid),
                    _ => Ok(Validation::Invalid("Please enter 'y' or 'n'".red().into())),
                })
                .with_error_message(&"Please type 'y' or 'n'".red())
                .prompt()
            } else {
                Ok("y".to_string())
            };
            match allow_mass {
                Ok(choice) => match choice.to_lowercase().as_str() {
                    "y" => {
                        let repo_path = repo.clone().into_os_string().into_string().unwrap();
                        let mut args = vec!["-C", &repo_path];
                        args.extend(command.split_whitespace());
                        run_cmd_s(Command::new("git").args(args), TEST, false);
                        break;
                    }
                    "n" => break,
                    _ => unreachable!(),
                },
                Err(_) => break,
            };
        }
    }
}

fn quick(
    message: &str,
    force: bool,
    regex: &str,
    reverse: bool,
    skip_interactive: bool,
    config: Config,
) {
    let repos = find_git_repos_parallel(None, regex, reverse);
    for repo in repos {
        let has_changes = run_cmd_o(
            Command::new("git").args(&[
                "-C",
                &repo.clone().into_os_string().into_string().unwrap(),
                "status",
                "--porcelain",
            ]),
            TEST,
        );
        match !has_changes.is_empty() {
            true => loop {
                println!(
                    "There are changes in the repository {}",
                    repo.clone().into_os_string().into_string().unwrap()
                );
                let allow_quicken = if !skip_interactive {
                    CustomType::<String>::new(
                        "Do you want to quicken this repo? (y)es/(n)o/(m)ore information:",
                    )
                    .with_validator(&|input: &String| match input.to_lowercase().as_str() {
                        "y" | "n" | "m" => Ok(Validation::Valid),
                        _ => Ok(Validation::Invalid(
                            "Please enter 'y', 'n', or 'm'.".red().into(),
                        )),
                    })
                    .with_error_message(&"Please type 'y', 'n', or 'm'.".red())
                    .prompt()
                } else {
                    Ok("y".to_string())
                };
                let remote_origin_url = run_cmd_o(
                    Command::new("git").args(&[
                        "-C",
                        &repo.clone().into_os_string().into_string().unwrap(),
                        "config",
                        "--get",
                        "remote.origin.url",
                    ]),
                    TEST,
                );
                match allow_quicken {
                    Ok(choice) => match choice.to_lowercase().as_str() {
                        "y" => {
                            let profile_keys = config.find_profiles_by_provider(&remote_origin_url);
                            let profile = if profile_keys.len() == 1 {
                                config
                                    .profiles
                                    .get(&profile_keys.first().unwrap().to_string())
                                    .unwrap()
                            } else {
                                let selected_profile_key: Result<&str, InquireError> =
                                    Select::new("Choose profile to quicken?", profile_keys)
                                        .prompt();
                                let chosen_profile = match selected_profile_key {
                                    Ok(choice) => config.profiles.get(choice).unwrap(),
                                    Err(_) => config.active_profile(),
                                };
                                chosen_profile
                            };
                            run_cmd_s(
                                Command::new("git").args(&[
                                    "-C",
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    "config",
                                    "user.name",
                                    &profile.username,
                                ]),
                                TEST,
                                true,
                            );
                            run_cmd_s(
                                Command::new("git").args(&[
                                    "-C",
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    "config",
                                    "user.email",
                                    &profile.email,
                                ]),
                                TEST,
                                true,
                            );
                            run_cmd_s(
                                Command::new("git").args(&[
                                    "-C",
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    "add",
                                    ".",
                                ]),
                                TEST,
                                true,
                            );
                            run_cmd_s(
                                Command::new("git").args(&[
                                    "-C",
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    "commit",
                                    "-m",
                                    message,
                                ]),
                                TEST,
                                true,
                            );
                            let current_branch = run_cmd_o(
                                Command::new("git").args(&[
                                    "-C",
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    "branch",
                                    "--show-current",
                                ]),
                                TEST,
                            );
                            let set_upstream = run_cmd_o(
                                Command::new("git").args(&[
                                    "-C",
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    "ls-remote",
                                    "--heads",
                                    "origin",
                                    &current_branch,
                                ]),
                                TEST,
                            )
                            .is_empty();
                            run_cmd_s(
                                Command::new("git").args(create_push_request_args(
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    &current_branch,
                                    set_upstream,
                                )),
                                TEST,
                                true,
                            );
                            println!(
                                "\n{} {} {} {}",
                                "Successfully pushed repo into:".green(),
                                remote_origin_url.green(),
                                "on branch".green(),
                                current_branch.green()
                            );
                            break;
                        }
                        "n" => break,
                        "m" => {
                            run_cmd_s(
                                Command::new("git").args(&[
                                    "-C",
                                    &repo.clone().into_os_string().into_string().unwrap(),
                                    "diff",
                                ]),
                                TEST,
                                false,
                            );
                            println!("{0: <10}: {1}", "URL", &remote_origin_url);
                            println!(
                                "{0: <10}: {1}",
                                "Branch",
                                &run_cmd_o(
                                    Command::new("git").args(&[
                                        "-C",
                                        &repo.clone().into_os_string().into_string().unwrap(),
                                        "branch",
                                        "--show-current"
                                    ]),
                                    TEST
                                )
                            );
                        }
                        _ => unreachable!(),
                    },
                    Err(_) => break,
                };
            },
            false => {}
        }
    }
}

fn command_to_string(command: &Command) -> String {
    let cmd_str = format!("{:?}", command);
    cmd_str
}

fn run_cmd_o(command: &mut Command, test: bool) -> String {
    if test {
        let cmd_str = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return String::from("");
    } else {
        let output = command.output().expect("Failed to execute command!");
        if !output.status.success() {
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }

        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
}

fn run_cmd_o_soft(command: &mut Command, test: bool) -> (String, bool) {
    if test {
        let cmd_str = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return (String::from(""), true);
    } else {
        let output = command.output().expect("Failed to execute command!");
        if !output.status.success() {
            return (
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                false,
            );
        }
        return (
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
            true,
        );
    }
}

fn run_cmd_s(mut command: &mut Command, test: bool, silent: bool) -> bool {
    if test {
        let cmd_str = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return true;
    } else {
        command = if silent {
            command.stdout(Stdio::null()).stderr(Stdio::null())
        } else {
            command
        };
        let status = command.status().expect("Failed to execute command!");
        if !status.success() {
            eprintln!("Error executing command on {}", command_to_string(command));
            std::process::exit(1);
        }
        return status.success();
    }
}

async fn clone(
    directory: &str,
    user: bool,
    branch: String,
    regex: &str,
    reverse: bool,
    config: Config,
) {
    let active_profile: Profile = config.active_profile().clone();
    let pat: Option<String> = Some(active_profile.clone().token);
    let provider_type: &str = &active_profile.provider;
    // Find amount of repositories
    let provider: Box<dyn GitProvider> = match provider_type {
        "gitlab" => Box::new(Gitlab),
        "github" => Box::new(Github),
        _ => unreachable!(),
    };
    let all_repos: Vec<Box<dyn Repo>> =
        provider.get_repos(&pat, directory, user, active_profile.clone());

    let num_threads = std::thread::available_parallelism().unwrap().into();
    let re = Regex::new(regex).expect("Invalid regex pattern");
    let repos_to_clone: Vec<Box<dyn Repo>> = all_repos
        .into_iter()
        .filter(|repo| (re.is_match(&repo.http_url()) || re.is_match(&repo.ssh_url())) ^ reverse)
        .collect();
    run_in_threads(
        num_threads,
        repos_to_clone,
        move |thread_id: usize, repo: &Box<dyn Repo>| {
            let destination_path: String =
                format!("{}/{}", active_profile.targetbasepath, repo.full_path());
            let clone_url = if active_profile.pulloption == "ssh" {
                &repo.ssh_url()
            } else {
                &repo.http_url()
            };
            if Path::new(&destination_path).exists() {
                let mut current_branch = branch.clone();
                if branch == "" {
                    let (symbolic_ref, success) = run_cmd_o_soft(
                        Command::new("git").args(&[
                            "-C",
                            &destination_path,
                            "symbolic-ref",
                            "refs/remotes/origin/HEAD",
                            "--short",
                        ]),
                        TEST,
                    );
                    if !success {
                        return; //HERE maybe an println as information?
                    }
                    current_branch = symbolic_ref.strip_prefix("origin/").unwrap().to_string();
                }
                let branch_exists = run_cmd_o(
                    Command::new("git").args(&[
                        "-C",
                        &destination_path,
                        "ls-remote",
                        "--heads",
                        "origin",
                        &current_branch,
                    ]),
                    TEST,
                );
                if branch_exists != "" {
                    run_cmd_s(
                        Command::new("git").args(&[
                            "-C",
                            &destination_path,
                            "checkout",
                            &current_branch,
                        ]),
                        TEST,
                        true,
                    );
                    //HERE only git pull the shit out of it
                    run_cmd_s(
                        Command::new("git").args(&["-C", &destination_path, "pull"]),
                        TEST,
                        true,
                    );
                    println!("Repo: {} successfully pulled!", clone_url);
                }
                return;
            }

            let status = run_cmd_s(
                Command::new("git")
                    .args(&create_pull_request_args(
                        &branch,
                        &clone_url,
                        &active_profile.targetbasepath,
                        &repo.full_path(),
                    ))
                    .stdout(Stdio::null())
                    .stderr(Stdio::null()),
                TEST,
                true,
            );
            if status {
                println!("Repo: {} successfully cloned!", clone_url.green());
            }
        },
    );
    println!("Finished cloning repositories");
}

fn create_pull_request_args(
    branch: &str,
    clone_url: &str,
    target_basepath: &str,
    directory: &str,
) -> Vec<String> {
    let mut args = vec![
        "clone".to_string(),
        clone_url.to_string(),
        format!("{}/{}", target_basepath, directory),
    ];

    if branch != "" {
        args.insert(1, "-b".to_string());
        args.insert(2, branch.to_string());
    }
    return args;
}

fn find_git_repos_parallel(root: Option<&Path>, pattern: &str, reverse: bool) -> Vec<PathBuf> {
    let root: &Path = root.unwrap_or(Path::new("."));
    if pattern == "false" {
        return vec![root.to_path_buf()];
    }
    let regex: Regex = Regex::new(pattern).expect("Invalid regex pattern");

    WalkDir::new(root)
        .into_iter()
        .par_bridge() // Convert iterator to a parallel iterator
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .filter(|entry| {
            let path = entry.path();
            // Check if the directory contains a .git folder
            path.join(".git").is_dir()
        })
        .filter(|entry| {
            let path_str = entry.path().to_string_lossy();
            // Check if the path matches the regex pattern
            regex.is_match(&path_str) ^ reverse
        })
        .map(|entry| entry.into_path())
        .collect()
}

fn create_push_request_args(repo_path: &str, branch: &str, set_upstream: bool) -> Vec<String> {
    let mut args = vec![
        "-C".to_string(),
        repo_path.to_string(),
        "push".to_string(),
        "origin".to_string(),
        branch.to_string(),
    ];
    if set_upstream {
        args.insert(3, "--set-upstream".to_string());
    }
    return args;
}

fn run_in_threads<F, T, R>(num_threads: usize, items: Vec<T>, task: F) -> Vec<R>
where
    F: Fn(usize, &T) -> R + Send + Sync + 'static,
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    let task: Arc<F> = Arc::new(task);
    let items: Arc<Vec<T>> = Arc::new(items);
    let (tx_result, rx_result) = mpsc::channel();

    let mut handles: Vec<thread::JoinHandle<()>> = vec![];

    for thread_id in 0..num_threads {
        let task: Arc<F> = Arc::clone(&task);
        let items: Arc<Vec<T>> = Arc::clone(&items);
        let tx_result: mpsc::Sender<R> = tx_result.clone();
        let handle: thread::JoinHandle<()> = thread::spawn(move || {
            for i in (thread_id..items.len()).step_by(num_threads) {
                let item: &T = &items[i];
                let result: R = task(thread_id, item);
                tx_result.send(result).expect("Failed to send result");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    drop(tx_result); // Close the channel
    let mut results: Vec<R> = Vec::new();
    for result in rx_result {
        results.push(result);
    }

    results
}
