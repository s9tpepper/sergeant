use std::error::Error;

use crate::commands::{store_token, TokenStatus};

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
