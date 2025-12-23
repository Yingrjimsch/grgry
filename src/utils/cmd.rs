use std::process::{Command, ExitStatus, Output, Stdio};

use colored::Colorize;

fn command_to_string(command: &Command) -> String {
    let cmd_str: String = format!("{:?}", command);
    cmd_str
}

pub fn run_cmd_o(command: &mut Command, test: bool) -> String {
    if test {
        let cmd_str: String = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return String::from("");
    } else {
        let output: Output = command.output().expect("Failed to execute command!");
        if !output.status.success() {
            eprintln!(
                "{} {}",
                "Error:".red(),
                String::from_utf8_lossy(&output.stderr).red()
            );
            std::process::exit(1);
        }

        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
}

pub fn run_cmd_o_soft(command: &mut Command, test: bool) -> (String, bool) {
    if test {
        let cmd_str: String = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return (String::from(""), true);
    } else {
        let output: Output = command.output().expect("Failed to execute command!");
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

pub fn run_cmd_s(mut command: &mut Command, test: bool, silent: bool) -> bool {
    if test {
        let cmd_str: String = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return true;
    } else {
        command = if silent {
            command.stdout(Stdio::null()).stderr(Stdio::null())
        } else {
            command
        };
        let status: ExitStatus = command.status().expect("Failed to execute command!");
        if !status.success() {
            eprintln!(
                "{} {}",
                "Error executing command ".red(),
                command_to_string(command).red()
            );
            std::process::exit(1);
        }
        return status.success();
    }
}

/// Execute a command and, on failure, return an error string (including stderr/stdout).
/// This is meant for "mass" operations where we want to continue processing other repos.
pub fn run_cmd_s_soft(command: &mut Command, test: bool, silent: bool) -> Result<(), String> {
    if test {
        let cmd_str: String = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return Ok(());
    }

    // If silent, suppress output on success; but still capture stderr/stdout for error reporting.
    if silent {
        command.stdout(Stdio::null()).stderr(Stdio::piped());
    }

    let output: Output = command.output().map_err(|e| {
        format!(
            "Failed to execute command {}: {}",
            command_to_string(command),
            e
        )
    })?;

    if output.status.success() {
        return Ok(());
    }

    let mut msg = format!(
        "Error executing command {}\n",
        command_to_string(command).red()
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        msg.push_str(&format!("stderr:\n{}\n", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        msg.push_str(&format!("stdout:\n{}\n", stdout));
    }

    Err(msg)
}

/// Run a command with limited retries for transient network/SSH errors.
pub fn run_cmd_s_retry(
    command: &mut Command,
    test: bool,
    silent: bool,
    max_attempts: u8,
) -> Result<(), String> {
    let mut attempt: u8 = 1;
    loop {
        match run_cmd_s_soft(command, test, silent) {
            Ok(()) => return Ok(()),
            Err(err) => {
                let err_lc = err.to_lowercase();
                let retryable = err_lc.contains("kex_exchange_identification")
                    || err_lc.contains("connection reset by peer")
                    || err_lc.contains("connection timed out")
                    || err_lc.contains("operation timed out")
                    || err_lc.contains("broken pipe");

                if !retryable || attempt >= max_attempts {
                    return Err(err);
                }

                // Small linear backoff without adding dependencies.
                let backoff_ms: u64 = match attempt {
                    1 => 250,
                    2 => 750,
                    _ => 1500,
                };

                eprintln!(
                    "{} attempt {}/{} failed (retrying in {}ms)...\n{}",
                    "Warning:".yellow(),
                    attempt,
                    max_attempts,
                    backoff_ms,
                    err
                );

                std::thread::sleep(std::time::Duration::from_millis(backoff_ms));
                attempt += 1;
                continue;
            }
        }
    }
}

pub fn create_git_cmd(repo_path: &str) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_path);
    return command;
}
