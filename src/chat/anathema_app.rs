use std::{sync::mpsc::Receiver, time::Duration};

use anathema::{
    component::{Component, Elements},
    prelude::Context,
    state::{CommonVal, List, State, Value},
};
use log::{error, info};

use crate::{
    channel::ChannelMessages,
    twitch::eventsub::deserialization::{Fragment, FragmentType, NotificationEvent},
};

pub struct AnathemaApp {
    #[allow(unused)]
    twitch_name: String,
    receiver: Receiver<ChannelMessages>,
}

impl Component for AnathemaApp {
    type State = AnathemaAppState;

    type Message = ();

    fn tick(
        &mut self,
        state: &mut Self::State,
        _children: Elements<'_, '_>,
        _context: Context<'_, Self::State>,
        _dt: Duration,
    ) {
        match self.receiver.try_recv() {
            Ok(channel_messages) => match channel_messages {
                message @ ChannelMessages::AdBreak { .. } => self.ad_break(message),
                message @ ChannelMessages::ChatMessage { .. } => self.chat_message(message, state),
                message @ ChannelMessages::BotAnnouncement { .. } => self.bot_announcement(message),
                message @ ChannelMessages::ClearMessagesByUser { .. } => self.clear_messages_by_user(message),
                message @ ChannelMessages::RedeemRefund { .. } => self.redeem_refund(message),
                message @ ChannelMessages::AutomaticRewardRedeem { .. } => self.automatic_reward_redeem(message),

                // TODO: Implement display for redeem messages
                _message @ ChannelMessages::RedeemMessage { .. } => todo!("Implement redeem messagers"),
            },

            Err(recv_error) => match recv_error {
                std::sync::mpsc::TryRecvError::Empty => {}
                std::sync::mpsc::TryRecvError::Disconnected => {
                    error!("Chat disconnected");
                    panic!("Chat disconnected");
                }
            },
        };
    }
}

impl AnathemaApp {
    #[allow(unused)]
    pub fn new(twitch_name: String, receiver: Receiver<ChannelMessages>) -> Self {
        AnathemaApp { twitch_name, receiver }
    }

    pub fn ad_break(&self, _message: ChannelMessages) {}

    pub fn chat_message(&self, message: ChannelMessages, state: &mut AnathemaAppState) {
        info!("Processing chat_message()");

        let ChannelMessages::ChatMessage { message } = message else {
            error!("Error destructuring ChannelMessages::ChatMessage, returning...");

            return;
        };

        info!("Got ChannelMessages::ChatMessage::message");

        let notification_event = *message;

        #[allow(clippy::single_match)]
        match notification_event {
            NotificationEvent::ChannelChatMessage {
                // broadcaster_user_id,
                // broadcaster_user_name,
                // broadcaster_user_login,
                // chatter_user_id,
                chatter_user_name,
                // chatter_user_login,
                // message_id,
                message,
                color,
                ..
                // message_type,
                // badges,
                // cheer,
                // reply,
                // channel_points_custom_reward_id,
                // source_broadcaster_user_id,
                // source_broadcaster_user_name,
                // source_broadcaster_user_login,
                // source_message_id,
                // source_badges,
            } => {
                info!("Adding log entry...");

                let log_entry = LogEntry {
                    chatter_user_name: chatter_user_name.into(),
                    color: color.into(),
                    fragments: List::from_iter(message.fragments.iter().map(|fragment| (fragment.clone()).into()))
                };

                state.log.push(log_entry);

                // let log_entry = LogEntry::ChatMessage { message: message.text };
                // info!("{log_entry:?}");
                // state.log.push(log_entry);
                // info!("log: {:?}", state.log);
                // info!("Log entry added.");
            }

            _ => {}
        }
    }

    pub fn bot_announcement(&self, _message: ChannelMessages) {}
    pub fn clear_messages_by_user(&self, _message: ChannelMessages) {}
    pub fn redeem_refund(&self, _message: ChannelMessages) {}
    pub fn automatic_reward_redeem(&self, _message: ChannelMessages) {}
}

