use core::time::*;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, thread::sleep};

use base64::prelude::*;

use crate::utils::get_data_directory;

const TWITCH_CREATE_TOKEN: &str = "https://twitchtokengenerator.com/api/create/[APP_NAME]/[SCOPES]";
const TWITCH_SCOPES: &str =
    "chat:read+chat:edit+channel:moderate+channel:read:redemptions+channel:bot+user:write:chat+moderator:manage:shoutouts";
const TWITCH_TOKEN_STATUS: &str = "https://twitchtokengenerator.com/api/status/[ID]";
// const TWITCH_TOKEN_REFRESH: &str = "https://twitchtokengenerator.com/api/refresh/[REFRESH_TOKEN]";

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    success: bool,
    id: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

pub fn add_chat_command(
    command_name: &str,
    message: &str,
    timing: Option<usize>,
) -> Result<(), Box<dyn Error>> {
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

pub fn get_list_commands() -> Result<Vec<String>, Box<dyn Error>> {
    let command_path = get_data_directory(Some("chat_commands"))?;
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

pub async fn authenticate_with_twitch() -> Result<(), Box<dyn Error>> {
    let app_name = BASE64_STANDARD.encode(clap::crate_name!());
    let url = TWITCH_CREATE_TOKEN
        .replace("[APP_NAME]", &app_name)
        .replace("[SCOPES]", TWITCH_SCOPES);

    let token_response = reqwest::get(url).await?.json::<TokenResponse>().await?;
    println!(
        "Navigate to this url to grant a token: {}",
        token_response.message
    );

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

        let token_status_response = reqwest::get(&status_url).await;
        if token_status_response.is_err() {
            dbg!(&token_status_response);
            return Err("token status response was bad".into());
        }

        let token_status_serializing = token_status_response?.json::<TokenStatus>().await;
        if token_status_serializing.is_err() {
            dbg!(&token_status_serializing);
            return Err("token serialization failed".into());
        }

        let token_status = token_status_serializing?;
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

fn store_token(token_status: TokenStatus) -> Result<(), Box<dyn Error>> {
    let mut token_dir = get_data_directory(Some("token"))?;
    token_dir.push("oath_token.txt");

    fs::write(token_dir, serde_json::to_string(&token_status)?)?;

    Ok(())
}
