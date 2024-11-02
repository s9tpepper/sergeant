use crate::login::get_spotify;

pub async fn queue(link: &str) -> anyhow::Result<()> {
    let mut spotify = get_spotify().await?;

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