#[derive(State)]
pub struct AnathemaAppState {
    pub log: Value<List<LogEntry>>,
    pub test_field: Value<String>,
}

#[derive(State)]
pub struct LogEntry {
    pub chatter_user_name: Value<String>,
    pub color: Value<String>,
    pub fragments: Value<List<LogEntryFragment>>,
}

#[derive(State)]
pub struct LogEntryFragment {
    pub r#type: Value<LogEntryFragmentType>,
    pub text: Value<String>,
    pub cheermote: Value<Option<LogEntryCheermote>>,
    pub emote: Value<Option<LogEntryEmote>>,
    pub mention: Value<Option<LogEntryMention>>,
}

impl From<Fragment> for LogEntryFragment {
    fn from(value: Fragment) -> Self {
        // TODO: Fix cheermote, emote, mention
        LogEntryFragment {
            r#type: Value::new(value.r#type.into()),
            text: value.text.into(),
            cheermote: None.into(),
            emote: None.into(),
            mention: None.into(),
        }
    }
}

impl From<FragmentType> for LogEntryFragmentType {
    fn from(value: FragmentType) -> Self {
        match value {
            FragmentType::Text => LogEntryFragmentType::Text,
            FragmentType::Cheermote => LogEntryFragmentType::Cheermote,
            FragmentType::Emote => LogEntryFragmentType::Emote,
            FragmentType::Mention => LogEntryFragmentType::Mention,
            FragmentType::Unknown => LogEntryFragmentType::Unknown,
        }
    }
}

#[derive(State)]
pub struct LogEntryCheermote {
    prefix: Value<String>,
    bits: Value<u16>,
    tier: Value<u8>,
}

#[derive(State)]
pub struct LogEntryEmote {
    id: Value<String>,
    emote_set_id: Value<String>,
    owner_id: Value<String>,
    format: Value<List<String>>, // animated | static
}

#[derive(State)]
pub struct LogEntryMention {
    user_id: Value<String>,
    user_name: Value<String>,
    user_login: Value<String>,
}

pub enum LogEntryFragmentType {
    Text,
    Cheermote,
    Emote,
    Mention,
    Unknown,
}

impl State for LogEntryFragmentType {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        match self {
            LogEntryFragmentType::Text => Some(CommonVal::Str("Text")),
            LogEntryFragmentType::Cheermote => Some(CommonVal::Str("Cheermote")),
            LogEntryFragmentType::Emote => Some(CommonVal::Str("Emote")),
            LogEntryFragmentType::Mention => Some(CommonVal::Str("Mention")),
            LogEntryFragmentType::Unknown => Some(CommonVal::Str("Unknown")),
        }
    }
}

// #[derive(Debug)]
// pub enum LogEntry {
//     AdBreak,
//     ChatMessage { message: String },
//     BotAnnouncement,
// }

// impl State for LogEntry {
//     fn type_info(&self) -> Type {
//         match self {
//             LogEntry::AdBreak => Type::Map,
//             LogEntry::ChatMessage { .. } => Type::Map,
//             LogEntry::BotAnnouncement => Type::Map,
//         }
//     }
//
//     fn as_str(&self) -> Option<&str> {
//         match self {
//             LogEntry::AdBreak => todo!(),
//             LogEntry::ChatMessage { message } => Some(message),
//             LogEntry::BotAnnouncement => todo!(),
//         }
//     }
//
// fn as_any_map(&self) -> Option<&dyn AnyMap> {
//     match self {
//         LogEntry::AdBreak => todo!(),
//
//         LogEntry::ChatMessage { message } => {
//             let mut map = Map::<String>::empty();
//             map.insert("message", message.to_string());
//
//             let boxed_map = Box::new(map);
//             Some(Box::leak(boxed_map))
//         }
//
//         LogEntry::BotAnnouncement => todo!(),
//     }
// }
// }
