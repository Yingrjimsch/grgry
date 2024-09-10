use colored::Colorize;
use inquire::{validator::Validation, CustomType, Select};

use crate::{
    config::config::{Config, Profile},
    utils::cmd::{create_git_cmd, run_cmd_o, run_cmd_s},
    utils::helper::{prntln, MessageType},
};

use super::mass::process_repos;

pub fn quick(
    message: &str,
    force: bool,
    regex: &str,
    reverse: bool,
    skip_interactive: bool,
    dry_run: bool,
    config: Config,
) {
    process_repos(
        regex,
        reverse,
        |repo| {
            let repo_path = repo.to_string_lossy();
            let has_changes = !run_cmd_o(
                create_git_cmd(&repo_path).arg("status").arg("--porcelain"),
                dry_run,
            )
            .is_empty();

            if !has_changes {
                return Ok(false);
            }

            prntln(
                &format!("There are changes in the repository {}", repo.display()),
                MessageType::Neutral,
            );

            if skip_interactive {
                return Ok(true);
            }

            let result = CustomType::<String>::new(
                "Do you want to quicken this repo? (y)es/(n)o/(m)ore information:",
            )
            .with_validator(|input: &String| match input.to_lowercase().as_str() {
                "y" | "n" | "m" => Ok(Validation::Valid),
                other => Ok(Validation::Invalid(
                    format!("Invalid argument {}. Please enter 'y' or 'n'", other)
                        .red()
                        .into(),
                )),
            })
            .prompt();
            match result {
                Ok(choice) => match choice.to_lowercase().as_str() {
                    "y" => Ok(true),
                    "n" => Ok(false),
                    "m" => {
                        run_cmd_s(create_git_cmd(&repo_path).arg("diff"), dry_run, false);
                        prntln(
                            &format!("{:<10}: {}", "URL", get_remote_url(&repo_path, dry_run)),
                            MessageType::Success,
                        );
                        prntln(
                            &format!(
                                "{:<10}: {}",
                                "Branch",
                                get_current_branch(&repo_path, dry_run)
                            ),
                            MessageType::Success,
                        );
                        Ok(false)
                    }
                    _ => unreachable!(),
                },
                Err(err) => Err(err),
            }
        },
        |repo| {
            let repo_path = repo.to_string_lossy();
            let remote_url = get_remote_url(&repo_path, dry_run);
            let profile = select_profile(&config, &remote_url);

            execute_quick_actions(&repo_path, &profile, message, dry_run);
        },
    );
}

fn get_remote_url(repo_path: &str, dry_run: bool) -> String {
    run_cmd_o(
        create_git_cmd(repo_path).args(&["config", "--get", "remote.origin.url"]),
        dry_run,
    )
}

fn get_current_branch(repo_path: &str, dry_run: bool) -> String {
    run_cmd_o(
        create_git_cmd(repo_path).args(&["branch", "--show-current"]),
        dry_run,
    )
}

fn select_profile<'a>(config: &'a Config, remote_url: &str) -> &'a Profile {
    let profiles = config.find_profiles_by_provider(remote_url);
    if profiles.len() == 1 {
        config.profiles.get(profiles[0]).unwrap()
    } else {
        let selected_profile = Select::new("Choose profile to quicken.", profiles).prompt();
        selected_profile
            .map(|profile_key| config.profiles.get(profile_key).unwrap())
            .unwrap_or_else(|_| config.active_profile())
    }
}

fn execute_quick_actions(repo_path: &str, profile: &Profile, message: &str, dry_run: bool) {
    run_cmd_s(
        create_git_cmd(repo_path).args(&["pull", "--rebase"]),
        dry_run,
        true,
    );
    run_cmd_s(
        create_git_cmd(repo_path).args(&["config", "user.name", &profile.username]),
        dry_run,
        true,
    );
    run_cmd_s(
        create_git_cmd(repo_path).args(&["config", "user.email", &profile.email]),
        dry_run,
        true,
    );
    run_cmd_s(create_git_cmd(repo_path).args(&["add", "."]), dry_run, true);
    run_cmd_s(
        create_git_cmd(repo_path).args(&["commit", "-m", message]),
        dry_run,
        true,
    );

    let branch = get_current_branch(repo_path, dry_run);
    let set_upstream = run_cmd_o(
        create_git_cmd(repo_path).args(&["ls-remote", "--heads", "origin", &branch]),
        dry_run,
    )
    .is_empty();

    run_cmd_s(
        create_git_cmd(repo_path).args(create_push_request_args(&branch, set_upstream)),
        dry_run,
        true,
    );

    prntln(
        &format!(
            "\n{} {} {} {}",
            "Successfully pushed repo into:",
            get_remote_url(repo_path, dry_run),
            "on branch",
            branch
        ),
        MessageType::Success,
    );
}

fn create_push_request_args(branch: &str, set_upstream: bool) -> Vec<String> {
    let mut args: Vec<String> = vec!["push".to_string(), "origin".to_string(), branch.to_string()];
    if set_upstream {
        args.insert(3, "--set-upstream".to_string());
    }
    return args;
}
