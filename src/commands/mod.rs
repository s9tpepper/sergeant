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
