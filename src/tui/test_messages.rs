use crate::{
    twitch::{
        parse::{ChatMessage, RaidMessage, TwitchMessage},
        ChannelMessages,
    },
    utils::unescape,
};

pub fn get_priv_message() -> TwitchMessage {
    TwitchMessage::PrivMessage {
        message: ChatMessage {
            id: "1234".to_string(),
            nickname: "some_person".to_string(),
            color: "#FF0000".to_string(),
            message: "This is a test message".to_string(),
            first_msg: false,
            badges: vec![],
            emotes: vec![],
            returning_chatter: false,
            subscriber: false,
            moderator: false,
            channel: "some_channel".to_string(),
            raw: "raw message".to_string(),
            area: None,
            animation_id: "".to_string(),
            can_animate: false,
            r: 0,
            g: 0,
            b: 0,
            direction: 0,
            timestamp: None,
        },
    }
}

pub fn get_raid_message() -> ChannelMessages {
    let message = RaidMessage {
        // display_name: "some_person".to_string(),
        // user_id: "1234".to_string(),
        // raid_notice: "1 raiders from some_person have joined!".to_string(),
        display_name: "MatisseTec".to_string(),
        user_id: "468106723".to_string(),
        raid_notice: unescape(r"37\\sraiders\\sfrom\\sMatisseTec\\shave\\sjoined!"),
        area: None,
        r: 128,
        g: 1,
        b: 249,
        direction: 1,
    };

    ChannelMessages::TwitchMessage(TwitchMessage::RaidMessage { message })
}

// NOTE: Push messages here to test with
//
//
// self.app.chat_log.push(ChannelMessages::TwitchMessage(chat_message));
//
//
// self.chat_log
//     .push(ChannelMessages::TwitchMessage(TwitchMessage::RaidMessage { message }));
//
// let chat_message = TwitchMessage::PrivMessage {
//     message: ChatMessage {
//         id: "1234".to_string(),
//         nickname: "some_person".to_string(),
//         color: "#FF0000".to_string(),
//         message: "This is a test message".to_string(),
//         first_msg: false,
//         badges: vec![],
//         emotes: vec![],
//         returning_chatter: false,
//         subscriber: false,
//         moderator: false,
//         channel: "some_channel".to_string(),
//         raw: "raw message".to_string(),
//         area: None,
//         animation_id: "simmer".to_string(),
//         animation_id: "rainbow-eclipse".to_string(),
//         can_animate: true,
//         r: 128,
//         g: 1,
//         b: 249,
//         direction: 1,
//         timestamp: None,
//     },
// };
// self.app.chat_log.push(ChannelMessages::TwitchMessage(chat_message));
//
// let data = SubMessage::Sub(SubscribeEvent {
//     area: None,
//     topic: "topic string".to_string(),
//     message: SubscribeMessage {
//         display_name: "some_dude".to_string(),
//         cumulative_months: 8,
//         streak_months: 10,
//         context: "".to_string(), // subgift, resub
//         sub_message: "This is a subscription message".to_string(),
//     },
// });
//
// self.app
//     .chat_log
//     .push(ChannelMessages::MessageData(MessageData { data }))

// pub is_anonymous: bool,
// pub message_type: String,
// pub data: BitsEventData,
//
// BitsEventData
// pub user_name: String,
// pub chat_message: String,
// pub bits_used: u64,
// pub total_bits_used: u64,
// pub context: String, // cheer

// let data = SubMessage::Bits(BitsEvent {
//     area: None,
//     is_anonymous: false,
//     message_type: "bits_event".to_string(),
//     data: BitsEventData {
//         user_name: "some_dude".to_string(),
//         chat_message: "some cheery message".to_string(),
//         bits_used: 500,
//         total_bits_used: 1000,
//         context: "cheer".to_string(),
//     },
// });
//
// self.app
//     .chat_log
//     .push(ChannelMessages::MessageData(MessageData { data }))
