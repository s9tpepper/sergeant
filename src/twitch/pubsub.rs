use std::{error::Error, fs::OpenOptions, io::Write, process::Command, sync::Arc};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tungstenite::connect;
use tungstenite::Message::{self, Close, Text};

use crate::{commands::get_reward, utils::get_data_directory};

use super::{client::User, messages::TwitchApiResponse};

#[derive(Deserialize, Serialize, Debug)]
pub struct SocketMessage {
    r#type: String,
    data: SocketMessageData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SocketMessageData {
    topic: String,
    message: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MessageData {
    pub data: SubMessage,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum SubMessage {
    Points(Box<ChannelPointsData>),
    Sub(SubscribeEvent),
    Bits(BitsEvent),
    // Bits {},
    // BitsUnlocks {},
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BitsEvent {
    pub is_anonymous: bool,
    pub message_type: String,
    pub data: BitsEventData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct BitsEventData {
    pub user_name: String,
    pub chat_message: String,
    pub bits_used: u64,
    pub total_bits_used: u64,
    pub context: String, // cheer
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscribeEvent {
    pub topic: String,
    pub message: SubscribeMessage,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubscribeMessage {
    pub display_name: String,   // some_person
    pub cumulative_months: u64, // 9
    pub streak_months: u64,     // 3
    pub context: String,        // subgift, resub
    pub sub_message: String,    // A message, possibly with emotes
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ChannelPointsData {
    pub timestamp: String,
    pub redemption: Redemption,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserReference {
    pub id: String,
    pub login: String,
    pub display_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Redemption {
    pub user: UserReference,
    pub user_input: Option<String>,
    pub status: String,
    pub reward: Reward,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Reward {
    pub title: String,
    pub prompt: String,
    pub cost: u64,
}

pub fn send_to_error_log(err: String, json: String) {
    let log = format!("{}: {}\n", err, json);
    let mut error_log = get_data_directory(Some("error_log")).unwrap();
    error_log.push("log.txt");

    let mut file = OpenOptions::new().create(true).append(true).open(error_log).unwrap();
    let _ = file.write_all(log.as_bytes());
}

fn handle_message(message: Message) -> Result<(), Box<dyn Error>> {
    match message {
        Message::Text(message) => {
            if !message.contains("MESSAGE") {
                return Err("Not a message".into());
            }

            let socket_message = serde_json::from_str::<SocketMessage>(&message.to_string());
            let Ok(socket_message) = socket_message else {
                let log = socket_message.unwrap_err().to_string();
                send_to_error_log(log, message.to_string());
                return Err("Not a message".into());
            };

            let sub_message = &socket_message.data.message;
            let Ok(sub_message) = serde_json::from_str::<MessageData>(sub_message) else {
                send_to_error_log(sub_message.to_string(), message.to_string());
                return Err("Not a message".into());
            };

            match sub_message.data {
                SubMessage::Points(sub_message) => {
                    let message = format!(
                        "{} redeemed {} for {}",
                        sub_message.redemption.user.display_name,
                        sub_message.redemption.reward.title,
                        sub_message.redemption.reward.cost
                    );

                    println!("{}", message.to_string().green().bold());

                    if let Ok(command_name) = get_reward(&sub_message.redemption.reward.title) {
                        if let Some(user_input) = sub_message.redemption.user_input {
                            let _ = Command::new(command_name).arg(user_input).output();
                        }

                        return Ok(());
                    }

                    Ok(())
                }

                SubMessage::Sub(sub_message) => {
                    let message = format!(
                        "{} has subscribed for {} months, currently on a {} month steak.",
                        sub_message.message.display_name,
                        sub_message.message.cumulative_months,
                        sub_message.message.streak_months
                    );

                    println!("{}", message.to_string().blue().bold());

                    Ok(())
                }

                SubMessage::Bits(sub_message) => {
                    let message = format!(
                        "{} has cheered {} bits",
                        sub_message.data.user_name, sub_message.data.bits_used
                    );

                    println!("{}", message.to_string().white().on_green().bold());
                    Ok(())
                }
            }
        }
        _ => Err("Not a message".into()),
    }
}

pub fn connect_to_pub_sub(oauth_token: Arc<String>, client_id: Arc<String>) -> Result<(), Box<dyn Error>> {
    let get_users_url = "https://api.twitch.tv/helix/users";
    let mut response = reqwest::blocking::Client::new()
        .get(get_users_url)
        .header("Authorization", format!("Bearer {}", oauth_token.replace("oauth:", "")))
        .header("Client-Id", client_id.to_string())
        .send()?
        .json::<TwitchApiResponse<Vec<User>>>()?;

    let user = response.data.swap_remove(0);
    let twitch_pub_sub = "wss://pubsub-edge.twitch.tv";

    match connect(twitch_pub_sub) {
        Ok((mut socket, _response)) => {
            let channel_bits = "channel-bits-events-v2.".to_string() + &user.id;
            // let channel_bits_unlocks = "channel-bits-badge-unlocks.".to_string() + &user.id;
            let channel_points = "channel-points-channel-v1.".to_string() + &user.id;
            let channel_subscribe = "channel-subscribe-events-v1.".to_string() + &user.id;

            let auth_token = oauth_token.to_string().replace("oauth:", "");

            let topics_message = json!({
                "type": "LISTEN",
                "data": {
                    "auth_token": auth_token,
                    "topics": [channel_bits, channel_points, channel_subscribe]
                }
            });

            socket.send(topics_message.to_string().into()).unwrap();

            loop {
                if let Ok(message) = socket.read() {
                    match message {
                        Text(message) => {
                            let _ = handle_message(Message::Text(message));
                        }
                        Close(_) => {
                            send_to_error_log("We got a close message, reconnecting...".to_string(), "".to_string());

                            return connect_to_pub_sub(oauth_token, client_id);
                        }
                        _ => {}
                    }
                }
            }
        }

        Err(error) => {
            send_to_error_log(error.to_string(), "Could not connect to pub sub".to_string());

            connect_to_pub_sub(oauth_token, client_id)
        }
    }
}
