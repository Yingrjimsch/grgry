use clap::Parser;
use grgry::{cli::{clone, commands::{Commands, ProfileCommands}, mass, profile::{activate_profile_prompt, add_profile_prompt, delete_profile_prompt, show_profile}, quick}, config::config::Config};

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
            let (regex, reverse) = regex_args.get_regex_args(".*");
            mass(command, &regex, reverse, *skip_interactive, *dry_run)
        }
        Commands::Profile { sub } => match &sub {
            ProfileCommands::Activate => activate_profile_prompt(&mut config),
            ProfileCommands::Add => add_profile_prompt(&mut config),
            ProfileCommands::Delete => delete_profile_prompt(&mut config),
            ProfileCommands::Show { all }=> show_profile(*all, config),
        },
        // Commands::Test { } => {
        //     // let reops = find_git_repos_parallel(None, ".*", false);
        //     // println!("{:?}", reops);
        // }
    }
}
