use std::{
    error::Error,
    sync::Arc,
    thread::sleep,
    time::{Duration, SystemTime},
};

use super::client::TwitchClient;

pub async fn start_announcements(
    twitch_name: Arc<String>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
) -> Result<(), Box<dyn Error>> {
    // TODO: Add a flag here to toggle announcements on/off

    let mut twitch_client = TwitchClient::new(twitch_name, oauth_token, client_id, vec![]).await?;

    let mut announcements = twitch_client.get_announcements()?;
    let mut start = SystemTime::now();

    loop {
        let _ = twitch_client.check_for_announcements(&mut announcements, &mut start);

        let duration = Duration::new(30, 0);
        sleep(duration);
    }
}
