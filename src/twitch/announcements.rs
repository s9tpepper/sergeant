use std::{
    error::Error,
    fs,
    sync::{mpsc::Sender, Arc},
    thread::sleep,
    time::{Duration, SystemTime},
};

use ratatui::layout::Rect;
use serde::{Deserialize, Serialize};

use crate::utils::get_data_directory;

use super::{irc::TwitchIRC, ChannelMessages};

#[derive(Debug, Deserialize, Serialize)]
pub struct Announcement {
    pub timing: Duration,
    pub message: String,
    pub start: SystemTime,
    #[serde(skip)]
    pub area: Option<Rect>,
}

impl Clone for Announcement {
    fn clone(&self) -> Self {
        Self {
            timing: self.timing,
            message: self.message.clone(),
            start: self.start,
            area: self.area,
        }
    }
}

pub fn get_announcements() -> Result<Vec<Announcement>, Box<dyn Error>> {
    let announcements_dir = get_data_directory(Some("chat_announcements"))?;

    let mut announcements = vec![];
    let dir_entries = fs::read_dir(announcements_dir)?;
    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let file_contents = fs::read_to_string(&path)?;
            if let Some((timing, message)) = file_contents.split_once('\n') {
                let timing = Duration::from_secs(timing.parse::<u64>()? * 60);
                let start = SystemTime::now();
                let message = message.to_string();
                let area = None;
                let announcement = Announcement {
                    timing,
                    message,
                    start,
                    area,
                };

                announcements.push(announcement);
            }
        }
    }

    Ok(announcements)
}

pub fn start_announcements(
    twitch_name: Arc<String>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    tx: Sender<ChannelMessages>,
    skip_announcements: bool,
) -> Result<(), Box<dyn Error>> {
    if skip_announcements {
        return Ok(());
    }

    let mut twitch_irc = TwitchIRC::new(twitch_name, oauth_token, client_id, tx);
    let mut announcements = get_announcements()?;

    loop {
        let new_announcements = get_announcements()?;
        if announcements.len() != new_announcements.len() {
            announcements = new_announcements;
        }

        for announcement in announcements.iter_mut() {
            if let Ok(elapsed) = announcement.start.elapsed() {
                let time_to_announce = elapsed > announcement.timing;

                if time_to_announce {
                    announcement.start = SystemTime::now();

                    twitch_irc.send_privmsg(&announcement.message);
                };
            }
        }

        let duration = Duration::new(30, 0);
        sleep(duration);
    }
}
