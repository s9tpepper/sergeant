use colored::*;
use futures_util::StreamExt;
use irc::client::prelude::Config;
use irc::client::{prelude::Client, ClientStream, Sender};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::sync::Arc;

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
    // channels: Vec<String>,
    // twitch_name: Arc<String>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub r#type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub created_at: String,
}

impl TwitchClient {
    pub async fn new(
        twitch_name: Arc<String>,
        oauth_token: Arc<String>,
        client_id: Arc<String>,
        mut channels: Vec<String>,
    ) -> Result<TwitchClient, Box<dyn Error>> {
        // If channels are not defined then default to the twitch user's channel
        if channels.is_empty() {
            channels.push(format!("#{}", twitch_name));
        }

        let config = Config {
            nickname: Some(twitch_name.to_string().to_owned()),
            username: Some(twitch_name.to_string().to_owned()),
            password: Some(oauth_token.to_string().to_owned()),
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
            // channels,
            // twitch_name,
            oauth_token,
            client_id,
        };

        Ok(twitch_client)
    }

    async fn get_user_id(&self) -> Result<User, Box<dyn Error>> {
        let get_users_url = "https://api.twitch.tv/helix/users";
        let client_id = &self.client_id;
        let mut response = reqwest::Client::new()
            .get(get_users_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.oauth_token.replace("oauth:", "")),
            )
            .header("Client-Id", client_id.to_string())
            .send()
            .await?
            .json::<TwitchApiResponse<Vec<User>>>()
            .await?;

        let user = response.data.swap_remove(0);
        Ok(user)
    }

    pub async fn start_receiving(&mut self) -> Result<(), Box<dyn Error>> {
        // Ask Twitch for more capabilities so we can receive message tags
        self.sender.send("CAP REQ :twitch.tv/commands twitch.tv/tags")?;

        let users_response = self.get_user_id().await?;

        while let Some(message) = self.stream.next().await.transpose()? {
            if let Ok(parsed_message) = parse(message).await {
                self.print_message(&parsed_message);
                self.print_raid_message(&parsed_message, &users_response).await?;
            }
        }

        Ok(())
    }

    async fn send_shoutout(&self, raid_user_id: &str, users_response: &User) -> Result<(), Box<dyn Error>> {
        let shoutout_api = "https://api.twitch.tv/helix/chat/shoutouts";
        let broadcaster_id = &users_response.id;
        let bearer = format!("Bearer {}", &self.oauth_token.replace("oauth:", ""));
        let _response = reqwest::Client::new()
            .post(shoutout_api)
            .header("Authorization", bearer)
            .header("Client-Id", &self.client_id.to_string())
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
        users_response: &User,
    ) -> Result<(), Box<dyn Error>> {
        if let TwitchMessage::RaidMessage { raid_notice, user_id } = twitch_message {
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
            let first_time_msg = "✨ First Time Chat:".to_string().truecolor(255, 255, 0).bold();
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

        let _ = self.client.send_privmsg(channel, format!("[bot] {}", message));

        Ok(())
    }

    pub fn send_message(&self, msg: &str) -> Result<(), Box<dyn Error>> {
        self.sender.send(msg)?;

        Ok(())
    }
}
