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
            eprintln!("{} {}", "Error:".red(), String::from_utf8_lossy(&output.stderr).red());
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
            eprintln!("{} {}", "Error executing command on".red(), command_to_string(command).red());
            std::process::exit(1);
        }
        return status.success();
    }
}

pub fn create_git_cmd(repo_path: &str) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_path);
    return command
}