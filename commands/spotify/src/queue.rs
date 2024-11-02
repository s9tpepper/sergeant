use lib::fs::get_project_directory;
use spotify_rs::client::Client;

use crate::login::{get_auth_code_flow, Tokens};

pub async fn queue(link: &str) -> anyhow::Result<()> {
    let file_dir = get_project_directory("SgtSpotify", "tokens")?;
    let file_path = file_dir.join("tokens.json");

    let serialized = std::fs::read_to_string(file_path)?;
    let tokens = serde_json::from_str::<Tokens>(&serialized)?;

    let auth_flow = get_auth_code_flow(&tokens.spotify_details)?;
    let mut spotify = Client::from_refresh_token(auth_flow, true, tokens.refresh_token).await?;

    let mut uri = link.to_string();
    let web_url = "https://open.spotify.com/track/";

    if link.starts_with(web_url) {
        let cleanup = link.replace(web_url, "");

        if let Some((id, _)) = cleanup.split_once("?") {
            uri = format!("spotify:track:{id}");
        }
    }

    spotify.add_item_to_queue(uri).send().await?;

    Ok(())
}
