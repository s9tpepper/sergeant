pub mod notifications;
pub mod structs;
pub mod subscriptions;

use std::{
    net::TcpStream,
    sync::{mpsc::Sender, Arc},
};

use serde::Deserialize;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

use crate::twitch::ChannelMessages;

use self::{
    notifications::{
        channel_ad_break_begin_notification, channel_chat_notification, chat_clear_user_messages_notification,
    },
    structs::{Subscription, SubscriptionEvent},
    subscriptions::{
        channel_ad_break_begin, channel_chat_clear_user_messages, CHANNEL_AD_BREAK_BEGIN, CHANNEL_CHAT_NOTIFICATION,
        CHAT_CLEAR_USER_MESSAGES,
    },
};

const EVENT_SUB: &str = "wss://eventsub.wss.twitch.tv:443/ws?keepalive_timeout_seconds=30";

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Session {
    id: String,
    status: String,
    connected_at: String,
    keepalive_timeout_seconds: u8,
    reconnect_url: Option<String>,
    recovery_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Metadata {
    message_id: String,
    message_type: String,
    message_timestamp: String,
}

#[derive(Debug, Deserialize)]
struct Payload {
    session: Option<Session>,
    subscription: Option<Subscription>,
    event: Option<SubscriptionEvent>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    metadata: Metadata,
    payload: Payload,
}

pub fn start_eventsub(
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    tx: Sender<ChannelMessages>,
    socket_tx: Sender<ChannelMessages>,
) {
    match tungstenite::connect(EVENT_SUB) {
        Ok((ref mut socket, _)) => {
            listen(socket, oauth_token, client_id, tx, socket_tx);
        }
        Err(_) => todo!(),
    }
}

fn listen(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    tx: Sender<ChannelMessages>,
    socket_tx: Sender<ChannelMessages>,
) {
    loop {
        if let Ok(message) = socket.read() {
            match message {
                tungstenite::Message::Text(text_message) => {
                    let deserialize_result = serde_json::from_str::<Message>(&text_message);
                    if deserialize_result.is_err() {
                        continue;
                    }

                    let msg = deserialize_result.unwrap();

                    match msg.metadata.message_type.as_str() {
                        "session_welcome" => {
                            create_subscriptions(msg, oauth_token.clone(), client_id.clone());
                        }

                        "notification" => {
                            if let Some(Subscription { r#type, .. }) = msg.payload.subscription {
                                match r#type.as_str() {
                                    CHANNEL_AD_BREAK_BEGIN => {
                                        if let Some(SubscriptionEvent { duration_seconds, .. }) = msg.payload.event {
                                            channel_ad_break_begin_notification(duration_seconds, tx.clone());
                                        }
                                    }

                                    CHAT_CLEAR_USER_MESSAGES => {
                                        if let Some(SubscriptionEvent { target_user_login, .. }) = msg.payload.event {
                                            chat_clear_user_messages_notification(target_user_login, tx.clone());
                                        }
                                    }

                                    CHANNEL_CHAT_NOTIFICATION => {
                                        if let Some(SubscriptionEvent { .. }) = msg.payload.event {
                                            channel_chat_notification(msg.payload.event, tx.clone(), socket_tx.clone());
                                        }
                                    }

                                    &_ => {}
                                }
                            }
                        }

                        &_ => {}
                    }
                }

                tungstenite::Message::Ping(ping_message) => {
                    let _ = socket.send(tungstenite::Message::Pong(ping_message));
                }

                tungstenite::Message::Close(close_message) => {
                    println!("Close message received: {close_message:?}");
                }

                _ => {}
            }
        }
    }
}

fn create_subscriptions(message: Message, oauth_token: Arc<String>, client_id: Arc<String>) {
    channel_ad_break_begin(&message, oauth_token.clone(), client_id.clone());
    channel_chat_clear_user_messages(&message, oauth_token.clone(), client_id.clone());
}
