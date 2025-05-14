use std::sync::mpsc::Sender;

use log::{error, info};

use crate::{
    channel::ChannelMessages,
    chat_commands::{check_for_commands, check_for_message_actions},
    twitch::eventsub::{
        deserialization::{NotificationEvent, NotificationPayload},
        notifications::send_to_channels,
    },
};

pub fn channel_chat_message(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
) -> anyhow::Result<()> {
    info!("channel_chat_message()");
    info!("{payload:?}");

    let NotificationEvent::ChannelChatMessage { .. } = &*payload.event else {
        error!("Error trying to destructure NotificationEvent::ChannelChatMessage");
        return Ok(());
    };

    let channel_message = ChannelMessages::ChatMessage {
        message: payload.event.clone(),
    };

    send_to_channels(channel_message, tui_tx, websocket_tx, "channel_chat_message")?;

    check_for_commands(&payload.event)?;

    check_for_message_actions(&payload.event)
}
