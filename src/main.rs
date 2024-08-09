mod commands;
mod config;

use clap::parser::ValueSource;
use clap::Parser;
use commands::Commands;
use inquire::formatter::OptionFormatter;
use inquire::validator::Validation;
use inquire::Confirm;
use rayon::str;
use serde::Deserialize;
use toml_edit::DocumentMut;
use toml_edit::Key;
use std::collections::HashMap;
use std::process::Command;
use std::io;
use std::process;
use std::fs;
use config_file::FromConfigFile;
use inquire::{
    error::{CustomUserError, InquireResult, InquireError},
    required, CustomType, MultiSelect, Select, Text,
};
use toml_edit::{Document, value};
use rayon::prelude::*;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use lazy_static::lazy_static;
use std::sync::Mutex;
use config::Config;
use config::Profile;

#[derive(Parser)]
#[command(name = "grgry")]
#[command(about = "A CLI tool for various tasks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// #[derive(Deserialize, Clone)]
// pub struct Config {
//     #[serde(flatten)]
//     profiles: HashMap<String, Profile>,
// }

// #[derive(Debug, Deserialize, Clone)]
// struct Profile {
//     active: bool,
//     pulloption: String, //This should be either ssh or https
//     username: String, //Add here the username for author
//     email: String, //Add here the email for author
//     baseaddress: String, //This is the base url of the git instance e.g. https://github.com or https://gitlab.com
//     managertype: String, //This is the type of version control manager either github, gitlab, bitbucket ...
//     token: String //This is the user token which should allow api calls for mass cloning
// }

// lazy_static!{
//     pub static ref CONFIG: Mutex<Config> = Mutex::new(Config::from_config_file("/home/axgno01/.config/grgry.toml").unwrap()); 
// }

fn main() {
    let mut config: Config = Config::new("/home/axgno01/.config/grgry.toml");
    let cli = Cli::parse();

    match &cli.command {
        Commands::Clone { directory, force, branch } => {
            // for (key, profile) in &config.profiles {
            //     println!("Profile: {} - {:?}", key, profile);
            // }
            // let active_profile = get_active_profile(&CONFIG.lock().unwrap());
            // println!("Profile: {:?}", active_profile);
            clone(directory, *force);
        },
        Commands::Reclone { directory, force } => {
            reclone(directory, *force);
        },
        Commands::Commit { directory, force, message, recursive, quick } => {
            clone(directory, *force);
        },
        Commands::Quick { message, force, mass } => {
            let _ = quick(message, *force, mass, config);
        },
        Commands::Profile { sub } => 
        // let profile_keys: Vec<String>;
        // let profiles_ref;
        {
            // println!("Hello");
            // let config_guard = CONFIG.lock().unwrap();
            // println!("yeee");
            // // println!("{:?}", config_guard);
            // let profiles_keys: Vec<&str> = config_guard.profiles.keys().map(|key| key.as_str()).collect();
            // let profiles_ref = &mut config_guard.profiles;
            match &sub {
                // commands::ProfileCommands::Activate => {
                //     // let mut config_guard = CONFIG.lock().unwrap();
                //     activate_profile_prompt(profiles_keys, &mut config_guard.profiles)
                // },
                // commands::ProfileCommands::Add => {
                //     let mut config_guard = CONFIG.lock().unwrap();
                //     println!("test");
                //     add_profile(profiles_keys, &mut config_guard.profiles)
                // },
                // commands::ProfileCommands::Delete => {
                //     let mut config_guard = CONFIG.lock().unwrap();
                //     delete_profile_prompt(profiles_keys, &mut config_guard.profiles)},
    
                commands::ProfileCommands::Activate => activate_profile_prompt(&mut config),
                commands::ProfileCommands::Add => add_profile(&mut config),
                commands::ProfileCommands::Delete => delete_profile_prompt(&mut config),
            }
        }
    }
}

// fn get_active_profile(config: &Config) -> &Profile {
//     match config.profiles.values().find(|profile| profile.active) {
//         Some(profile) => profile,
//         None => {
//             eprintln!("One profile needs to be activated. For activating a profile use grgry activate!");
//             process::exit(1);
//         }
//     }
// }

fn delete_profile_prompt(config: &mut Config) {
    let test = do_clone(&config.profiles);
    let profile_keys: Vec<&str> = test.keys().map(|key| key.as_str()).collect();
    let ans: Result<&str, InquireError> = Select::new("Which profile do you choose?", profile_keys).prompt();
    
    match ans {
        Ok(choice) => {
            config.delete_profile(choice);
        
            config.save_config();
            
            println!("{:?}", config);
        },
        Err(_) => println!("There was an error, please try again"),
    }
}

// fn delete_profile(profiles: &mut HashMap<String, Profile>, choice: &str) {
//     profiles.remove(choice);
// }

