use clap::Subcommand;
#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Clone a repository into the specified directory")]
    Clone {
        #[arg(
            value_name = "DIRECTORY",
            required = true,
            help = "The group / org / user / repo to clone"
        )]
        directory: String,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Specify if the directory is a user directory or not (default false)"
        )]
        user: bool,

        #[arg(
            short,
            long,
            default_value = "",
            help = "Clone specific branch (default no specific branch)"
        )]
        branch: String,

        #[arg(
            long,
            default_value = ".*",
            help = "Filter the directory via regex (default .*)"
        )]
        regex: String,
    },
    #[command(about = "Make git add, git commit, git push in one go.")]
    Quick {
        #[arg(
            value_name = "MESSAGE",
            required = true,
            help = "Commit message same as git commit -m <MESSAGE>."
        )]
        message: String,

        #[arg(short, long, default_value_t = false)]
        force: bool,

        #[arg(long, value_parser, num_args(0..=1), help = "Quicken multiple repos at the same time, use --mass or --mass <regex>")]
        mass: Option<Option<String>>,
    },
    Mass {
        #[arg(
            value_name = "COMMAND",
            required = true,
            help = "This is the command to execute as if it was a git command without git prefix"
        )]
        command: String,

        #[arg(
            value_name = "REGEX",
            default_value = ".*",
            help = "Filter the repositories via regex (default .*)"
        )]
        regex: String,

        #[arg(short, long, default_value_t = false)]
        interactive: bool,
    },
    Profile {
        #[clap(subcommand)]
        sub: ProfileCommands,
    },
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    Activate,
    Add,
    Delete,
}
