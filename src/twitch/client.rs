use irc::client::{prelude::Config, Client, Sender, ClientStream};
use irc::proto::Command;
use futures_util::StreamExt;
use std::error::Error;
use colored::*;

use super::messages::{TwitchMessage, BadgeItem};
use super::messages::messages::parse;

const TWITCH_IRC_SERVER:&str = "irc.chat.twitch.tv";

pub struct TwitchClient {
    // config: Config,
    // client: Client,
    sender: Sender,
    stream: ClientStream,
    badges: Vec<BadgeItem>,
}

impl TwitchClient {
    pub async fn new(
        twitch_name: String,
        oauth_token: String,
        mut channels: Vec<String>,
        badges: Vec<BadgeItem>) -> Result<TwitchClient, Box<dyn Error>> {

        // If channels are not defined then default to the twitch user's channel 
        if channels.len() == 0 {
            channels.push(format!("#{}", twitch_name));
        }

        let config = Config {
            nickname: Some(twitch_name.to_owned()),
            username: Some(twitch_name.to_owned()),
            password: Some(oauth_token.to_owned()),
            server: Some(TWITCH_IRC_SERVER.to_string()),
            port: Some(6697),
            channels,
            ..Config::default()
        };

        let mut client = Client::from_config(config.clone()).await?;
        client.identify()?;

        let stream = client.stream()?;
        let sender = client.sender();

        let twitch_client = TwitchClient {
            // config,
            // client,
            sender,
            stream,
            badges,
        };

        Ok(twitch_client)
    }

    pub async fn start_receiving(&mut self) -> Result<(), Box<dyn Error>> {
        // Ask Twitch for more capabilities so we can receive message tags
        self.sender.send("CAP REQ :twitch.tv/commands twitch.tv/tags")?;

        while let Some(message) = self.stream.next().await.transpose()? {
            if let Command::PRIVMSG(ref _sender, ref _msg) = message.command {
                self.print_message(parse(message).await?);
            }
        }

        Ok(())
    }

    fn print_message(&self, twitch_message: TwitchMessage) {
        let nickname = twitch_message.display_name.clone().unwrap_or("unknown_soldier".to_string());
        let mut message = twitch_message.message.clone().unwrap();
        let (r, g, b) = twitch_message.get_nickname_color().to_owned();

        if message.ends_with("\n\r") || message.ends_with("\r\n") {
            message.pop();
            message.pop();
            message.pop();
            message.pop();
        } 

        if message.ends_with("\r") || message.ends_with("\n") {
            message.pop();
            message.pop();
        }

        if message.starts_with("\r\n") || message.starts_with("\n\r") {
            message.replace_range(0..3, "");
        }

        if message.starts_with("\r") || message.starts_with("\n") {
            message.replace_range(0..1, "");
        }

        let nick = nickname.truecolor(r, g, b).bold();
        let final_message = format!("{nick}: {message}");
        println!("{final_message}");
    }


    pub fn send_message(self, msg:String) -> Result<(), Box<dyn Error>>{
        self.sender.send(msg.as_str())?;

        Ok(())
    }
}



