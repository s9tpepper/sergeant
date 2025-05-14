use std::sync::OnceLock;

use anyhow::bail;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TwitchApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub r#type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
struct SendMessage {
    broadcaster_id: String,
    sender_id: String,
    message: String,
}

static USER: OnceLock<User> = OnceLock::new();

pub fn send_message(oauth_token: &str, client_id: &str, message: &str) -> anyhow::Result<()> {
    USER.get_or_init(|| get_user(oauth_token, client_id).unwrap());

    let send_message_url = "https://api.twitch.tv/helix/chat/messages";
    let Some(user) = USER.get() else {
        bail!("Could not load user");
    };

    let body = SendMessage {
        broadcaster_id: user.id.to_string(),
        sender_id: user.id.to_string(),
        message: message.to_string(),
    };

    let response = ureq::post(send_message_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", client_id)
        .send_json(body)?;

    if response.status() != 200 {
        let error_message = response.status_text().to_string();
        bail!(error_message);
    }

    Ok(())
}

pub fn get_user_by_login(user_login: &str, oauth_token: &str, client_id: &str) -> anyhow::Result<User> {
    let get_users_url = format!("https://api.twitch.tv/helix/users?login={user_login}");
    let response = ureq::get(&get_users_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", client_id)
        .call();

    let Ok(response) = response else {
        bail!("Failed to get user data");
    };

    let mut response: TwitchApiResponse<Vec<User>> = serde_json::from_reader(response.into_reader())?;

    let user = response.data.swap_remove(0);

    Ok(user)
}

pub fn get_user(oauth_token: &str, client_id: &str) -> anyhow::Result<User> {
    let get_users_url = "https://api.twitch.tv/helix/users";
    let response = ureq::get(get_users_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", client_id)
        .call();

    let Ok(response) = response else {
        bail!("Failed to get user data");
    };

    let mut response: TwitchApiResponse<Vec<User>> = serde_json::from_reader(response.into_reader())?;

    let user = response.data.swap_remove(0);

    Ok(user)
}
