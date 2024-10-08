use colored::Colorize;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::Path;
use std::{collections::HashMap, path::PathBuf};
use toml_edit::{value, DocumentMut};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub active: bool,
    pub pulloption: String,
    pub username: String,
    pub email: String,
    pub baseaddress: String,
    pub provider: String,
    pub token: String,
    pub targetbasepath: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    config_file_path: PathBuf,
}

impl Config {
    // Creates a new Config by loading the profiles from the given file
    pub fn new() -> Self {
        let config_file_path: PathBuf = Self::get_default_config_path();
        if !config_file_path.exists() {
            Self::create_empty_config_file(&config_file_path);
        }
        let profiles: HashMap<String, Profile> = Self::load_profiles(&config_file_path);
        Config {
            profiles,
            config_file_path,
        }
    }

    fn get_default_config_path() -> PathBuf {
        let mut config_path: PathBuf = home_dir().expect("Failed to get home directory");
        config_path.push(".config");
        config_path.push("grgry.toml");
        config_path
    }

    fn create_empty_config_file(config_file_path: &Path) {
        if let Some(parent) = config_file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create config directory");
        }
        File::create(config_file_path).expect("Failed to create config file");
    }

    pub fn reload(&mut self) {
        self.profiles = Self::load_profiles(&self.config_file_path);
    }

    fn load_profiles<P: AsRef<Path>>(config_file_path: P) -> HashMap<String, Profile> {
        let toml_content: String =
            fs::read_to_string(config_file_path).expect("Failed to read file");
        let doc: DocumentMut = toml_content
            .parse::<DocumentMut>()
            .expect("Failed to parse TOML");

        //TODO on load check if not empty (VALID)
        let mut profiles: HashMap<String, Profile> = HashMap::new();
        for (key, value) in doc.iter() {
            let profile: Profile = Profile {
                active: value["active"].as_bool().unwrap_or(false),
                pulloption: value["pulloption"].as_str().unwrap_or("").to_string(),
                username: value["username"].as_str().unwrap_or("").to_string(),
                email: value["email"].as_str().unwrap_or("").to_string(),
                baseaddress: value["baseaddress"].as_str().unwrap_or("").to_string(),
                targetbasepath: value["targetbasepath"].as_str().unwrap_or("").to_string(),
                provider: value["provider"].as_str().unwrap_or("").to_string(),
                token: value["token"].as_str().unwrap_or("").to_string(),
            };
            profiles.insert(key.to_string(), profile);
        }
        profiles
    }

    // Activates the profile specified by `choice`
    pub fn activate_profile(&mut self, choice: &str) {
        for (key, profile) in self.profiles.iter_mut() {
            profile.active = key == choice;
        }
        self.save_config();
    }

    // Deletes the profile specified by `choice`
    pub fn delete_profile(&mut self, choice: &str) {
        self.profiles.remove(choice);
        self.save_config();
    }

    // Saves the current state of profiles back to the config file
    pub fn save_config(&self) {
        let toml_content: String =
            fs::read_to_string(&self.config_file_path).expect("Failed to read file");
        let mut doc: DocumentMut = toml_content
            .parse::<DocumentMut>()
            .expect("Failed to parse TOML");

        // Remove profiles that are no longer in the HashMap
        let existing_keys: Vec<String> = doc.iter().map(|(key, _)| key.to_string()).collect();
        for key in existing_keys {
            if !self.profiles.contains_key(&key) {
                doc.remove(&key);
            }
        }

        // Update or create profiles in the TOML document
        for (key, profile) in self.profiles.iter() {
            if let Some(profile_table) = doc[key].as_table_mut() {
                profile_table["active"] = value(profile.active);
                profile_table["pulloption"] = value(&profile.pulloption);
                profile_table["username"] = value(&profile.username);
                profile_table["email"] = value(&profile.email);
                profile_table["baseaddress"] = value(&profile.baseaddress);
                profile_table["targetbasepath"] = value(&profile.targetbasepath);
                profile_table["provider"] = value(&profile.provider);
                profile_table["token"] = value(&profile.token);
            } else {
                // If the profile does not exist, create it
                let mut new_profile = toml_edit::Table::new();
                new_profile["active"] = value(profile.active);
                new_profile["pulloption"] = value(&profile.pulloption);
                new_profile["username"] = value(&profile.username);
                new_profile["email"] = value(&profile.email);
                new_profile["baseaddress"] = value(&profile.baseaddress);
                new_profile["targetbasepath"] = value(&profile.targetbasepath);
                new_profile["provider"] = value(&profile.provider);
                new_profile["token"] = value(&profile.token);
                doc[key] = toml_edit::Item::Table(new_profile);
            }
        }

        fs::write(&self.config_file_path, doc.to_string()).expect("Failed to write file");
    }

    pub fn active_profile(&self) -> &Profile {
        match self
            .profiles
            .values()
            .find(|profile: &&Profile| profile.active)
        {
            Some(profile) => profile,
            None => {
                eprintln!(
                    "{}",
                    "One profile needs to be activated. For activating a profile use grgry profile activate!".red()
                );
                std::process::exit(1);
            }
        }
    }

    pub fn find_profiles_by_provider(&self, remote_origin_url: &str) -> Vec<&str> {
        self.profiles
            .iter()
            .filter_map(|(key, profile)| {
                if remote_origin_url.contains(&profile.provider) {
                    Some(key.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}
