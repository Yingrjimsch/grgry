use clap::Subcommand;

use crate::Profile;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Clone a repository into the specified directory")]
    Clone {
        #[arg(value_name = "DIRECTORY", help = "The group / org / user / repo to clone")]
        directory: String,
        
        #[arg(short, long, default_value_t = false, help = "Specify if the directory is a user directory or not (default false)")]
        user: bool,

        #[arg(short, long, default_value = "", help = "Clone specific branch (default no specific branch)")]
        branch: String,

        #[arg(long, default_value = ".*", help = "Filter the directory via regex (default .*)")]
        regex: String,
    },
    #[command(about = "Make git add, git commit, git push in one go.")]
    Quick {
        #[arg(value_name = "MESSAGE", help = "Commit message for quick.")]
        message: String,

        #[arg(short, long, default_value_t = false)]
        force: bool,

        #[arg(long, value_parser, num_args(0..=1), help = "Quicken multiple repos at the same time, use --mass or --mass <regex>")]
        mass: Option<Option<String>>,
    },
    Profile {
        #[clap(subcommand)]
        sub: ProfileCommands

    },
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    Activate,
    Add,
    Delete
}