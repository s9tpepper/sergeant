use std::io::ErrorKind;
use std::process::{self, Command};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{self, Duration};
use std::vec;
use std::{error::Error, fs::OpenOptions, io::Write};

use ratatui::buffer::Buffer;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Widget};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tungstenite::connect;
use tungstenite::Error::{AlreadyClosed, ConnectionClosed, Io};
use tungstenite::Message::{self, Close, Ping, Text};

use crate::commands::{get_action, get_reward};
use crate::tui::{MessageParts, Symbol};
use crate::utils::get_data_directory;

use super::parse::{
    get_lines, get_message_symbols, get_screen_lines, write_to_buffer, RedeemMessage, RenderCursor, TwitchMessage,
};
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

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct MessageData {
    pub data: SubMessage,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum SubMessage {
    Points(Box<ChannelPointsData>),
    Sub(SubscribeEvent),
    Bits(BitsEvent),
    // Bits {},
    // BitsUnlocks {},
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BitsEvent {
    #[serde(skip)]
    pub area: Option<Rect>,
    pub is_anonymous: bool,
    pub message_type: String,
    pub data: BitsEventData,
}

impl Widget for &mut BitsEvent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bits_from = if self.is_anonymous {
            "Anonymous"
        } else {
            &self.data.user_name
        };

        let message = format!(
            "{} has cheered {} bits. They have cheered a total of {} bits in this channel.",
            bits_from, self.data.bits_used, self.data.total_bits_used
        );

        let mut cursor = RenderCursor {
            x: area.left(),
            y: area.bottom().saturating_sub(1),
        };

        // Render the bits details in gray
        let bits_details_message: Vec<Symbol> = get_message_symbols(&message, &mut [], Some((128, 128, 128)));

        // Shrink horizontal area by 4 to make space for border and scroll bar
        let mut line_area = area;
        line_area.width = area.width - 4;

        let mut lines: Vec<Vec<MessageParts>> = get_lines(&bits_details_message, &line_area);

        // Get symbols for cheer message in white
        let message_symbols: Vec<Symbol> = get_message_symbols(&self.data.chat_message, &mut [], Some((255, 255, 255)));
        // Get lines for cheer message
        let mut message_lines: Vec<Vec<MessageParts>> = get_lines(&message_symbols, &line_area);

        // Add cheer message lines to the bits details
        lines.append(&mut message_lines);

        let mut screen_lines = get_screen_lines(&mut lines, &line_area);

        // Move cursor one over to make space for border
        cursor.x = area.left() + 1;
        cursor.y = cursor.y.saturating_sub(lines.len() as u16);

        write_to_buffer(&mut screen_lines, buf, &mut cursor);

        cursor.x = 0;
        cursor.y -= screen_lines.len() as u16;

        let block_area = Rect {
            x: 0,
            y: cursor.y.saturating_sub(1),
            width: area.width.saturating_sub(2),
            height: screen_lines.len() as u16 + 2,
        };

        // Purple border
        Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::reset().fg(Color::Rgb(138, 43, 226)))
            .title("♦️ Cheer!")
            .render(block_area, buf);

        self.area = Some(Rect {
            x: 0,
            y: cursor.y,
            width: area.width,
            height: screen_lines.len() as u16,
        });
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BitsEventData {
    pub user_name: String,
    pub chat_message: String,
    pub bits_used: u64,
    pub total_bits_used: u64,
    pub context: String, // cheer
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SubscribeEvent {
    #[serde(skip)]
    pub area: Option<Rect>,
    pub topic: String,
    pub message: SubscribeMessage,
}

impl Widget for &mut SubscribeEvent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let message = format!(
            "{} has subscribed for {} months, currently on a {} month streak.",
            self.message.display_name,
            self.message.cumulative_months,
            self.message.streak_months,
            // self.message.sub_message
        );

        let mut cursor = RenderCursor {
            x: area.left(),
            y: area.bottom().saturating_sub(1),
        };

        // Render the subscription details in gray
        let sub_details_symbols: Vec<Symbol> = get_message_symbols(&message, &mut [], Some((128, 128, 128)));

        // Shrink horizontal area by 4 to make space for border and scroll bar
        let mut line_area = area;
        line_area.width = area.width - 4;

        let mut lines: Vec<Vec<MessageParts>> = get_lines(&sub_details_symbols, &line_area);

        // Get symbols for subscription message
        let message_symbols: Vec<Symbol> =
            get_message_symbols(&self.message.sub_message, &mut [], Some((255, 255, 255)));
        // Get lines for subscription message
        let mut message_lines: Vec<Vec<MessageParts>> = get_lines(&message_symbols, &line_area);

        // Add subscription message lines to the subscription details
        lines.append(&mut message_lines);

        let mut screen_lines = get_screen_lines(&mut lines, &line_area);

        // Move cursor one over to make space for border
        cursor.x = area.left() + 1;
        cursor.y = cursor.y.saturating_sub(lines.len() as u16);

        write_to_buffer(&mut screen_lines, buf, &mut cursor);

        cursor.x = 0;
        cursor.y -= screen_lines.len() as u16;

        let block_area = Rect {
            x: 0,
            y: cursor.y - 1,
            width: area.width - 2,
            height: screen_lines.len() as u16 + 2,
        };

        let (sub_icon, sub_desc) = if self.message.context == "subgift" {
            ('🎁', "was gifted a sub!")
        } else if self.message.context == "resub" {
            ('📅', "has resubbed!")
        } else {
            ('🎉', "has subbed!")
        };

        Block::bordered()
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::reset().fg(Color::LightBlue))
            .title(format!("{}{} {}", sub_icon, self.message.display_name, sub_desc))
            .render(block_area, buf);

        self.area = Some(Rect {
            x: 0,
            y: cursor.y,
            width: area.width,
            height: screen_lines.len() as u16,
        });
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SubscribeMessage {
    pub display_name: String,   // some_person
    pub cumulative_months: u64, // 9
    pub streak_months: u64,     // 3
    pub context: String,        // subgift, resub
    pub sub_message: String,    // A message, possibly with emotes
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ChannelPointsData {
    pub timestamp: String,
    pub redemption: Redemption,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct UserReference {
    pub id: String,
    pub login: String,
    pub display_name: String,
    pub profile_url: Option<String>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Redemption {
    pub id: String,
    pub user: UserReference,
    pub user_input: Option<String>,
    pub status: String,
    pub reward: Reward,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Reward {
    pub id: String,
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

fn add_user_profile_url(message_data: &mut MessageData, credentials: &Credentials) -> Result<(), Box<dyn Error>> {
    match message_data.data {
        SubMessage::Points(ref mut sub_message) => {
            let api_url = "https://api.twitch.tv/helix/users";
            let id = &sub_message.redemption.user.id;
            let response = ureq::get(api_url)
                .set(
                    "Authorization",
                    &format!("Bearer {}", credentials.oauth_token.replace("oauth:", "")),
                )
                .set("Client-Id", credentials.client_id.as_str())
                .query_pairs(vec![("id", id.as_str())])
                .call();

            if response.is_err() {
                send_to_error_log("Error getting user profile pic".to_string(), format!("{response:?}"));
                return Ok(());
            }

            let response = response.unwrap();
            send_to_error_log("user profile pic response:".to_string(), format!("{response:?}"));

            let mut response: TwitchApiResponse<Vec<User>> = serde_json::from_reader(response.into_reader())?;
            let user = response.data.swap_remove(0);

            sub_message.redemption.user.profile_url = Some(user.profile_image_url);

            Ok(())
        }
        SubMessage::Sub(_) => Ok(()),
        SubMessage::Bits(_) => Ok(()),
    }
}

fn handle_message(
    message: Message,
    user: &User,
    tx: &Sender<ChannelMessages>,
    credentials: &Credentials,
) -> Result<(), Box<dyn Error>> {
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
            let Ok(mut sub_message) = serde_json::from_str::<MessageData>(sub_message) else {
                send_to_error_log(sub_message.to_string(), message.to_string());
                return Err("Not a message".into());
            };

            let _ = add_user_profile_url(&mut sub_message, credentials);

            // NOTE: Send message transmission before checking for CLI commands attached
            // to a message so that the CLI command does not block the chat log update
            tx.send(ChannelMessages::MessageData(sub_message.clone()))?;

            'commands: {
                match sub_message.data {
                    SubMessage::Points(ref sub_message) => {
                        let reward = get_reward(&sub_message.redemption.reward.title);
                        let irc_action = get_action(&sub_message.redemption.reward.title);

                        let found_command = if reward.is_ok() {
                            reward
                        } else if irc_action.is_ok() {
                            irc_action
                        } else {
                            break 'commands;
                        };

                        let Ok(command_name) = found_command else {
                            break 'commands;
                        };

                        let default_input = &String::new();
                        let user_input = &sub_message.redemption.user_input.as_ref().unwrap_or(default_input);

                        let command_result = Command::new(&command_name)
                            .arg(user_input)
                            .arg(&sub_message.redemption.user.display_name)
                            .stdout(process::Stdio::piped())
                            .stderr(process::Stdio::piped())
                            .output()
                            .expect("reward failed");

                        let command_success = command_result.status.success();

                        if !command_success {
                            send_to_error_log(
                                command_name.to_string(),
                                format!("Error running reward command with input: {}", user_input),
                            );

                            send_to_error_log(
                                format!("{} output: {:?}", command_name, command_result),
                                format!("Error running reward command with input: {}", user_input),
                            );

                            if sub_message.redemption.status == "UNFULFILLED" {
                                refund_points(sub_message, user, tx, credentials, command_result);
                            }
                        } else if sub_message.redemption.status == "UNFULFILLED" {
                            reward_fulfilled(sub_message, user, credentials);
                        }
                    }

                    SubMessage::Sub(ref _sub_message) => {}
                    SubMessage::Bits(ref _sub_message) => {}
                }
            }

            Ok(())
        }
        other => {
            send_to_error_log(other.to_string(), "Unknown Error".into());
            Err("Not a message".into())
        }
    }
}

fn reward_fulfilled(channel_points_data: &ChannelPointsData, user: &User, credentials: &Credentials) {
    let api_url = "https://api.twitch.tv/helix/channel_points/custom_rewards/redemptions";
    let id = &channel_points_data.redemption.id;
    let reward_id = &channel_points_data.redemption.reward.id;
    let response = ureq::patch(api_url)
        .set(
            "Authorization",
            &format!("Bearer {}", credentials.oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", credentials.client_id.as_str())
        .query_pairs(vec![
            ("id", id.as_str()),
            ("broadcaster_id", &user.id),
            ("reward_id", reward_id),
            ("status", "FULFILLED"),
        ])
        .call();

    if response.is_err() {
        send_to_error_log("Fulfill Error".to_string(), format!("{response:?}"));
    }
}

fn refund_points(
    channel_points_data: &ChannelPointsData,
    user: &User,
    tx: &Sender<ChannelMessages>,
    credentials: &Credentials,
    command_result: process::Output,
) {
    let api_url = "https://api.twitch.tv/helix/channel_points/custom_rewards/redemptions";

    let id = &channel_points_data.redemption.id;
    let reward_id = &channel_points_data.redemption.reward.id;
    let response = ureq::patch(api_url)
        .set(
            "Authorization",
            &format!("Bearer {}", credentials.oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", credentials.client_id.as_str())
        .query_pairs(vec![
            ("id", id.as_str()),
            ("broadcaster_id", &user.id),
            ("reward_id", reward_id),
            ("status", "CANCELED"),
        ])
        .call();

    let success = response.is_ok();
    if !success {
        send_to_error_log("Refund Error".to_string(), format!("{response:?}"));
    }

    let points = channel_points_data.redemption.reward.cost;
    let result = if success { "were" } else { "could not be" };
    let redeemer = &channel_points_data.redemption.user.display_name;
    let message = format!("{points} points {result} refunded to {redeemer}");
    let area = None;

    let _ = tx.send(ChannelMessages::TwitchMessage(TwitchMessage::RedeemMessage {
        message: RedeemMessage { message, area },
    }));

    let _ = tx.send(ChannelMessages::TwitchMessage(TwitchMessage::RedeemMessage {
        message: RedeemMessage {
            message: String::from_utf8(command_result.stdout)
                .expect("Invalid UTF-8")
                .to_string(),
            area,
        },
    }));
}

pub struct Credentials {
    pub oauth_token: Arc<String>,
    pub client_id: Arc<String>,
}

pub fn get_user(oauth_token: &str, client_id: &str) -> Result<User, Box<dyn Error>> {
    let get_users_url = "https://api.twitch.tv/helix/users";
    let response = ureq::get(get_users_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", client_id)
        .call();

    let Ok(response) = response else {
        return Err("Failed to get user data".into());
    };

    let mut response: TwitchApiResponse<Vec<User>> = serde_json::from_reader(response.into_reader())?;

    let user = response.data.swap_remove(0);

    Ok(user)
}

pub fn connect_to_pub_sub(
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    tx: Sender<ChannelMessages>,
) -> Result<(), Box<dyn Error>> {
    let user = get_user(&oauth_token, &client_id)?;
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
                            let credentials = Credentials {
                                oauth_token: oauth_token.clone(),
                                client_id: client_id.clone(),
                            };

                            let _ = handle_message(Message::Text(message), &user, &tx, &credentials);
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
