use crate::twitch::irc::TwitchIrcClient;

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

use super::{
    irc::{TwitchIRC, MESSAGE_DELIMITER},
    parse::{parse, TwitchMessage},
    ChannelMessages,
};

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

    let mut announcements = get_announcements()?;
    let mut twitch_irc = TwitchIRC::new(twitch_name.clone(), oauth_token.clone(), client_id.clone(), tx.clone());

    loop {
        if let Ok(message) = twitch_irc.socket.read() {
            let messages = message.to_text().unwrap().split(MESSAGE_DELIMITER);
            let messages = messages.map(tungstenite::Message::from);

            messages.for_each(|message| {
                if let tungstenite::Message::Text(new_message) = message {
                    match parse(&new_message, &mut twitch_irc) {
                        Ok(TwitchMessage::PingMessage { message }) => {
                            let pong_message = format!("PONG {message}");

                            let _ = twitch_irc.socket.send(pong_message.into());
                        }

                        // NOTE: Dont care about other messages for announcements
                        Ok(_) => {}

                        Err(_) => {}
                    }
                }
            })
        }

        let new_announcements = get_announcements()?;
        if announcements.len() != new_announcements.len() {
            announcements = new_announcements;
        }

        for announcement in announcements.iter_mut() {
            let time_to_announce = check_announcement(announcement);

            if time_to_announce {
                announcement.start = SystemTime::now();

                twitch_irc.send_privmsg(&announcement.message);
            };
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

#[test]
fn test_get_announcements() {
    let announcements = get_announcements();
    if let Ok(list) = announcements {
        dbg!(&list);

        assert!(list.len() == 1);
        return;
    }

    panic!("Couldn't get announcements");
}

#[test]
fn test_check_announcement_true() {
    let now = SystemTime::now();
    let twenty_five_mins_ago = now.checked_sub(Duration::from_secs(60 * 26));

    let announcement = Announcement {
        timing: Duration::from_secs(60 * 25),
        message: "hello".to_string(),
        start: twenty_five_mins_ago.unwrap(),
        area: None,
    };

    let time_to_announce = check_announcement(&announcement);

    assert!(time_to_announce);
}

#[test]
fn test_check_announcement_false() {
    let now = SystemTime::now();
    let five_mins_ago = now.checked_sub(Duration::from_secs(60 * 5));

    let announcement = Announcement {
        timing: Duration::from_secs(60 * 25),
        message: "hello".to_string(),
        start: five_mins_ago.unwrap(),
        area: None,
    };

    let time_to_announce = check_announcement(&announcement);

    assert!(!time_to_announce);
}
