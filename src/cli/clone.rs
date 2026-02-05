use crate::{
    config::config::{Config, Profile},
    git_api::git_providers::{get_provider, GitProvider, Repo},
    utils::cmd::{create_git_cmd, run_cmd_o, run_cmd_o_soft, run_cmd_s_retry},
    utils::helper::{self, prntln, run_in_threads, MessageType},
};
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Client;
use std::{
    fs::remove_dir_all,
    ops::ControlFlow,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
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
    let branch_arg = branch.clone();
    // Limit concurrency for network-heavy git operations to reduce SSH connection resets.
    // (Fixes flaky failures like "kex_exchange_identification" when running many fetch/clone in parallel.)
    run_in_threads(
        4,
        repos_to_clone,
        move |_thread_id: usize, repo: &Box<dyn Repo>| {
            let destination_path: String =
                format!("{}/{}", active_profile.targetbasepath, repo.full_path());
            let clone_url = get_clone_url(&active_profile.pulloption, repo);
            if Path::new(&destination_path).exists() {
                if dry_run {
                    let branch_from_metadata =
                        determine_target_branch(&branch_arg, repo.default_branch());
                    let target_branch =
                        resolved_branch(&branch_arg, branch_from_metadata.as_deref(), &destination_path);

                    let would_skip = branch_from_metadata
                        .as_deref()
                        .map(|b| should_skip_pull(b, repo, &destination_path))
                        .unwrap_or(false);

                    if would_skip {
                        prntln(
                            &format!(
                                "Repo exists at {} (dry-run): remote not newer; would skip pull",
                                destination_path
                            ),
                            MessageType::Success,
                        );
                        ControlFlow::Continue(())
                    } else {
                        match target_branch {
                            Some(branch_name) => prntln(
                                &format!(
                                    "Repo exists at {} (dry-run): would pull branch {}",
                                    destination_path, branch_name
                                ),
                                MessageType::Neutral,
                            ),
                            None => prntln(
                                &format!(
                                    "Repo exists at {} (dry-run): no cached origin refs; would skip pull",
                                    destination_path
                                ),
                                MessageType::Success,
                            ),
                        }
                        ControlFlow::Continue(())
                    }
                } else {
                    let branch_from_metadata =
                        determine_target_branch(&branch_arg, repo.default_branch());
                    let target_branch =
                        resolved_branch(&branch_arg, branch_from_metadata.as_deref(), &destination_path);

                    if branch_from_metadata
                        .as_deref()
                        .map(|b| should_skip_pull(b, repo, &destination_path))
                        .unwrap_or(false)
                    {
                        prntln(
                            &format!(
                                "Repository {} up-to-date locally (skipping pull)",
                                clone_url
                            ),
                            MessageType::Success,
                        );
                        ControlFlow::Continue(())
                    } else {
                        match target_branch {
                            Some(branch_name) => pull(&branch_name, destination_path, clone_url, dry_run),
                            None => {
                                prntln(
                                    &format!(
                                        "Repository {} has no cached origin refs; skipping pull to avoid network",
                                        clone_url
                                    ),
                                    MessageType::Neutral,
                                );
                                ControlFlow::Continue(())
                            }
                        }
                    }
                }
            } else {
                let mut cmd = Command::new("git");
                cmd.args(&create_clone_args(
                    &branch,
                    &clone_url,
                    &active_profile.targetbasepath,
                    &repo.full_path(),
                ))
                .stdout(Stdio::null())
                .stderr(Stdio::null());

                match run_cmd_s_retry(&mut cmd, dry_run, true, 3) {
                    Ok(()) => {
                        helper::prntln(
                            &format!(
                                "\n{} {} {}",
                                "Repository", clone_url, "successfully cloned!"
                            ),
                            MessageType::Success,
                        );
                    }
                    Err(err) => {
                        prntln(
                            &format!(
                                "Failed to clone {} into {}:\n{}",
                                clone_url, destination_path, err
                            ),
                            MessageType::Error,
                        );
                    }
                }

                ControlFlow::Continue(())
            }
        },
    );

    // let elapsed = now.elapsed();
    // println!("Elapsed: {:.2?}", elapsed);
    prntln("\n\nFinished to clone repositories", MessageType::Success);
}