fn add_profile(config: &mut Config) {
    let _profile_name = Text::new("profile name:")
        .with_validator(required!("This field is required"))
        //TODO: would be nice to validate if the profile already exists but does not work due to my rust incapabilities
        // .with_validator(
        //     move |name: &str| {
        //     if choices.iter().any(|e| name.contains(e)) {
        //         Ok(Validation::Invalid("not exists".into()))
        //     } else {
        //         Ok(Validation::Valid)
        //     }
        // })
        .with_help_message("Optional notes")
        .prompt();
    match _profile_name {Ok(_) => {}, Err(_) => process::exit(1)};

    let _user_name = Text::new("user name:")
        .with_help_message("Optional notes")
        .with_default("")
        .prompt();
    match _user_name {Ok(_) => {}, Err(_) => process::exit(1)};

    let _user_email = Text::new("user email:")
        .with_help_message("Optional notes")
        .with_default("")
        .prompt();
    match _user_email {Ok(_) => {}, Err(_) => process::exit(1)};

    let manager_types: Vec<&str> = vec!["github", "gitlab"];
    let managertype: Result<&str, InquireError> = Select::new("choose manager type", manager_types).prompt();
    match managertype {Ok(_) => {}, Err(_) => process::exit(1)};

    let pull_options: Vec<&str> = vec!["ssh", "https"];
    let pulloption: Result<&str, InquireError> = Select::new("choose pull option", pull_options).prompt();
    match pulloption {Ok(_) => {}, Err(_) => process::exit(1)};

    let _baseaddress = Text::new("base address:")
        .with_validator(required!("This field is required"))
        .with_help_message("Optional notes")
        .prompt();
    match _baseaddress {Ok(_) => {}, Err(_) => process::exit(1)};

    let _token = Text::new("token:")
        .with_help_message("Optional notes")
        .with_default("")
        .prompt();
    match _token {Ok(_) => {}, Err(_) => process::exit(1)};

    let _activate = Confirm::new("Do you want to activate the profile?")
        .with_default(false)
        .prompt();
    match _activate {Ok(_) => {}, Err(_) => process::exit(1)};

    let profile_name = _profile_name.unwrap();
    config.profiles.insert(profile_name.clone(), Profile { active: false, pulloption: String::from(pulloption.unwrap()),  username: _user_name.unwrap(), email: _user_email.unwrap(), baseaddress: _baseaddress.unwrap(), managertype: String::from(managertype.unwrap()), token: _token.unwrap()});
    match _activate {
        Ok(true) => {
            config.activate_profile(&profile_name.clone());
        }
        _ => {}
    }
    config.save_config();
    println!("{:?}", config);

}

fn do_clone<K: Clone, V: Clone>(data: &HashMap<K,V>) -> HashMap<K, V> {
    data.clone()
}
fn activate_profile_prompt(config: &mut Config) {
    let test = do_clone(&config.profiles);
    let profile_keys: Vec<&str> = test.keys().map(|key| key.as_str()).collect();

    // let test: Vec<&str> = profile_keys.into_iter().map(String::as_str).collect();
    let ans: Result<&str, InquireError> = Select::new("Which profile do you choose?", profile_keys).prompt();
    
    match ans {
        Ok(choice) => {
            config.activate_profile(choice);
            config.save_config();
            
            println!("{:?}", config);
            // activate_profile(config, choice);
        
            // save_profiles(profiles);
        },
        Err(_) => println!("There was an error, please try again"),
    }
}

// fn activate_profile(profiles: &mut HashMap<String, Profile>, choice: &str) {
//     for (key, profile) in profiles.iter_mut() {
//         profile.active = key == choice;
//     }
// }

// fn save_profiles(profiles: &mut HashMap<String, Profile>) {
//     let file_path = "/home/axgno01/.config/grgry.toml";
//     let toml_content = fs::read_to_string(file_path).expect("Failed to read file");
//     let mut doc = toml_content.parse::<DocumentMut>().expect("Failed to parse TOML");

//     // Remove profiles that are no longer in the HashMap
//     let existing_keys: Vec<String> = doc.iter().map(|(key, _)| key.to_string()).collect();

//     for key in existing_keys {
//         if !profiles.contains_key(&key) {
//             doc.remove(&key);
//         }
//     }

//     for (key, profile) in profiles.iter() {
//         if let Some(profile_table) = doc[key].as_table_mut() {
//             profile_table["active"] = value(profile.active);
//             profile_table["pulloption"] = value(&profile.pulloption);
//             profile_table["username"] = value(&profile.username);
//             profile_table["email"] = value(&profile.email);
//             profile_table["baseaddress"] = value(&profile.baseaddress);
//             profile_table["managertype"] = value(&profile.managertype);
//             profile_table["token"] = value(&profile.token);
//         } else {
//             // If the profile does not exist, create it
//             let mut new_profile = toml_edit::Table::new();
//             new_profile["active"] = value(profile.active);
//             new_profile["pulloption"] = value(&profile.pulloption);
//             new_profile["username"] = value(&profile.username);
//             new_profile["email"] = value(&profile.email);
//             new_profile["baseaddress"] = value(&profile.baseaddress);
//             new_profile["managertype"] = value(&profile.managertype);
//             new_profile["token"] = value(&profile.token);
//             doc[key] = toml_edit::Item::Table(new_profile);
//         }
//     }
//     fs::write(file_path, doc.to_string()).expect("Failed to write file");
// }

