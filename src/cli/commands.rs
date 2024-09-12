use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Clone a repository into the specified directory.")]
    Clone {
        #[arg(
            value_name = "DIRECTORY",
            required = true,
            help = "The group / org / user / repo to clone."
        )]
        directory: String,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Specify if the base directory should be removed before cloning or only a pull is necessary (default false)."
        )]
        force: bool,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Specify if the directory is a user directory or not (default false)."
        )]
        user: bool,

        #[arg(
            short,
            long,
            default_value = "",
            help = "Clone specific branch (default no specific branch)."
        )]
        branch: String,

        #[clap(flatten)]
        regex_args: Regex,

        #[arg(
            long,
            default_value_t = false,
            help = "Only make a dry run and list the commands which would be executed."
        )]
        dry_run: bool,
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

        #[clap(flatten)]
        regex_args: Regex,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Don't ask for permission to execute command per repository."
        )]
        skip_interactive: bool,

        #[arg(
            long,
            default_value_t = false,
            help = "Only make a dry run and list the commands which would be executed."
        )]
        dry_run: bool,
    },
    #[command(about = "Execute a git command on a mass of repos.")]
    Mass {
        #[arg(
            value_name = "COMMAND",
            required = true,
            help = "This is the command to execute as if it was a git command without git prefix."
        )]
        command: String,

        #[clap(flatten)]
        regex_args: Regex,

        #[arg(
            short,
            long,
            default_value_t = false,
            help = "Don't ask for permission to execute command per repository."
        )]
        skip_interactive: bool,

        #[arg(
            long,
            default_value_t = false,
            help = "Only make a dry run and list the commands which would be executed."
        )]
        dry_run: bool,
    },
    #[command(
        about = "Manage your grgry profiles for different providers like github and gitlab."
    )]
    Profile {
        #[clap(subcommand)]
        sub: ProfileCommands,
    },
    #[command(about = "EXPERIMENTAL: add this as git alias to simply use git for mass commands.")]
    Alias {
        // This will collect all trailing arguments that are not part of options like `regex_args`.
        #[arg(
            trailing_var_arg = true,
            value_name = "COMMAND",
            required = true,
            help = "This is the command to execute as if it was a git command without git prefix."
        )]
        command: Vec<String>,
    },
    #[command(
        about = "EXPERIMENTAL: Update the grgry version in itself simply by calling grgry update."
    )]
    Update,
    // Test,
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    #[command(about = "Activate a profile to use.")]
    Activate,
    #[command(about = "Adding a profile interactively.")]
    Add,
    #[command(about = "Remove a unused or wrong profile.")]
    Delete,
    #[command(about = "Show the current activated profile.")]
    Show {
        #[arg(short, long, default_value_t = false, help = "Show all profiles.")]
        all: bool,
    },
}

#[derive(Debug, clap::Args)]
#[group(multiple = false)]
pub struct Regex {
    #[clap(
        long,
        help = "Use regex to execute command en mass. Without option it searches for all repos."
    )]
    regex: Option<String>,
    #[clap(
        long,
        help = "Use regex to execute command en mass excluding matching repos."
    )]
    rev_regex: Option<String>,
}

impl Regex {
    pub fn get_regex_args(&self, default: &str) -> (String, bool) {
        // Determine the value and whether reverse is true
        if let Some(rev_regex) = &self.rev_regex {
            return (rev_regex.clone(), true);
        } else if let Some(regex) = &self.regex {
            return (regex.clone(), false);
        } else {
            return (default.to_string(), false);
        }
    }
}
