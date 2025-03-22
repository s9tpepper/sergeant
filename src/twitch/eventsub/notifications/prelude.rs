mod channel_ad_break_begin;
mod channel_chat_clear_user_messages;
mod channel_chat_message;
mod channel_chat_notification;
mod channel_points_automatic_reward_redemption;
mod channel_points_custom_reward_redemption_add;

use std::sync::mpsc::Sender;

use log::error;

use crate::channel::ChannelMessages;

pub use super::prelude::channel_ad_break_begin::channel_ad_break_begin;
pub use super::prelude::channel_chat_clear_user_messages::channel_chat_clear_user_messages;
pub use super::prelude::channel_chat_message::channel_chat_message;
pub use super::prelude::channel_chat_notification::channel_chat_notification;
pub use super::prelude::channel_points_automatic_reward_redemption::channel_points_automatic_reward_redemption;
pub use super::prelude::channel_points_custom_reward_redemption_add::channel_points_custom_reward_redemption_add;

pub fn send_to_channels(
    channel_message: ChannelMessages,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
    context: &str,
) {
    match tui_tx.send(channel_message.clone()) {
        Ok(_) => {}
        Err(error) => error!("[{context}]: Error sending to TUI: {error}"),
    }

    match websocket_tx.send(channel_message) {
        Ok(_) => {}
        Err(error) => error!("[{context}]: Error sending to websocket: {error}"),
    }
}
