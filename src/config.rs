use std::collections::HashMap;
use std::fs;
use toml_edit::{value, DocumentMut};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub active: bool,
    pub pulloption: String,
    pub username: String,
    pub email: String,
    pub baseaddress: String,
    pub managertype: String,
    pub token: String,
    pub targetbasepath: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    config_file_path: String,
}

impl Config {
    // Creates a new Config by loading the profiles from the given file
    pub fn new(config_file_path: &str) -> Self {
        let profiles = Self::load_profiles(config_file_path);
        Config {
            profiles,
            config_file_path: config_file_path.to_string(),
        }
    }

    pub fn reload(&mut self) {
        self.profiles = Self::load_profiles(&self.config_file_path);
    }

    fn load_profiles(config_file_path: &str) -> HashMap<String, Profile> {
        let toml_content = fs::read_to_string(config_file_path).expect("Failed to read file");
        let doc = toml_content.parse::<DocumentMut>().expect("Failed to parse TOML");

        //TODO on load check if not empty (VALID)
        let mut profiles = HashMap::new();
        for (key, value) in doc.iter() {
            let profile = Profile {
                active: value["active"].as_bool().unwrap_or(false),
                pulloption: value["pulloption"].as_str().unwrap_or("").to_string(),
                username: value["username"].as_str().unwrap_or("").to_string(),
                email: value["email"].as_str().unwrap_or("").to_string(),
                baseaddress: value["baseaddress"].as_str().unwrap_or("").to_string(),
                targetbasepath: value["targetbasepath"].as_str().unwrap_or("").to_string(), 
                managertype: value["managertype"].as_str().unwrap_or("").to_string(),
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
        let mut toml_content = fs::read_to_string(&self.config_file_path).expect("Failed to read file");
        let mut doc = toml_content.parse::<DocumentMut>().expect("Failed to parse TOML");

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
                profile_table["managertype"] = value(&profile.managertype);
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
                new_profile["managertype"] = value(&profile.managertype);
                new_profile["token"] = value(&profile.token);
                doc[key] = toml_edit::Item::Table(new_profile);
            }
        }

        fs::write(&self.config_file_path, doc.to_string()).expect("Failed to write file");
    }

    pub fn active_profile(&self) -> &Profile {
    match self.profiles.values().find(|profile| profile.active) {
        Some(profile) => profile,
        None => {
            eprintln!("One profile needs to be activated. For activating a profile use grgry activate!");
            std::process::exit(1);
        }
    }
}
}