fn quick(message: &str, force: bool, mass: &Option<Option<String>>, config: Config) -> io::Result<()>{
    println!("Quick with message: {}", message);
    if force {
        println!("Force option is enabled.");
    }
    let mass_val = match mass {
        Some(Some(mass_value)) => mass_value.to_string(), // If the user provided a value, use it
        Some(None) => String::from(".*"),      // If the user provided the flag but no value, use ".*"
        None => String::from("false"),        // If the user didn't provide the flag, use "false" meaning only current folder
    };
    println!("{}", mass_val);
    let repos = find_git_repos_parallel(None, &mass_val);
    for repo in repos {
        match has_changes() {
            Ok(true) => {
                println!("There are changes in the repository {}", repo.clone().into_os_string().into_string().unwrap());
                let _activate = Confirm::new("Do you want to quicken the repository?")
                .with_default(false)
                .prompt();
                match _activate {Ok(_) => {
                    run_cmd_s(Command::new("git").args(&["-C", &repo.clone().into_os_string().into_string().unwrap(), "add", "."]), false);
                    run_cmd_s(Command::new("git").args(&["-C", &repo.clone().into_os_string().into_string().unwrap(), "commit", "--author", &format!("{} <{}>", config.active_profile().username, config.active_profile().email), "-m", message]), false);
                    let current_branch = run_cmd_o(Command::new("git").args(&["branch", "--show-current"]), false);
                    println!("Current branch: {}", &current_branch);
                    let branch_exists = false;// check_branch_exists_on_origin(&current_branch);
                    println!("Current branch exists?: {}", branch_exists);
                    run_cmd_s(Command::new("git").args(push_to_origin(&repo.clone().into_os_string().into_string().unwrap(), &current_branch, !branch_exists)), false); //TODO here please right push command
                    
                }, Err(_) => continue};
            },
            Ok(false) => {},
            Err(e) => eprintln!("Error checking repository: {}", e),
        }
    }
    Ok(())
}

fn command_to_string(command: &Command) -> String {
    let cmd_str = format!("{:?}", command);
    cmd_str
}

// fn run_command(command: &mut Command, test: bool) -> io::Result<()> {
//     if test {
//         let cmd_str = command_to_string(command);
//         println!("Executing: {}", cmd_str);
//         Ok(())    
//     }
//     else {
//         let output = command.output()?;
//         if output.status.success() {
//             Ok(())
//         } else {
//             let stderr = String::from_utf8_lossy(&output.stderr);
//             Err(io::Error::new(io::ErrorKind::Other, format!("Command failed: {}", stderr)))
//         }    
//     }
// }

fn run_cmd_o(command: &mut Command, test: bool) -> String {
    if test {
        let cmd_str = command_to_string(command);
        println!("Executing: {}", cmd_str);
        return String::from("");
    } else {
        let output = command.output().expect("Failed to execute command!");
        if !output.status.success() {
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        }
    
        return String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}

fn run_cmd_s(command: &mut Command, test: bool) {
    if test {
        let cmd_str = command_to_string(command);
        println!("Executing: {}", cmd_str);
    } else {
        let status = command.status().expect("Failed to execute command!");
        if !status.success() {
            eprintln!("Error executing command");
            std::process::exit(1);
        }
    }
}

fn has_changes() -> io::Result<bool> {
    let output = Command::new("git")
    .arg("status")
    .arg("--porcelain")
    .output()?;

    if output.status.success() {
        // Check if there is any output
        Ok(!output.stdout.is_empty())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(io::Error::new(io::ErrorKind::Other, format!("Failed to run git status: {}", stderr)))
    }
}

fn clone(directory: &str, force: bool) {
    println!("Cloning into directory: {}", directory);
    if force {
        println!("Force option is enabled.");
    }
    // Implement your clone logic here
}

fn reclone(directory: &str, force: bool) {
    println!("Recloning into directory: {}", directory);
    if force {
        println!("Force option is enabled.");
    }
    // Implement your reclone logic here
}

fn find_git_repos_parallel(root: Option<&Path>, pattern: &str) -> Vec<PathBuf> {
    let root = root.unwrap_or(Path::new("."));
    if pattern == "false" {
        return vec![root.to_path_buf()];
 
    }
    let regex = Regex::new(pattern).expect("Invalid regex pattern");

    WalkDir::new(root)
        .into_iter()
        .par_bridge() // Convert iterator to a parallel iterator
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_dir())
        .filter(|entry| {
            let path = entry.path();
            // Check if the directory contains a .git folder
            path.join(".git").is_dir()
        })
        .filter(|entry| {
            let path_str = entry.path().to_string_lossy();
            // Check if the path matches the regex pattern
            regex.is_match(&path_str)
        })
        .map(|entry| entry.into_path())
        .collect()
}

fn push_to_origin(repo_path: &str, branch: &str, set_upstream: bool) -> Vec<String> {

    let mut args = vec!["-C".to_string(), repo_path.to_string(), "push".to_string(), "origin".to_string(), branch.to_string()];

    if set_upstream {
        args.insert(3, "--set-upstream".to_string());
    }

    return args;
}