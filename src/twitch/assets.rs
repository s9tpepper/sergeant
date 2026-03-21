use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::{fs::get_data_directory, image_protocols::get_iterm_image_encoding};

use super::api::{TwitchApiResponse, get_user};

#[derive(Default, Clone, Serialize, Deserialize, Debug, Hash, Eq, PartialEq)]
pub struct BadgeVersion {
    pub id: String,
    // title: String,
    // description: String,
    // click_action: String,
    // click_url: String,
    pub image_url_1x: String,
    pub image_url_2x: Option<String>,
    //pub image_url_4x: String,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct BadgeItem {
    pub set_id: String,
    pub versions: Vec<BadgeVersion>,
}

pub fn get_global_badges(token: &str, client_id: &str) -> anyhow::Result<HashMap<String, BadgeItem>> {
    // Global badges: https://api.twitch.tv/helix/chat/badges/global
    // oauth:141241241241241
    //
    // scopes:
    // chat:read+chat:edit+channel:moderate+channel:read:redemptions+channel:bot+user:write:chat
    // base64: encoded app title
    // https://twitchtokengenerator.com/api/create
    //
    let response = ureq::get("https://api.twitch.tv/helix/chat/badges/global")
        .set("Authorization", &format!("Bearer {}", token.replace("oauth:", "")))
        .set("Client-Id", client_id)
        .call()?;

    // println!("{}", response.into_string()?);

    let mut response: TwitchApiResponse<Vec<BadgeItem>> = serde_json::from_reader(response.into_reader())?;

    let data_dir = get_data_directory(Some("badges"))?;

    for badge_item in response.data.iter_mut() {
        for version in badge_item.versions.iter_mut() {
            let file_name = format!("{}_{}.txt", badge_item.set_id, version.id);
            let badge_path = data_dir.join(file_name);
            if !badge_path.exists() {
                generate_badge_file(badge_path, version)?;
            }
        }
    }

    let mut badge_map: HashMap<String, BadgeItem> = HashMap::new();
    for badge_item in response.data {
        badge_map.insert(badge_item.set_id.clone(), badge_item);
    }

    Ok(badge_map)
}

pub fn get_channel_badges(client_id: &str, oauth_token: &str) -> anyhow::Result<HashMap<String, BadgeItem>> {
    // Get channel badges
    let user = get_user(oauth_token, client_id)?;
    let channel_badges = ureq::get(
        format!(
            "https://api.twitch.tv/helix/chat/badges?broadcaster_id={}",
            user.id.as_str()
        )
        .as_str(),
    )
    .set("Client-ID", client_id)
    .set(
        "Authorization",
        &format!("Bearer {}", oauth_token.replace("oauth:", "")),
    )
    .call()?;

    let mut response: TwitchApiResponse<Vec<BadgeItem>> = serde_json::from_reader(channel_badges.into_reader())?;

    let data_dir = get_data_directory(Some("badges"))?;

    for badge_item in response.data.iter_mut() {
        for version in badge_item.versions.iter_mut() {
            let file_name = format!("{}_{}.txt", badge_item.set_id, version.id);
            let badge_path = data_dir.join(file_name);
            if !badge_path.exists() {
                generate_badge_file(badge_path, version)?;
            }
        }
    }

    let mut badge_map: HashMap<String, BadgeItem> = HashMap::new();

    for badge_item in response.data {
        badge_map.insert(badge_item.set_id.clone(), badge_item);
    }

    Ok(badge_map)
}

// TODO: Update this to be able to support multiple image protocols
pub fn get_badge_from_disk(badge_item: &BadgeItem) -> anyhow::Result<String> {
    let data_dir = get_data_directory(Some("badges"))?;

    let Some(version) = badge_item.versions.first() else {
        bail!("No badge version found");
    };

    let file_name = format!("{}_{}.txt", badge_item.set_id, version.id);
    let badge_path = data_dir.join(file_name);

    if !badge_path.exists() {
        bail!("No badge contents found");
    }

    Ok(fs::read_to_string(badge_path)?)
}

// TODO: Update this to be able to support multiple image protocols
fn generate_badge_file(badge_path: PathBuf, version: &BadgeVersion) -> anyhow::Result<()> {
    let image_url = match &version.image_url_2x {
        Some(url) => url,
        None => &version.image_url_1x,
    };

    let encoded_image = get_iterm_image_encoding(image_url)?;
    fs::write(badge_path, encoded_image)?;

    Ok(())
}
