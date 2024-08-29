use colored::*; // Make sure to add `colored` crate to Cargo.toml

enum MessageType {
    Error,
    Success,
    Neutral,
}

impl MessageType {
    fn colorize(&self, message: &str) -> String {
        match self {
            MessageType::Error => message.red().to_string(),
            MessageType::Success => message.green().to_string(),
            MessageType::Neutral => message.to_string(),
        }
    }
}

fn prntln(message: &str, message_type: MessageType) {
    println!("{}", message_type.colorize(message))
}