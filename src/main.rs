use std::{process::Command, sync::Arc};

use clap::{CommandFactory, Parser};
use grgry::{cli::{clone, commands::{Commands, ProfileCommands}, mass, profile::{activate_profile_prompt, add_profile_prompt, delete_profile_prompt, show_profile}, quick, update::download_latest_release}, config::config::Config, utils::cmd::{run_cmd_o, run_cmd_o_soft}};
use reqwest::Client;

#[derive(Parser)]
#[command(name = "grgry")]
#[command(about = "A CLI tool for git en mass", long_about = None)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    let mut config: Config = Config::new();
    let cli: Cli = Cli::parse();
    let client = Arc::new(Client::builder()
        .http2_prior_knowledge()
        .build()
        .expect("Failed to build reqwest client"));

    match &cli.command {
        Commands::Clone {
            directory,
            user,
            branch,
            regex_args,
            dry_run
        } => {
            let (regex, reverse) = regex_args.get_regex_args(".*"); //TODO: default as global variable
            clone(
                directory,
                *user,
                branch.to_string(),
                &regex,
                reverse,
                *dry_run,
                config,
                client,
            )
            .await;
        }
        Commands::Quick {
            message,
            force,
            regex_args,
            skip_interactive,
            dry_run,
        } => {
            let (regex, reverse) = regex_args.get_regex_args(".*");
            quick(message, *force, &regex, reverse, *skip_interactive, *dry_run, config);
        }
        Commands::Mass {
            command,
            regex_args,
            skip_interactive,
            dry_run,
        } => {
            println!("Stuck3");
            let (regex, reverse) = regex_args.get_regex_args(".*");
            mass(command, &regex, reverse, *skip_interactive, *dry_run)
        }
        Commands::Profile { sub } => match &sub {
            ProfileCommands::Activate => activate_profile_prompt(&mut config),
            ProfileCommands::Add => add_profile_prompt(&mut config),
            ProfileCommands::Delete => delete_profile_prompt(&mut config),
            ProfileCommands::Show { all }=> show_profile(*all, config),
        },
        Commands::Alias { command } => {
            let mut command_vec: Vec<String> = vec!["grgry".to_string(), "mass".to_string()];
            let mut mass_command: String = String::new();
            let mut command = command.into_iter();
            while let Some(arg) = command.next() {
                match arg.as_str() {
                    // Check for recognized arguments
                    "-s" | "--skip-interactive" | "--dry-run" => {
                        command_vec.push(arg.to_string())
                    },
                    "--regex" | "--rev-regex" => {
                        command_vec.push(arg.to_string());
                        let regex_arg = command.next();
                        match regex_arg {
                            Some(argument) => command_vec.push(argument.to_string()),
                            _ => {}
                        };
                    },
                    _ => {
                        if !mass_command.is_empty() {
                            mass_command.push(' ');
                        }
                        mass_command.push_str(&arg);
                    }
                }
            }
            command_vec.insert(2, mass_command);
            let cli = Cli::parse_from(command_vec);
            if let Commands::Mass {
                command,
                regex_args,
                skip_interactive,
                dry_run,
            } = cli.command
            {
                let (regex, reverse) = regex_args.get_regex_args(".*");
                mass(&command, &regex, reverse, skip_interactive, dry_run)
            }
        },
        Commands::Update { } => {
            match download_latest_release().await {
                Ok(_) => println!("Successfully updated grgry, check new version with grgry --version."),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Commands::Test { } => {
            match download_latest_release().await {
                Ok(_) => println!("Success!"),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    }
}
