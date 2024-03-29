use std::{
    error::Error,
    fs,
    sync::mpsc::Sender,
    thread::sleep,
    time::{Duration, SystemTime},
};

use crate::utils::get_data_directory;

use super::ChannelMessages;

#[derive(Debug)]
pub struct Announcement {
    pub timing: Duration,
    pub message: String,
    pub start: SystemTime,
}

impl Clone for Announcement {
    fn clone(&self) -> Self {
        Self {
            timing: self.timing,
            message: self.message.clone(),
            start: self.start,
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
                let announcement = Announcement { timing, message, start };

                announcements.push(announcement);
            }
        }
    }

    Ok(announcements)
}

pub fn start_announcements(tx: Sender<ChannelMessages>) -> Result<(), Box<dyn Error>> {
    // TODO: Add a flag here to toggle announcements on/off?

    // let mut twitch_irc = TwitchIRC::new(twitch_name, oauth_token);

    let mut announcements = get_announcements()?;

    loop {
        for announcement in announcements.iter_mut() {
            if let Ok(elapsed) = announcement.start.elapsed() {
                let time_to_announce = elapsed > announcement.timing;

                if time_to_announce {
                    announcement.start = SystemTime::now();
                    tx.send(ChannelMessages::Announcement(announcement.clone()))?;

                    // twitch_irc.send_privmsg(&announcement.message);
                };
            }
        }

        let duration = Duration::new(30, 0);
        sleep(duration);
    }
}
