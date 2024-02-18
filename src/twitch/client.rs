use colored::*;
use futures_util::StreamExt;
use irc::client::prelude::Config;
use irc::client::{prelude::Client, ClientStream, Sender};
use irc::proto::Command;
use std::error::Error;
use std::fs;

use crate::commands::get_list_commands;
use crate::utils::get_data_directory;

use super::messages::parse;
use super::messages::TwitchMessage;

const TWITCH_IRC_SERVER: &str = "irc.chat.twitch.tv";

pub struct TwitchClient {
    // config: Config,
    client: Client,
    sender: Sender,
    stream: ClientStream,
}

impl TwitchClient {
    pub async fn new(
        twitch_name: String,
        oauth_token: String,
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
        };

        Ok(twitch_client)
    }

    pub async fn start_receiving(&mut self) -> Result<(), Box<dyn Error>> {
        // Ask Twitch for more capabilities so we can receive message tags
        self.sender
            .send("CAP REQ :twitch.tv/commands twitch.tv/tags")?;

        while let Some(message) = self.stream.next().await.transpose()? {
            if let Command::PRIVMSG(ref _sender, ref _msg) = message.command {
                self.print_message(parse(message).await?);
            }
        }

        Ok(())
    }

    fn print_message(&self, twitch_message: TwitchMessage) {
        let (r, g, b) = twitch_message.get_nickname_color().to_owned();
        let nickname = twitch_message.display_name;

        let nick = nickname.truecolor(r, g, b).bold();
        let final_message = format!("{nick}: {}", twitch_message.message);
        println!("{final_message}");

        self.check_for_chat_commands(&twitch_message.message, &twitch_message.channel);
    }

    fn check_for_chat_commands(&self, message: &str, channel: &str) {
        let commands_list = get_list_commands();
        if let Ok(list) = &commands_list {
            for item in list {
                let command = format!("!{}", item);
                if message.contains(&command) {
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
