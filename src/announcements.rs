use std::{
    sync::mpsc::Sender,
    thread::sleep,
    time::{Duration, SystemTime},
};

use clap::builder::OsStr;
use serde::{Deserialize, Serialize};

use crate::{
    channel::ChannelMessages, fs::get_data_directory, twitch::eventsub::notifications::prelude::send_to_channels,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Announcement {
    pub name: String,
    pub timing: Duration,
    pub message: String,
    pub start: SystemTime,
}

impl Clone for Announcement {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            timing: self.timing,
            message: self.message.clone(),
            start: self.start,
        }
    }
}

pub fn start_announcements(
    tui_tx: Sender<ChannelMessages>,
    websocket_tx: Sender<ChannelMessages>,
    skip_announcements: bool,
) -> anyhow::Result<()> {
    if skip_announcements {
        return Ok(());
    }

    let mut announcements = get_announcements()?;

    loop {
        let new_announcements = get_announcements()?;
        if announcements.len() != new_announcements.len() {
            announcements = new_announcements;
        }

        for announcement in announcements.iter_mut() {
            let time_to_announce = check_announcement(announcement);

            if !time_to_announce {
                continue;
            };

            announcement.start = SystemTime::now();

            let channel_message = ChannelMessages::BotAnnouncement {
                message: announcement.message.clone(),
            };

            let _ = send_to_channels(channel_message, &tui_tx, &websocket_tx, "Bot Announcements");
        }

        sleep(Duration::from_secs(30));
    }
}

fn check_announcement(announcement: &Announcement) -> bool {
    if let Ok(elapsed) = announcement.start.elapsed() {
        return elapsed > announcement.timing;
    }

    false
}

pub fn get_announcements() -> anyhow::Result<Vec<Announcement>> {
    let announcements_dir = get_data_directory(Some("chat_announcements"))?;

    let mut announcements = vec![];
    let dir_entries = std::fs::read_dir(announcements_dir)?;
    for entry in dir_entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let default_file_name = &OsStr::from("unknown_file");
        let name = path.file_name().unwrap_or(default_file_name);

        let file_contents = std::fs::read_to_string(&path)?;
        let Some((timing, message)) = file_contents.split_once('\n') else {
            continue;
        };

        let timing = Duration::from_secs(timing.parse::<u64>()? * 60);
        let start = SystemTime::now();
        let message = message.to_string();
        let announcement = Announcement {
            name: String::from(name.to_str().unwrap_or("unknown_name")),
            timing,
            message,
            start,
        };

        announcements.push(announcement);
    }

    Ok(announcements)
}
