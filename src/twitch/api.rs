use std::error::Error;

use serde::{Deserialize, Serialize};

use crate::commands::{store_token, TokenStatus};

use super::pubsub::{send_to_error_log, Credentials};

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

pub fn get_user_profile(id: &str, credentials: &Credentials) -> Result<Option<String>, Box<dyn Error>> {
    let api_url = "https://api.twitch.tv/helix/users";
    let response = ureq::get(api_url)
        .set(
            "Authorization",
            &format!("Bearer {}", credentials.oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", credentials.client_id.as_str())
        .query_pairs(vec![("id", id)])
        .call();

    if response.is_err() {
        send_to_error_log("Error getting user profile pic".to_string(), format!("{response:?}"));
        return Ok(None);
    }

    let response = response.unwrap();
    send_to_error_log("user profile pic response:".to_string(), format!("{response:?}"));

    let mut response: TwitchApiResponse<Vec<User>> = serde_json::from_reader(response.into_reader())?;
    let user = response.data.swap_remove(0);

    Ok(Some(user.profile_image_url))
}

pub fn get_user(oauth_token: &str, client_id: &str) -> Result<User, Box<dyn Error>> {
    let get_users_url = "https://api.twitch.tv/helix/users";
    let response = ureq::get(get_users_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", client_id)
        .call();

    let Ok(response) = response else {
        return Err("Failed to get user data".into());
    };

    let mut response: TwitchApiResponse<Vec<User>> = serde_json::from_reader(response.into_reader())?;

    let user = response.data.swap_remove(0);

    Ok(user)
}

pub fn validate(oauth_token: &str) -> Result<(), Box<dyn Error>> {
    let url = "https://id.twitch.tv/oauth2/validate";
    let token = oauth_token.replace("oauth:", "");
    let response = ureq::get(url).set("Authorization", &format!("OAuth {}", token)).call();

    let is_ok = response.is_ok();
    let status = response.unwrap().status();

    if is_ok && status == 200 {
        Ok(())
    } else {
        Err(format!("Failed to validate token: {status}").into())
    }
}

pub fn refresh_token(refresh_token: &str) -> Result<TokenStatus, Box<dyn Error>> {
    let url = format!("https://twitchtokengenerator.com/api/refresh/{refresh_token}");
    let response = ureq::get(&url).call();

    if response.is_err() {
        return Err("Token refresh has failed.".into());
    }

    let token_status = serde_json::from_str::<TokenStatus>(&response?.into_string()?)?;
    if token_status.success {
        // TODO: Fix this so it doesnt need a clone
        store_token(token_status.clone())?;
    }

    Ok(token_status)
}