fn determine_target_branch(cli_branch: &str, repo_default_branch: Option<&str>) -> Option<String> {
    if !cli_branch.is_empty() {
        // Only trust metadata when the requested branch matches the remote default.
        if let Some(default_branch) = repo_default_branch {
            if default_branch == cli_branch {
                return Some(cli_branch.to_string());
            }
        }
        return None;
    }

    repo_default_branch.map(|b| b.to_string())
}

fn resolved_branch(
    cli_branch: &str,
    metadata_branch: Option<&str>,
    destination_path: &str,
) -> Option<String> {
    if let Some(b) = metadata_branch {
        return Some(b.to_string());
    }

    if !cli_branch.is_empty() {
        return Some(cli_branch.to_string());
    }

    if let Some(local_origin_branch) = best_local_origin_branch(destination_path) {
        return Some(local_origin_branch);
    }

    None
}

fn should_skip_pull(branch: &str, repo: &Box<dyn Repo>, destination_path: &str) -> bool {
    if let Some(remote_activity) = repo.last_activity_at() {
        let (local_ts_str, ok) = run_cmd_o_soft(
            create_git_cmd(destination_path)
                .arg("log")
                .arg("-1")
                .arg("--format=%ct")
                .arg(branch),
            false,
        );
        if ok {
            if let Some(local_ts) = parse_epoch_to_datetime(&local_ts_str) {
                if remote_activity <= local_ts {
                    return true;
                }
            }
        }
    }

    // Fallback: compare local branch tip with cached remote-tracking ref without hitting the network.
    let (local_sha, ok_local) = run_cmd_o_soft(
        create_git_cmd(destination_path).arg("rev-parse").arg(branch),
        false,
    );
    let (remote_sha, ok_remote) = run_cmd_o_soft(
        create_git_cmd(destination_path)
            .arg("rev-parse")
            .arg(format!("origin/{}", branch)),
        false,
    );

    if ok_remote && !remote_sha.is_empty() {
        return ok_local && !local_sha.is_empty() && local_sha == remote_sha;
    }

    // If we have no cached remote ref, avoid server load by skipping.
    true
}

fn parse_epoch_to_datetime(epoch: &str) -> Option<DateTime<Utc>> {
    let seconds: i64 = epoch.trim().parse().ok()?;
    DateTime::from_timestamp(seconds, 0)
}

