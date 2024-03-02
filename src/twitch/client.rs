use colored::*;
use futures_util::StreamExt;
use irc::client::prelude::Config;
use irc::client::{prelude::Client, ClientStream, Sender};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::time::{Duration, SystemTime};

use crate::commands::get_list_commands;
use crate::utils::get_data_directory;

use super::messages::TwitchMessage;
use super::messages::{parse, TwitchApiResponse};

const TWITCH_IRC_SERVER: &str = "irc.chat.twitch.tv";

pub struct TwitchClient {
    // config: Config,
    client: Client,
    sender: Sender,
    stream: ClientStream,
    channels: Vec<String>,
    twitch_name: String,
    oauth_token: String,
    client_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GetUsersResponse {
    id: String,
    login: String,
    display_name: String,
    r#type: String,
    broadcaster_type: String,
    description: String,
    profile_image_url: String,
    offline_image_url: String,
    created_at: String,
}

pub struct Announcement {
    pub timing: Duration,
    pub message: String,
    pub start: SystemTime,
}

impl TwitchClient {
    pub async fn new(
        twitch_name: String,
        oauth_token: String,
        client_id: String,
        mut channels: Vec<String>,
    ) -> Result<TwitchClient, Box<dyn Error>> {
        // If channels are not defined then default to the twitch user's channel
        if channels.is_empty() {
            channels.push(format!("#{}", twitch_name));
        }

        let config = Config {
            nickname: Some(twitch_name.to_owned()),
            username: Some(twitch_name.to_owned()),
            password: Some(oauth_token.to_owned()),
            server: Some(TWITCH_IRC_SERVER.to_string()),
            port: Some(6697),
            channels: channels.clone(),
            ..Config::default()
        };

        let mut client = Client::from_config(config.clone()).await?;
        client.identify()?;

        let stream = client.stream()?;
        let sender = client.sender();

        let twitch_client = TwitchClient {
            // config,
            client,
            sender,
            stream,
            channels,
            twitch_name,
            oauth_token,
            client_id,
        };

        Ok(twitch_client)
    }

    async fn get_user_id(&self) -> Result<GetUsersResponse, Box<dyn Error>> {
        let get_users_url = "https://api.twitch.tv/helix/users";
        let client_id = &self.client_id;
        let mut response = reqwest::Client::new()
            .get(get_users_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.oauth_token.replace("oauth:", "")),
            )
            .header("Client-Id", client_id)
            .send()
            .await?
            .json::<TwitchApiResponse<Vec<GetUsersResponse>>>()
            .await?;

        let user = response.data.swap_remove(0);
        Ok(user)
    }

    pub async fn start_receiving(
        &mut self,
        announcements: &mut Vec<Announcement>,
    ) -> Result<(), Box<dyn Error>> {
        let mut start = SystemTime::now();
        // Ask Twitch for more capabilities so we can receive message tags
        self.sender
            .send("CAP REQ :twitch.tv/commands twitch.tv/tags")?;

        let users_response = self.get_user_id().await?;

        while let Some(message) = self.stream.next().await.transpose()? {
            if let Ok(parsed_message) = parse(message).await {
                self.print_message(&parsed_message);
                self.print_raid_message(&parsed_message, &users_response)
                    .await?;
                let _ = self.check_for_announcements(announcements, &mut start);
            }
        }

        Ok(())
    }

    pub fn get_announcements(&mut self) -> Result<Vec<Announcement>, Box<dyn Error>> {
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
                    let announcement = Announcement {
                        timing,
                        message,
                        start,
                    };

                    announcements.push(announcement);
                }
            }
        }

        Ok(announcements)
    }

    fn check_for_announcements(
        &self,
        announcements: &mut Vec<Announcement>,
        start: &mut SystemTime,
    ) -> Result<(), Box<dyn Error>> {
        let channel = &self.channels[0];
        for announcement in announcements {
            if let Ok(elapsed) = start.elapsed() {
                let time_to_announce = elapsed > announcement.timing;

                if time_to_announce {
                    announcement.start = SystemTime::now();
                    self.client.send_privmsg(channel, &announcement.message)?;
                };
            }
        }

        Ok(())
    }

    async fn send_shoutout(
        &self,
        raid_user_id: &str,
        users_response: &GetUsersResponse,
    ) -> Result<(), Box<dyn Error>> {
        let shoutout_api = "https://api.twitch.tv/helix/chat/shoutouts";
        let broadcaster_id = &users_response.id;
        let bearer = format!("Bearer {}", &self.oauth_token.replace("oauth:", ""));
        let _response = reqwest::Client::new()
            .post(shoutout_api)
            .header("Authorization", bearer)
            .header("Client-Id", &self.client_id)
            .query(&[
                ("from_broadcaster_id", broadcaster_id),
                ("to_broadcaster_id", &raid_user_id.to_string()),
            ])
            .send()
            .await?;

        Ok(())
    }

    async fn print_raid_message(
        &self,
        twitch_message: &TwitchMessage,
        users_response: &GetUsersResponse,
    ) -> Result<(), Box<dyn Error>> {
        if let TwitchMessage::RaidMessage {
            raid_notice,
            user_id,
        } = twitch_message
        {
            let first_time_msg = "🪂 Raid!:".to_string().truecolor(255, 255, 0).bold();
            println!("{}", first_time_msg);
            println!("{}", raid_notice.replace("\\s", " "));

            self.send_shoutout(user_id, users_response).await?;
        }

        Ok(())
    }

    fn print_message(&self, twitch_message: &TwitchMessage) {
        let TwitchMessage::PrivMessage { message } = twitch_message else {
            return;
        };

        let (r, g, b) = message.get_nickname_color().to_owned();
        let nickname = &message.display_name;

        let nick = nickname.truecolor(r, g, b).bold();
        let final_message = format!("{nick}: {}", message.message);

        if message.first_msg {
            let first_time_msg = "✨ First Time Chat:"
                .to_string()
                .truecolor(255, 255, 0)
                .bold();
            println!("{}", first_time_msg);
        }

        println!("{final_message}");

        self.check_for_chat_commands(&message.message, &message.channel);
    }

    fn check_for_chat_commands(&self, message: &str, channel: &str) {
        let commands_list = get_list_commands();
        if let Ok(list) = &commands_list {
            for item in list {
                let command = format!("!{}", item);
                if message == command {
                    let _ = self.output_chat_command(item, channel);
                }
            }
        }
    }

    fn output_chat_command(&self, command: &str, channel: &str) -> Result<(), Box<dyn Error>> {
        let mut data_dir = get_data_directory(Some("chat_commands"))?;
        data_dir.push(command);

        let message = fs::read_to_string(data_dir)?;

        let _ = self
            .client
            .send_privmsg(channel, format!("[bot] {}", message));

        Ok(())
    }

    pub fn send_message(&self, msg: &str) -> Result<(), Box<dyn Error>> {
        self.sender.send(msg)?;

        Ok(())
    }
}
