use std::sync::mpsc::Sender;

use log::{error, info};

use crate::{
    channel::ChannelMessages,
    chat_commands::check_for_commands,
    twitch::eventsub::{
        deserialization::{NotificationEvent, NotificationPayload},
        notifications::send_to_channels,
    },
};

pub fn channel_chat_message(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
) {
    info!("channel_chat_message()");
    info!("{payload:?}");

    let NotificationEvent::ChannelChatMessage { .. } = &*payload.event else {
        error!("Error trying to destructure NotificationEvent::ChannelChatMessage");
        return;
    };

    let channel_message = ChannelMessages::ChatMessage {
        message: payload.event.clone(),
    };

    send_to_channels(channel_message, tui_tx, websocket_tx, "channel_chat_message");

    let _ = check_for_commands(&payload.event);
}
