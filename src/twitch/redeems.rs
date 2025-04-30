use std::{
    process,
    sync::{mpsc::Sender, Arc},
};

use log::{error, info};

use crate::channel::ChannelMessages;

use super::eventsub::{deserialization::Reward, notifications::prelude::send_to_channels};

#[allow(clippy::too_many_arguments)]
pub fn refund_points(
    id: &str,
    user_name: &str,
    broadcaster_user_id: &str,
    reward: &Reward,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
    oauth_token: &Arc<String>,
    client_id: &Arc<String>,
    command_result: process::Output,
) {
    let api_url = "https://api.twitch.tv/helix/channel_points/custom_rewards/redemptions";

    let response = ureq::patch(api_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", client_id)
        .query_pairs(vec![
            ("id", id),
            ("broadcaster_id", broadcaster_user_id),
            ("reward_id", &reward.id),
            ("status", "CANCELED"),
        ])
        .call();

    let success = response.is_ok();
    if !success {
        error!("Refund Error: {response:?}");
    }

    let points = reward.cost;
    let result = if success { "were" } else { "could not be" };
    let message = format!("{points} points {result} refunded to {user_name}");
    let command_output = String::from_utf8(command_result.stdout)
        .expect("Invalid UTF-8")
        .to_string();

    let channel_message = ChannelMessages::RedeemRefund {
        message,
        command_output,
    };

    send_to_channels(channel_message, tui_tx, websocket_tx, "refund_points");
}

#[allow(clippy::too_many_arguments)]
pub fn reward_fulfilled(
    id: &str,
    reward: &Reward,
    user_name: &str,
    user_input: &str,
    broadcaster_user_id: &str,
    oauth_token: &Arc<String>,
    client_id: &Arc<String>,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
) {
    info!("reward_fulfilled()");

    let api_url = "https://api.twitch.tv/helix/channel_points/custom_rewards/redemptions";
    let reward_id = &reward.id;
    let response = ureq::patch(api_url)
        .set(
            "Authorization",
            &format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .set("Client-Id", client_id)
        .query_pairs(vec![
            ("id", id),
            ("broadcaster_id", broadcaster_user_id),
            ("reward_id", reward_id),
            ("status", "FULFILLED"),
        ])
        .call();

    if response.is_err() {
        error!("Fulfill Error {response:?}");
        // TODO: Send message to tui/ws channels
    } else {
        info!("[Reward] {} fulfilled", reward.title);

        let message = get_reward_fulfilled_message(reward, user_name, user_input);
        let channel_message = ChannelMessages::RedeemMessage { message };

        send_to_channels(channel_message, tui_tx, websocket_tx, "refund_points");
    }
}

fn get_reward_fulfilled_message(reward: &Reward, user_name: &str, user_input: &str) -> String {
    if reward.prompt.is_empty() {
        format!("{} redeemed by {}", reward.title, user_name)
    } else {
        format!("{}({}) redeemed by {}", reward.title, user_input, user_name)
    }
}
