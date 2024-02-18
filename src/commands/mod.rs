use std::{error::Error, fs};

use crate::utils::get_data_directory;

pub fn add_chat_command(command_name: String, message: String) -> Result<(), Box<dyn Error>> {
    let mut command_path = get_data_directory()?;
    command_path.push("chat_commands");

    if !command_path.exists() {
        std::fs::create_dir_all(&command_path)?;
    }

    command_path.push(command_name);
    dbg!(&command_path);

    fs::write(command_path, message)?;

    Ok(())
}

pub fn get_list_commands() -> Result<Vec<String>, Box<dyn Error>>{
    let mut command_path = get_data_directory()?;
    command_path.push("chat_commands");

    let mut commands = vec![];
    let dir_entries = fs::read_dir(command_path)?;

    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name();
            if let Some(file_name) = file_name {
                commands.push(file_name.to_string_lossy().to_string());
            }
        }
    }

    Ok(commands)
}
