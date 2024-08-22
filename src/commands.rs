use std::default;

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

        #[clap(flatten)]
        regex_args: Regex,
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

        #[arg(short, long, default_value_t = false, help = "Don't ask for permission to execute command per repository")]
        skip_interactive: bool,
    },
    Mass {
        #[arg(
            value_name = "COMMAND",
            required = true,
            help = "This is the command to execute as if it was a git command without git prefix"
        )]
        command: String,

        #[clap(flatten)]
        regex_args: Regex,

        #[arg(short, long, default_value_t = false, help = "Don't ask for permission to execute command per repository")]
        skip_interactive: bool,
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

#[derive(Debug, clap::Args)]
#[group(multiple = false)]
pub struct Regex {
    /// Argument1.
    #[clap(long, value_parser, num_args(0..=1),
        help = "Use regex to execute command en mass. Without option it searches for all repos.")]
    regex: Option<Option<String>>,
    /// Argument2.
    #[clap(long,
        help = "Use regex to execute command en mass excluding matching repos.")]
    rev_regex: Option<String>,
}

impl Regex {
    pub fn get_regex_args(&self, default: &str) -> (String, bool) {
        // Determine the value and whether reverse is true
        if let Some(rev_regex) = &self.rev_regex {
            (rev_regex.clone(), true)
        } else if let Some(regex) = &self.regex {
            match regex {
                Some(pattern) => (pattern.to_string(), false), // If the user provided a value, use it
                None => (String::from(".*"), false), // If the user provided the flag but no value, use ".*"
            }
        } else {
            (default.to_string(), false)
        }
    }
}