fn best_local_origin_branch(destination_path: &str) -> Option<String> {
    // First, try origin/HEAD if it exists locally.
    let (symbolic_ref, ok) = run_cmd_o_soft(
        create_git_cmd(destination_path)
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .arg("--short"),
        false,
    );
    if ok {
        if let Some(branch_name) = symbolic_ref.strip_prefix("origin/") {
            if !branch_name.is_empty() {
                return Some(branch_name.to_string());
            }
        }
    }

    // Next, pick the most recently updated remote branch from local refs.
    let (branches, ok_branches) = run_cmd_o_soft(
        create_git_cmd(destination_path)
            .arg("for-each-ref")
            .arg("--format=%(refname:short)")
            .arg("--sort=-committerdate")
            .arg("refs/remotes/origin"),
        false,
    );
    if ok_branches {
        if let Some(first) = branches.lines().find(|l| l.starts_with("origin/")) {
            if let Some(name) = first.strip_prefix("origin/") {
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
    }

    None
}

fn pull(branch: &str, destination_path: String, clone_url: &str, dry_run: bool) -> ControlFlow<()> {
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
        if let Err(err) = run_cmd_s_retry(
            create_git_cmd(&destination_path)
                .arg("checkout")
                .arg(&current_branch),
            dry_run,
            true,
            3,
        ) {
            prntln(
                &format!(
                    "Failed to checkout '{}' in {}:\n{}",
                    current_branch, destination_path, err
                ),
                MessageType::Error,
            );
            return ControlFlow::Continue(());
        }

        if let Err(err) = run_cmd_s_retry(
            create_git_cmd(&destination_path).arg("pull"),
            dry_run,
            true,
            3,
        ) {
            prntln(
                &format!("Failed to pull in {}:\n{}", destination_path, err),
                MessageType::Error,
            );
            return ControlFlow::Continue(());
        }

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
    // In dry-run mode we don't execute git, so we can't reliably determine the remote HEAD.
    // Use the provided branch (if any) or fall back to a common default.
    if dry_run {
        return Ok(if branch.is_empty() {
            "main".to_string()
        } else {
            branch.to_string()
        });
    }

    // If user provided a branch, use it as-is.
    if !branch.is_empty() {
        return Ok(branch.to_string());
    }

    // Best-effort: ensure the remote refs exist locally before reading origin/HEAD.
    // This fixes repos where the local clone exists but remote refs are missing/stale.
    // Don't silence errors: run_cmd_s would exit(1) on failure, which is what we want here.
    if let Err(err) = run_cmd_s_retry(
        create_git_cmd(destination_path)
            .arg("fetch")
            .arg("--prune")
            .arg("origin"),
        false,
        true,
        3,
    ) {
        prntln(
            &format!(
                "Failed to fetch/prune origin in {}:\n{}",
                destination_path, err
            ),
            MessageType::Error,
        );
        return Err(ControlFlow::Break(()));
    }

    // Determine default branch from origin/HEAD (e.g. "origin/main").
    let (symbolic_ref, success) = run_cmd_o_soft(
        create_git_cmd(destination_path)
            .arg("symbolic-ref")
            .arg("refs/remotes/origin/HEAD")
            .arg("--short"),
        false,
    );

    if success {
        if let Some(branch_name) = symbolic_ref.strip_prefix("origin/") {
            if !branch_name.is_empty() {
                return Ok(branch_name.to_string());
            }
        }
        prntln(
            &format!(
                "Could not determine default branch from origin/HEAD (got: '{}')",
                symbolic_ref
            ),
            MessageType::Error,
        );
        return Err(ControlFlow::Break(()));
    }

    // Fallback: try to read the remote default branch via `git remote show origin`
    // (works even if origin/HEAD isn't set locally).
    let (remote_show, ok_remote_show) = run_cmd_o_soft(
        create_git_cmd(destination_path)
            .arg("remote")
            .arg("show")
            .arg("origin"),
        false,
    );

    if ok_remote_show {
        if let Some(line) = remote_show
            .lines()
            .find(|l| l.trim_start().starts_with("HEAD branch:"))
        {
            if let Some(head) = line.splitn(2, ':').nth(1).map(|s| s.trim()) {
                if !head.is_empty() {
                    return Ok(head.to_string());
                }
            }
        }
    }

    // Final fallback: avoid hard failure; try "main" then "master" by preferring whatever exists.
    let (remote_branches, ok_branches) = run_cmd_o_soft(
        create_git_cmd(destination_path).arg("branch").arg("-r"),
        false,
    );
    if ok_branches {
        if remote_branches.contains("origin/main") {
            return Ok("main".to_string());
        }
        if remote_branches.contains("origin/master") {
            return Ok("master".to_string());
        }
    }

    prntln(
        "There is no HEAD branch defined in origin and no default branch could be determined.",
        MessageType::Error,
    );
    Err(ControlFlow::Break(()))
}

fn get_clone_url<'a>(pulloption: &'a str, repo: &'a Box<dyn Repo>) -> &'a str {
    let clone_url = if pulloption == "ssh" {
        repo.ssh_url()
    } else {
        repo.http_url()
    };
    return clone_url;
}

fn create_clone_args(
    branch: &str,
    clone_url: &str,
    target_basepath: &str,
    directory: &str,
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "clone".to_string(),
        "--filter=blob:none".to_string(),
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
