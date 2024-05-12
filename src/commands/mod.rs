use core::time::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, thread::sleep};

use base64::prelude::*;

use crate::utils::get_data_directory;

const TWITCH_SCOPES: [&str; 13] = [
    "channel:read:subscriptions",
    "chat:read",
    "chat:edit",
    "channel:moderate",
    "channel:read:redemptions",
    "channel:manage:redemptions",
    "channel:bot",
    "user:write:chat",
    "moderator:manage:shoutouts",
    "user_read",
    "chat_login",
    "bits:read",
    "channel:moderate",
];

const TWITCH_CREATE_TOKEN: &str = "https://twitchtokengenerator.com/api/create/[APP_NAME]/[SCOPES]";
const TWITCH_TOKEN_STATUS: &str = "https://twitchtokengenerator.com/api/status/[ID]";

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    success: bool,
    id: String,
    message: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TokenStatus {
    pub success: bool,
    pub id: String,

    // Success field
    pub scopes: Option<Vec<String>>,
    pub token: Option<String>,
    pub refresh: Option<String>,
    pub username: Option<String>,
    pub client_id: Option<String>,

    // Error fields
    pub message: Option<String>,
    pub error: Option<usize>,
}

pub fn add_chat_command(command_name: &str, message: &str, timing: Option<usize>) -> Result<(), Box<dyn Error>> {
    let mut target_dir = "chat_commands";

    let file_contents = if let Some(timing) = timing {
        target_dir = "chat_announcements";
        format!("{}\n{}", timing, message)
    } else {
        message.to_string()
    };

    let mut command_path = get_data_directory(Some(target_dir))?;

    if !command_path.exists() {
        std::fs::create_dir_all(&command_path)?;
    }

    command_path.push(command_name);

    fs::write(command_path, file_contents)?;

    Ok(())
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

pub fn remove_chat_command(command_name: &str) -> Result<(), Box<dyn Error>> {
    let mut command_path = get_data_directory(Some("chat_commands"))?;
    command_path.push(command_name);
    if command_path.exists() {
        return Ok(fs::remove_file(command_path)?);
    }

    let mut command_path = get_data_directory(Some("chat_announcements"))?;
    command_path.push(command_name);
    if command_path.exists() {
        return Ok(fs::remove_file(command_path)?);
    }

    Ok(())
}

pub fn get_list_announcements() -> Result<Vec<String>, Box<dyn Error>> {
    let command_path = get_data_directory(Some("chat_announcements"))?;
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

pub fn authenticate_with_twitch() -> Result<(), Box<dyn Error>> {
    let app_name = BASE64_STANDARD.encode(clap::crate_name!());
    let url = TWITCH_CREATE_TOKEN
        .replace("[APP_NAME]", &app_name)
        .replace("[SCOPES]", &TWITCH_SCOPES.join("+"));

    let token_response = ureq::get(&url).call();
    if token_response.is_err() {
        return Err("Failed to get token response".into());
    }
    let token_response = serde_json::from_str::<TokenResponse>(&token_response?.into_string()?)?;
    println!("Navigate to this url to grant a token: {}", token_response.message);

    let ten_seconds = Duration::from_secs(10);
    let max_retries = 18;
    let mut retries = 0;

    let status_id = token_response.id.as_str();
    let status_url = TWITCH_TOKEN_STATUS.replace("[ID]", status_id);
    loop {
        if retries == max_retries {
            println!("You took too long, please try again");
            break;
        }

        let token_status_response = ureq::get(&status_url).call();
        if token_status_response.is_err() {
            println!("Failed to get token status");
            return Err("token status response was bad".into());
        }

        let token_status = serde_json::from_str::<TokenStatus>(&token_status_response?.into_string()?)?;
        if token_status.success {
            store_token(token_status)?;
            break;
        }

        sleep(ten_seconds);
        retries += 1;
    }

    println!("Token has been successfully generated.");
    Ok(())
}

pub fn store_token(token_status: TokenStatus) -> Result<(), Box<dyn Error>> {
    let mut token_dir = get_data_directory(Some("token"))?;
    token_dir.push("oath_token.txt");

    fs::write(token_dir, serde_json::to_string(&token_status)?)?;

    Ok(())
}

fn list(list_type: String) {
    if let Ok(list) = get_list(&list_type) {
        let human_readable = list_type.replace('_', " ");
        if list.is_empty() {
            println!("Currently no {human_readable} have been added.");
        }

        println!("Available {human_readable}:");
        for item in list {
            println!("- {}", item);
        }
    }
}

fn get_item(item_name: &str, item_type: &str) -> Result<String, Box<dyn Error>> {
    let mut item_path = get_data_directory(Some(item_type))?;
    item_path.push(item_name);

    if item_path.exists() {
        let item = fs::read_to_string(item_path)?;
        return Ok(item);
    }

    let human_readable = item_type.replace('_', " ");
    Err(format!("No {human_readable} found").into())
}

pub fn remove_item(item_name: &str, item_type: &str) -> Result<(), Box<dyn Error>> {
    let mut item_path = get_data_directory(Some(item_type))?;
    item_path.push(item_name);
    if item_path.exists() {
        return Ok(fs::remove_file(item_path)?);
    }

    Ok(())
}

pub fn add_item(item_name: &str, cli: &str, item_type: &str) -> Result<(), Box<dyn Error>> {
    let file_contents = cli.to_string();

    let mut item_path = get_data_directory(Some(item_type))?;

    if !item_path.exists() {
        std::fs::create_dir_all(&item_path)?;
    }

    item_path.push(item_name);

    fs::write(item_path, file_contents)?;

    Ok(())
}

pub fn add_reward(reward_name: &str, cli: &str) -> Result<(), Box<dyn Error>> {
    add_item(reward_name, cli, "chat_rewards")
}

pub fn get_reward(reward_name: &str) -> Result<String, Box<dyn Error>> {
    get_item(reward_name, "chat_rewards")
}

pub fn list_rewards() {
    list("chat_rewards".to_string())
}

pub fn remove_reward(reward_name: &str) -> Result<(), Box<dyn Error>> {
    remove_item(reward_name, "chat_rewards")
}

pub fn remove_action(action_name: &str) -> Result<(), Box<dyn Error>> {
    remove_item(action_name, "irc_actions")
}

pub fn add_action(action_name: &str, cli: &str) -> Result<(), Box<dyn Error>> {
    add_item(action_name, cli, "irc_actions")
}

pub fn list_actions() {
    list("irc_actions".to_string())
}

pub fn get_action(action_name: &str) -> Result<String, Box<dyn Error>> {
    get_item(action_name, "irc_actions")
}
