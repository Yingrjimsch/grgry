use crate::{
    config::config::{Config, Profile},
    git_api::git_providers::{get_provider, GitProvider, Repo},
    utils::cmd::{create_git_cmd, run_cmd_o, run_cmd_o_soft, run_cmd_s},
    utils::helper::{self, prntln, run_in_threads_default, MessageType},
};
use inquire::error;
use regex::Regex;
use reqwest::Client;
use std::{
    fs::remove_dir_all,
    ops::ControlFlow,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
    time::Instant,
};

pub async fn clone(
    directory: &str,
    force: bool,
    user: bool,
    branch: String,
    regex: &str,
    reverse: bool,
    dry_run: bool,
    config: Config,
    client: Arc<Client>,
) {
    // let now = Instant::now();
    let active_profile: Profile = config.active_profile().clone();
    let pat: Option<String> = Some(active_profile.clone().token);
    let provider_type: &str = &active_profile.provider;
    // Find amount of repositories
    let provider: Box<dyn GitProvider> = get_provider(provider_type);
    let all_repos: Vec<Box<dyn Repo>> =
        provider.get_repos(client, &pat, directory, user, active_profile.clone());
    let re: Regex = Regex::new(regex).expect("Invalid regex pattern");
    let repos_to_clone: Vec<Box<dyn Repo>> = all_repos
        .into_iter()
        .filter(|repo: &Box<dyn Repo>| {
            (re.is_match(&repo.http_url()) || re.is_match(&repo.ssh_url())) ^ reverse
        })
        .collect();
    // let elapsed = now.elapsed();
    // println!("Elapsed: {:.2?}", elapsed);
    if force {
        remove_directory_recursively(&active_profile.targetbasepath);
    }

    prntln(
        &format!(
            "\nCloning {} repositories from {}",
            repos_to_clone.len(),
            active_profile.baseaddress
        ),
        MessageType::Neutral,
    );
    run_in_threads_default(
        repos_to_clone,
        move |_thread_id: usize, repo: &Box<dyn Repo>| {
            let destination_path: String =
                format!("{}/{}", active_profile.targetbasepath, repo.full_path());
            let clone_url = get_clone_url(&active_profile.pulloption, repo);
            if Path::new(&destination_path).exists() {
                pull(&branch, destination_path, clone_url, dry_run)
            } else {
                let status: bool = run_cmd_s(
                    Command::new("git")
                        .args(&create_pull_request_args(
                            &branch,
                            &clone_url,
                            &active_profile.targetbasepath,
                            &repo.full_path(),
                        ))
                        .stdout(Stdio::null())
                        .stderr(Stdio::null()),
                    dry_run,
                    true,
                );
                if status {
                    helper::prntln(
                        &format!(
                            "\n{} {} {}",
                            "Repository", clone_url, "successfully cloned!"
                        ),
                        MessageType::Success,
                    );
                }
                ControlFlow::Continue(())
            }
        },
    );

    // let elapsed = now.elapsed();
    // println!("Elapsed: {:.2?}", elapsed);
    prntln("\n\nFinished to clone repositories", MessageType::Success);
}

fn pull(
    branch: &String,
    destination_path: String,
    clone_url: &str,
    dry_run: bool,
) -> ControlFlow<()> {
    let current_branch = match get_current_pull_branch(branch, &destination_path, dry_run) {
        Ok(value) => value,
        Err(value) => return value,
    };
    let branch_exists: String = run_cmd_o(
        create_git_cmd(&destination_path)
            .arg("ls-remote")
            .arg("--heads")
            .arg("origin")
            .arg(&current_branch),
        dry_run,
    );
    if branch_exists != "" {
        run_cmd_s(
            create_git_cmd(&destination_path)
                .arg("checkout")
                .arg(&current_branch),
            dry_run,
            true,
        );
        run_cmd_s(create_git_cmd(&destination_path).arg("pull"), dry_run, true);
        helper::prntln(
            &format!("{} {} {}", "Repository", clone_url, "successfully pulled!"),
            MessageType::Success,
        );
    }
    ControlFlow::Continue(())
}

fn get_current_pull_branch(
    branch: &str,
    destination_path: &String,
    dry_run: bool,
) -> Result<String, ControlFlow<()>> {
    let current_branch: String = if branch == "" {
        let (symbolic_ref, success) = run_cmd_o_soft(
            create_git_cmd(destination_path)
                .arg("symbolic-ref")
                .arg("refs/remotes/origin/HEAD")
                .arg("--short"),
            dry_run,
        );
        if !success {
            prntln(
                "There is no HEAD branch defined in origin",
                MessageType::Error,
            );
            return Err(ControlFlow::Break(()));
        }
        symbolic_ref.strip_prefix("origin/").unwrap().to_string()
    } else {
        branch.to_string()
    };
    Ok(current_branch)
}

fn get_clone_url<'a>(pulloption: &'a str, repo: &'a Box<dyn Repo>) -> &'a str {
    let clone_url = if pulloption == "ssh" {
        repo.ssh_url()
    } else {
        repo.http_url()
    };
    return clone_url;
}

fn create_pull_request_args(
    branch: &str,
    clone_url: &str,
    target_basepath: &str,
    directory: &str,
) -> Vec<String> {
    let mut args: Vec<String> = vec![
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

fn remove_directory_recursively(path: &str) {
    println!("removing");
    let dir_path = Path::new(path);
    if dir_path.exists() {
        match remove_dir_all(dir_path) {
            Ok(()) => prntln(
                &format!("Base directory {} cleaned successfully.", path),
                MessageType::Success,
            ),
            Err(err) => prntln(
                &format!("Base directory {} not cleaned due to {}", path, err),
                MessageType::Error,
            ),
        };
    }
}
