use std::{error::Error, fs};

use colored::*;
use hex_rgb::Color;

use crate::{
    twitch::{
        irc::TwitchIRC,
        parse::{ChatMessage, TwitchMessage},
    },
    utils::get_data_directory,
};

pub fn output(message: TwitchMessage, client: &mut TwitchIRC) {
    if let TwitchMessage::PrivMessage { message } = message {
        print_message(&message, client);
    }
}

fn get_nickname_color(color: &str) -> (u8, u8, u8) {
    let color = Color::new(color).unwrap();

    (color.red, color.green, color.blue)
}

fn get_list(directory: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let command_path = get_data_directory(Some(directory))?;
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

pub fn get_list_commands() -> Result<Vec<String>, Box<dyn Error>> {
    get_list("chat_commands")
}

fn check_for_chat_commands(message: &str, client: &mut TwitchIRC) {
    let commands_list = get_list_commands();
    if let Ok(list) = &commands_list {
        for item in list {
            let command = format!("!{}", item);
            if message == command {
                let _ = output_chat_command(item, client);
            }
        }
    }
}

fn output_chat_command(command: &str, client: &mut TwitchIRC) -> Result<(), Box<dyn Error>> {
    let mut data_dir = get_data_directory(Some("chat_commands"))?;
    data_dir.push(command);

    let message = fs::read_to_string(data_dir)?;

    client.send_privmsg(format!("[bot] {}", message).as_str());

    Ok(())
}

fn print_message(message: &ChatMessage, client: &mut TwitchIRC) {
    let (r, g, b) = get_nickname_color(&message.color);
    let nickname = &message.nickname;

    let nick = nickname.truecolor(r, g, b).bold();
    let final_message = format!("{nick}: {}", message.message.trim());

    if message.first_msg {
        let first_time_msg = "✨ First Time Chat:".to_string().truecolor(255, 255, 0).bold();
        println!("{}", first_time_msg);
    }

    println!("{final_message}");

    check_for_chat_commands(&message.message, client);
}
