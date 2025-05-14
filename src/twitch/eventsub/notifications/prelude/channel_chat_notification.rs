use std::sync::mpsc::Sender;

use crate::{
    channel::ChannelMessages,
    twitch::eventsub::deserialization::{NotificationEvent, NotificationPayload},
};

use super::send_to_channels;

pub fn channel_chat_notification(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
) -> anyhow::Result<()> {
    let notification_event @ NotificationEvent::ChannelChatMessage { .. } = &*payload.event else {
        return Ok(());
    };

    let message = Box::new(notification_event.clone());
    let channel_message = ChannelMessages::ChatMessage { message };

    send_to_channels(channel_message, tui_tx, websocket_tx, "channel_chat_notification")
}
