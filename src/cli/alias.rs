pub fn alias(command: Vec<String>) -> Vec<String> {
    let mut command_vec: Vec<String> = vec!["grgry".to_string(), "mass".to_string()];
    let mut mass_command: String = String::new();
    let mut command = command.into_iter();
    while let Some(arg) = command.next() {
        match arg.as_str() {
            // Check for recognized arguments
            "-s" | "--skip-interactive" | "--dry-run" => command_vec.push(arg.to_string()),
            "--regex" | "--rev-regex" => {
                command_vec.push(arg.to_string());
                let regex_arg = command.next();
                match regex_arg {
                    Some(argument) => command_vec.push(argument.to_string()),
                    _ => {}
                };
            }
            _ => {
                if !mass_command.is_empty() {
                    mass_command.push(' ');
                }
                mass_command.push_str(&arg);
            }
        }
    }
    command_vec.insert(2, mass_command);

    return command_vec;
}
