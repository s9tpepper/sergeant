use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::twitch::{api::get_user, pubsub::send_to_error_log};

use super::{Message, Session};

const SUBSCRIPTIONS: &str = "https://api.twitch.tv/helix/eventsub/subscriptions";

pub const CHAT_CLEAR_USER_MESSAGES: &str = "channel.chat.clear_user_messages";
pub const CHANNEL_AD_BREAK_BEGIN: &str = "channel.ad_break.begin";

#[derive(Debug, Deserialize, Serialize)]
struct Condition {
    broadcaster_user_id: Option<String>,
    moderator_user_id: Option<String>,
    user_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Transport {
    method: String,
    session_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subscription {
    pub r#type: String,
    version: String,
    condition: Condition,
    transport: Transport,
    status: Option<String>,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubscriptionEvent {
    pub duration_seconds: Option<u64>,
    started_at: String,
    is_automatic: bool,
    broadcaster_user_id: String,
    broadcaster_user_login: String,
    broadcaster_user_name: String,
    requester_user_id: Option<String>,
    requester_user_login: Option<String>,
    requester_user_name: Option<String>,
    target_user_id: Option<String>,
    pub target_user_login: Option<String>,
    target_user_name: Option<String>,
}

fn request_subscription(
    r#type: String,
    condition: Condition,
    message: &Message,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
) {
    if let Some(Session { id: session_id, .. }) = &message.payload.session {
        let subscription = Subscription {
            r#type,
            condition,
            version: "1".to_string(),
            transport: Transport {
                method: "websocket".to_string(),
                session_id: session_id.to_string(),
            },
            status: None,
            created_at: None,
        };

        let subscription_result = ureq::post(SUBSCRIPTIONS)
            .set(
                "Authorization",
                format!("Bearer {}", &oauth_token.replace("oauth:", "")).as_str(),
            )
            .set("Client-Id", &client_id)
            .set("Content-Type", "application/json")
            .send_json(subscription);

        dbg!(&subscription_result);

        if let Err(error) = subscription_result {
            send_to_error_log("Subscription Error:".to_string(), error.to_string());
        }
    }
}

pub fn channel_ad_break_begin(message: &Message, oauth_token: Arc<String>, client_id: Arc<String>) {
    if let Ok(user) = get_user(&oauth_token, &client_id) {
        let condition = Condition {
            broadcaster_user_id: Some(user.id),
            moderator_user_id: None,
            user_id: None,
        };

        request_subscription(
            String::from_str(CHANNEL_AD_BREAK_BEGIN).unwrap(),
            condition,
            message,
            oauth_token,
            client_id,
        );
    }
}

pub fn channel_chat_clear_user_messages(message: &Message, oauth_token: Arc<String>, client_id: Arc<String>) {
    if let Ok(user) = get_user(&oauth_token, &client_id) {
        let condition = Condition {
            broadcaster_user_id: Some(user.id.clone()),
            moderator_user_id: None,
            user_id: Some(user.id),
        };

        request_subscription(
            String::from_str(CHAT_CLEAR_USER_MESSAGES).unwrap(),
            condition,
            message,
            oauth_token,
            client_id,
        );
    }
}
