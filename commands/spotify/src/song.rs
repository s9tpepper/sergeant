use crate::login::get_spotify;

pub async fn song() -> anyhow::Result<()> {
    let mut spotify = get_spotify().await?;

    let track = spotify.get_currently_playing_track(Some("us")).await;
    if track.is_err() {
        println!("{}", track.unwrap_err());

        return Ok(());
    }

    let track = track?;

    #[allow(clippy::single_match)]
    match track.item {
        Some(item) => match item {
            spotify_rs::model::PlayableItem::Track(track) => {
                let artists: Vec<String> = track.artists.iter().map(|artist| artist.name.clone()).collect();

                println!(
                    "{} - {} - {} - Preview Link: {}",
                    artists.join(","),
                    track.name,
                    track.external_urls.spotify,
                    track.preview_url.unwrap_or("".to_string()),
                );
            }

            spotify_rs::model::PlayableItem::Episode(_) => {}
        },
        None => {}
    };

    Ok(())
}
