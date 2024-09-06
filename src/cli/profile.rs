use std::{collections::HashMap, process};

use inquire::{required, Confirm, InquireError, Select, Text};

use crate::{config::config::{Config, Profile}, utils::helper::{prntln, MessageType}};

pub fn show_profile(all: bool, config: Config) {
    if all {
        prntln(&serde_json::to_string_pretty(&config).unwrap(), MessageType::Neutral)
    } else {
        prntln(&serde_json::to_string_pretty(&config).unwrap(), MessageType::Neutral)
    }
}

pub fn delete_profile_prompt(config: &mut Config) {
    let profiles_cloned: HashMap<String, Profile> = do_clone(&config.profiles);
    let profile_keys: Vec<&str> = profiles_cloned.keys().map(|key: &String| key.as_str()).collect();
    let profile_to_delete_key: Result<&str, InquireError> =
        Select::new("Which profile do you want to delete?", profile_keys).prompt();

    match profile_to_delete_key {
        Ok(choice) => {
            config.delete_profile(choice);
            config.save_config();
            show_profile(false, config.clone());
        }
        Err(_) => prntln("There was an error, please try again", MessageType::Error),
    }
}

pub fn add_profile_prompt(config: &mut Config) {
    let profile_name: Result<String, InquireError> = Text::new("profile name:")
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
        .with_help_message("The name of your profile (e.g. Github Profile)")
        .prompt();
    match profile_name {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let target_base_path: Result<String, InquireError> = Text::new("target base path:")
        .with_validator(required!("This field is required"))
        .with_help_message("The absolute path where the repos should be cloned to (e.g. /home/you/repos)")
        .with_default("")
        .prompt();
    match target_base_path {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let user_name: Result<String, InquireError> = Text::new("user name:")
        .with_help_message("The name shown in the commit")
        .with_default("")
        .prompt();
    match user_name {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let user_email: Result<String, InquireError> = Text::new("user email:")
        .with_help_message("The email shown in the commit")
        .with_default("")
        .prompt();
    match user_email {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let providers: Vec<&str> = vec!["github", "gitlab"];
    let provider: Result<&str, InquireError> = Select::new("choose provider", providers).prompt();
    match provider {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let pull_options: Vec<&str> = vec!["ssh", "https"];
    let pulloption: Result<&str, InquireError> =
        Select::new("choose pull option", pull_options).prompt();
    match pulloption {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let base_address: Result<String, InquireError> = Text::new("base address:")
        .with_validator(required!("This field is required"))
        .with_help_message("The base address of your provider (e.g. https://github.com)")
        .prompt();
    match base_address {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let token: Result<String, InquireError> = Text::new("token:")
        .with_help_message("The token to access the provider, if empty only public repos can be cloned")
        .with_default("")
        .prompt();
    match token {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let activate: Result<bool, InquireError> = Confirm::new("Do you want to activate the profile?")
        .with_default(false)
        .prompt();
    match activate {
        Ok(_) => {}
        Err(_) => process::exit(1),
    };

    let profile_name: String = profile_name.unwrap();
    config.profiles.insert(
        profile_name.clone(),
        Profile {
            active: false,
            pulloption: String::from(pulloption.unwrap()),
            username: user_name.unwrap(),
            targetbasepath: target_base_path.unwrap(),
            email: user_email.unwrap(),
            baseaddress: base_address.unwrap(),
            provider: String::from(provider.unwrap()),
            token: token.unwrap(),
        },
    );
    match activate {
        Ok(true) => {
            config.activate_profile(&profile_name.clone());
        }
        _ => {}
    }
    config.save_config();
    println!("{}", serde_json::to_string_pretty(&config).unwrap());
}

fn do_clone<K: Clone, V: Clone>(data: &HashMap<K, V>) -> HashMap<K, V> {
    data.clone()
}

pub fn activate_profile_prompt(config: &mut Config) {
    let profiles_cloned: HashMap<String, Profile> = do_clone(&config.profiles);
    let profile_keys: Vec<&str> = profiles_cloned.keys().map(|key: &String| key.as_str()).collect();

    let profile_to_activate_key: Result<&str, InquireError> =
        Select::new("Choose profile to activate:", profile_keys).prompt();
    match profile_to_activate_key {
        Ok(choice) => {
            config.activate_profile(choice);
            config.save_config();
            prntln(&format!("{} {}", "Activated profile is:", choice), MessageType::Success);
        }
        Err(_) => prntln("Active profile could not be changed! Make sure there you have a profile configured with grgry profile add.", MessageType::Error),
    }
}