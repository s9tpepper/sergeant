use std::{
    net::TcpStream,
    sync::{mpsc::Sender, Arc},
};

use anyhow::bail;
use deserialization::{Condition, Messages, MethodType, Subscription, SubscriptionType, Transport, WelcomePayload};
use log::info;
use notifications::handle_notification;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

use crate::channel::ChannelMessages;

use super::api::{get_user, User};

const EVENT_SUB: &str = "wss://eventsub.wss.twitch.tv:443/ws?keepalive_timeout_seconds=30";
const SUBSCRIPTIONS: &str = "https://api.twitch.tv/helix/eventsub/subscriptions";

pub mod deserialization;
pub mod notifications;

pub fn start_eventsub(
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    tx: Sender<ChannelMessages>,
    socket_tx: Sender<ChannelMessages>,
) {
    match tungstenite::connect(EVENT_SUB) {
        Ok((ref mut socket, _)) => {
            info!("eventsub connected");

            listen(socket, oauth_token, client_id, tx, socket_tx);
        }
        Err(_) => todo!(),
    }
}

fn listen(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    tui_tx: Sender<ChannelMessages>,
    websocket_tx: Sender<ChannelMessages>,
) {
    loop {
        if let Ok(message) = socket.read() {
            match message {
                tungstenite::Message::Text(text_message) => {
                    info!("EventSub Message: {text_message} end of EventSub");

                    let deserialized_msg = serde_json::from_str::<Messages>(&text_message);
                    info!("deserialized_msg: {:?}", deserialized_msg);

                    match deserialized_msg {
                        Ok(message) => match &message {
                            Messages::Welcome { payload, .. } => {
                                info!("listen::Messages::Welcome()");
                                create_subscriptions(payload, &oauth_token, &client_id)
                            }

                            Messages::KeepAlive { .. } => {
                                info!("listen::Messages::KeepAlive()");
                            }

                            Messages::Notification { metadata, payload } => {
                                info!("listen::Messages::Notification()");
                                handle_notification(metadata, payload, &tui_tx, &websocket_tx, &oauth_token, &client_id)
                            }

                            Messages::Reconnect { .. } => {
                                info!("listen::Messages::Reconnect()");
                            }

                            Messages::Revocation { .. } => {
                                info!("listen::Messages::Revocation()");
                            }
                        },

                        Err(error) => {
                            info!("serde_json error: {error}");
                            continue;
                        }
                    }
                }

                tungstenite::Message::Ping(ping_message) => {
                    info!("PING");
                    let _ = socket.send(tungstenite::Message::Pong(ping_message));
                    info!("PONG");
                }

                tungstenite::Message::Close(close_message) => {
                    info!("Close message received: {close_message:?}");
                }

                catchall => {
                    info!("Catchall: {catchall}");
                }
            }
        }
    }
}

fn create_subscriptions(payload: &WelcomePayload, oauth_token: &Arc<String>, client_id: &Arc<String>) {
    let subscriptions = [
        SubscriptionType::ChannelAdBreakBegin,
        SubscriptionType::ChannelChatClearUserMessages,
        SubscriptionType::ChannelChatNotification,
        SubscriptionType::ChannelPointsCustomRewardRedemptionAdd,
        SubscriptionType::ChannelPointsCustomRewardRedemptionUpdate,
        SubscriptionType::ChannelPointsAutomaticRewardRedemption,
        SubscriptionType::ChannelChatMessage,
    ];

    subscriptions.iter().for_each(|subscription_type| {
        let sub_name = match serde_json::to_string(&subscription_type) {
            Ok(serialized) => serialized,
            Err(_) => "subscription_type(unserialized)".to_string(),
        };

        info!("Subscribing to {sub_name}");

        let error_message = format!("{sub_name} subscription failed");
        get_eventsub_subscription(
            &payload.session.id,
            subscription_type,
            MethodType::WebSocket,
            oauth_token,
            client_id,
        )
        .expect(&error_message);

        info!("Subscribed to {sub_name}.");
    });

    info!("Finished subscribing to all events");
}

fn get_eventsub_subscription(
    session_id: &str,
    r#type: &SubscriptionType,
    method: MethodType,
    oauth_token: &Arc<String>,
    client_id: &Arc<String>,
) -> anyhow::Result<()> {
    match get_user(&oauth_token.clone(), &client_id.clone()) {
        Ok(user) => {
            let sub_type = r#type.clone();
            let condition = get_condition(&user);
            let subscription = get_subscription(r#type, condition, session_id, method);

            match ureq::post(SUBSCRIPTIONS)
                .set(
                    "Authorization",
                    format!("Bearer {}", &oauth_token.replace("oauth:", "")).as_str(),
                )
                .set("Client-Id", client_id)
                .set("Content-Type", "application/json")
                .send_json(subscription)
            {
                Ok(response) => {
                    info!("Subscription Response: {}", response.into_string()?);

                    Ok(())
                }
                Err(_) => bail!("Could not complete subscription request for {sub_type:?}"),
            }
        }
        Err(user_error) => bail!("Error retrieving user: {user_error}"),
    }
}

fn get_condition(user: &User) -> Condition {
    Condition {
        to_broadcaster_user_id: None,
        broadcaster_user_id: Some(user.id.clone()),
        moderator_user_id: None,
        user_id: Some(user.id.clone()),
        reward_id: None,
        client_id: None,
        organization_id: None,
        category_id: None,
        campaign_id: None,
        extension_client_id: None,
    }
}

fn get_subscription<'a>(
    r#type: &'a SubscriptionType,
    condition: Condition,
    session_id: &str,
    method: MethodType,
) -> Subscription<'a> {
    Subscription {
        r#type,
        condition,
        version: get_subscription_version(r#type),
        transport: Transport {
            method,
            session_id: session_id.to_string(),
        },
        status: None,
        created_at: None,
    }
}

// TODO: Update this to something better than this match statement, maybe a part of the enum
fn get_subscription_version(r#type: &SubscriptionType) -> String {
    match r#type {
        SubscriptionType::ChannelPointsAutomaticRewardRedemption => "2",
        _ => "1",
    }
    .to_string()
}
