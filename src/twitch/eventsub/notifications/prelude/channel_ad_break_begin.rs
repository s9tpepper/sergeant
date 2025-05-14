use std::sync::mpsc::Sender;

use crate::{
    channel::ChannelMessages,
    twitch::eventsub::deserialization::{NotificationEvent, NotificationPayload},
};

use super::send_to_channels;

pub fn channel_ad_break_begin(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
) -> anyhow::Result<()> {
    let NotificationEvent::ChannelAdBreak { duration_seconds, .. } = *payload.event else {
        return Ok(());
    };

    let message = format!("Starting {duration_seconds} second ad break...");
    let channel_message = ChannelMessages::AdBreak { message };

    send_to_channels(channel_message, tui_tx, websocket_tx, "channel_ad_break_begin")
}
