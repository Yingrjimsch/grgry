use clap::Subcommand;

use crate::Profile;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Clone a repository into the specified directory")]
    Clone {
        #[arg(value_name = "DIRECTORY")]
        directory: String,

        #[arg(short, long, default_value_t = false)]
        force: bool,

        #[arg(short, long, default_value = "")]
        branch: String,
    },
    Reclone {
        #[arg(value_name = "DIRECTORY")]
        directory: String,

        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    Commit {
        #[arg(value_name = "DIRECTORY")]
        directory: String,

        #[arg(short, long, default_value = "")]
        message: String,

        #[arg(short, long, default_value_t = false)]
        recursive: bool,

        #[arg(short, long, default_value_t = false)]
        quick: bool,

        #[arg(short, long, default_value_t = false)]
        force: bool,
    },
    Quick {
        #[arg(value_name = "MESSAGE")]
        message: String,

        #[arg(short, long, default_value_t = false)]
        force: bool,

        #[arg(long, value_parser, num_args(0..=1))]
        mass: Option<Option<String>>,
    },
    Profile {
        #[clap(subcommand)]
        sub: ProfileCommands

    },
    Test
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    Activate,
    Add,
    Delete
}