use std::sync::mpsc::Sender;

use crate::twitch::{
    self,
    parse::{ClearMessageByUser, RedeemMessage, TwitchMessage},
    ChannelMessages,
};

pub fn channel_ad_break_begin_notification(duration_seconds: Option<u64>, tx: Sender<ChannelMessages>) {
    if let Some(duration_seconds) = duration_seconds {
        let ad_message = format!("Starting {duration_seconds} second ad break...");
        let rm = RedeemMessage {
            message: ad_message,
            area: None,
            color: Some((0, 255, 255)), // cyan
        };

        let redeem_message = TwitchMessage::RedeemMessage { message: rm };
        let _ = tx.send(twitch::ChannelMessages::TwitchMessage(redeem_message));
    }
}

pub fn chat_clear_user_messages_notification(display_name: Option<String>, tx: Sender<ChannelMessages>) {
    if let Some(display_name) = display_name {
        let message = ClearMessageByUser { display_name };
        let twitch_message = TwitchMessage::ClearMessageByUser { message };
        let _ = tx.send(twitch::ChannelMessages::TwitchMessage(twitch_message));
    }
}
