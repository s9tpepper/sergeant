use std::sync::mpsc::Sender;

use crate::{channel::ChannelMessages, twitch::eventsub::deserialization::NotificationPayload};

use super::send_to_channels;

pub fn channel_points_automatic_reward_redemption(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
) -> anyhow::Result<()> {
    let channel_message = ChannelMessages::AutomaticRewardRedeem {
        message: payload.event.clone(),
    };

    send_to_channels(channel_message, tui_tx, websocket_tx, "channel_chat_message")
}
