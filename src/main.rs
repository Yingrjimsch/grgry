use std::{process::Command, sync::Arc};

use clap::Parser;
use grgry::{
    cli::{
        alias, clone,
        commands::{Commands, ProfileCommands},
        mass,
        profile::{
            activate_profile_prompt, add_profile_prompt, delete_profile_prompt, show_profile,
        },
        quick, update,
    },
    config::config::Config,
    utils::cmd::run_cmd_s,
};
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
    let client = Arc::new(
        Client::builder()
            .http2_prior_knowledge()
            .build()
            .expect("Failed to build reqwest client"),
    );
    run_cmd_s(Command::new("git").arg("--version"), false, true);

    match &cli.command {
        Commands::Clone {
            directory,
            force,
            user,
            branch,
            regex_args,
            dry_run,
        } => {
            let (regex, reverse) = regex_args.get_regex_args(".*"); //TODO: default as global variable
            clone(
                directory,
                *force,
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
            quick(
                message,
                *force,
                &regex,
                reverse,
                *skip_interactive,
                *dry_run,
                config,
            );
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
            ProfileCommands::Show { all } => show_profile(*all, config),
        },
        Commands::Alias { command } => {
            let mass_command = alias(command.to_vec());
            let cli = Cli::parse_from(mass_command);
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
        }
        Commands::Update {} => match update(client).await {
            Ok(_) => {
                println!("Successfully updated grgry, check new version with grgry --version.")
            }
            Err(e) => eprintln!("Error: {}", e),
        }, // Commands::Test { } => {
           //
           // }
    }
}
