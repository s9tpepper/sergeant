use std::io::ErrorKind;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{self, Duration};
use std::{error::Error, fs::OpenOptions, io::Write};

use serde::{Deserialize, Serialize};
use serde_json::json;
use tungstenite::connect;
use tungstenite::Error::{AlreadyClosed, ConnectionClosed, Io};
use tungstenite::Message::{self, Close, Ping, Text};

use crate::commands::get_reward;
use crate::utils::get_data_directory;

use super::ChannelMessages;

#[derive(Serialize, Deserialize, Debug)]
pub struct TwitchApiResponse<T> {
    pub data: T,
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
    pub id: String,
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
    let now = time::SystemTime::now();

    let log = format!("{:?} - {}: {}\n", now, err, json);
    let mut error_log = get_data_directory(Some("error_log")).unwrap();
    error_log.push("log.txt");

    let mut file = OpenOptions::new().create(true).append(true).open(error_log).unwrap();
    let _ = file.write_all(log.as_bytes());
}

fn handle_message(message: Message, tx: &Sender<ChannelMessages>) -> Result<(), Box<dyn Error>> {
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

            'commands: {
                match sub_message.data {
                    SubMessage::Points(ref sub_message) => {
                        let Ok(command_name) = get_reward(&sub_message.redemption.reward.title) else {
                            break 'commands;
                        };

                        let Some(user_input) = &sub_message.redemption.user_input else {
                            break 'commands;
                        };

                        let command_result = Command::new(&command_name).arg(user_input).output();
                        match command_result {
                            Ok(_) => {}
                            Err(_) => {
                                // TODO: Refund the points if the command fails

                                send_to_error_log(
                                    command_name.to_string(),
                                    format!("Error running reward command with input: {}", user_input),
                                );
                            }
                        }
                    }

                    SubMessage::Sub(ref _sub_message) => {
                        // let message = format!(
                        //     "{} has subscribed for {} months, currently on a {} month steak.",
                        //     sub_message.message.display_name,
                        //     sub_message.message.cumulative_months,
                        //     sub_message.message.streak_months
                        // );
                        //
                        // println!("{}", message.to_string().blue().bold());
                        //
                        // Ok(())
                    }

                    SubMessage::Bits(ref _sub_message) => {
                        // let message = format!(
                        //     "{} has cheered {} bits",
                        //     sub_message.data.user_name, sub_message.data.bits_used
                        // );
                        //
                        // println!("{}", message.to_string().white().on_green().bold());
                        // Ok(())
                    }
                }
            }

            tx.send(ChannelMessages::MessageData(sub_message))?;

            Ok(())
        }
        other => {
            send_to_error_log(other.to_string(), "Unknown Error".into());
            Err("Not a message".into())
        }
    }
}

pub fn connect_to_pub_sub(
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    tx: Sender<ChannelMessages>,
) -> Result<(), Box<dyn Error>> {
    let get_users_url = "https://api.twitch.tv/helix/users";
    let response = ureq::get(get_users_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", &client_id.to_string())
        .call();

    let Ok(response) = response else {
        return Err("Failed to get user data".into());
    };

    let mut response: TwitchApiResponse<Vec<User>> = serde_json::from_reader(response.into_reader())?;

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

            let stream = socket.get_ref();
            let timeout_duration = Duration::new(60 * 4, 0);
            match stream {
                tungstenite::stream::MaybeTlsStream::Plain(stream) => {
                    let _ = stream.set_read_timeout(Some(timeout_duration));
                }

                tungstenite::stream::MaybeTlsStream::NativeTls(stream) => {
                    match stream.get_ref().set_read_timeout(Some(timeout_duration)) {
                        Ok(it) => it,
                        Err(_err) => {}
                    }
                }

                _ => {}
            }

            loop {
                let _ = socket.send("PING".into());

                match socket.read() {
                    Ok(message) => match message {
                        Text(message) => {
                            let _ = handle_message(Message::Text(message), &tx);
                        }
                        Close(_) => {
                            send_to_error_log("We got a close message, reconnecting...".to_string(), "".to_string());

                            return connect_to_pub_sub(oauth_token, client_id, tx);
                        }

                        Ping(_) => {}

                        wtf => {
                            send_to_error_log("HOW? Why are we here???".to_string(), wtf.to_string());

                            return connect_to_pub_sub(oauth_token, client_id, tx);
                        }
                    },
                    Err(error) => {
                        send_to_error_log(error.to_string(), "Mistakes were made".to_string());

                        match error {
                            ConnectionClosed | AlreadyClosed => {
                                return connect_to_pub_sub(oauth_token, client_id, tx);
                            }

                            Io(error) => {
                                if error.kind() != ErrorKind::WouldBlock {
                                    return connect_to_pub_sub(oauth_token, client_id, tx);
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }

        Err(error) => {
            send_to_error_log(error.to_string(), "Could not connect to pub sub".to_string());

            connect_to_pub_sub(oauth_token, client_id, tx)
        }
    }
}
