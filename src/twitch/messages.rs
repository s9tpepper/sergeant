// #[cfg(test)]
// mod tests {
//     use irc::proto::message::Tag;
//     use irc::proto::Message;
//
//     use crate::twitch::fixtures::TEST_MESSAGE_WITH_EMOTES;
//
//     use super::parse;
//
//     use std::error::Error;
//
//     #[tokio::test]
//     async fn test_parse_raid_message() -> Result<(), Box<dyn Error>> {
//         let tag = Tag(
//             "emotes".to_string(),
//             Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
//         );
//         let tag2 = Tag("msg-id".to_string(), Some("raid".to_string()));
//         let tag3 = Tag(
//             "system-msg".to_string(),
//             Some("system-msg=1\\sraiders\\sfrom\\svei_bean\\shave\\sjoined!".to_string()),
//         );
//
//         let tags = vec![tag, tag2, tag3];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//             "USERNOTICE",
//             vec!["#s9tpepper_"],
//         )
//         .unwrap();
//
//         println!("{:?}", msg.prefix);
//         println!("{:?}", msg.command);
//
//         let twitch_message = parse(msg).await?;
//
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::RaidMessage { raid_notice } => {
//                 assert_eq!("system-msg=1\\sraiders\\sfrom\\svei_bean\\shave\\sjoined!", raid_notice);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_emotes_attaching() -> Result<(), Box<dyn Error>> {
//         let tag = Tag(
//             "emotes".to_string(),
//             Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
//         );
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(TEST_MESSAGE_WITH_EMOTES, message.message);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_emotes_length() -> Result<(), Box<dyn Error>> {
//         let tag = Tag(
//             "emotes".to_string(),
//             Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
//         );
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(2, message.emotes.len());
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_emotes_url() -> Result<(), Box<dyn Error>> {
//         let tag = Tag(
//             "emotes".to_string(),
//             Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
//         );
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(
//                     "https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0",
//                     message.emotes[0].url
//                 );
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     // #[tokio::test]
//     // async fn test_parse_emotes_id() -> Result<(), Box<dyn Error>> {
//     //     let tag = Tag(
//     //         "emotes".to_string(),
//     //         Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
//     //     );
//     //     let tags = vec![tag];
//     //     let msg = Message::with_tags(
//     //         Some(tags),
//     //         Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//     //         "PRIVMSG",
//     //         vec!["#s9tpepper_", "This is a message from twitch"],
//     //     );
//     //
//     //     let twitch_message = parse(msg.unwrap()).await?;
//     //
//     //     assert_eq!("303147449", twitch_message.emotes[0].id);
//     //
//     //     Ok(())
//     // }
//
//     #[tokio::test]
//     async fn test_parse_emotes_position() -> Result<(), Box<dyn Error>> {
//         let tag = Tag(
//             "emotes".to_string(),
//             Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()),
//         );
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(0, message.emotes[0].start);
//                 assert_eq!(13, message.emotes[0].end);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_message() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!("This is a message from twitch", message.message);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_nickname() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!("rayslash", message.nickname);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_display_name() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("display-name".to_string(), Some("s9tpepper_".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(" s9tpepper_", message.display_name);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_color() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("color".to_string(), Some("#8A2BE2".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!("#8A2BE2", message.color);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_returning_chatter_is_true() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("returning-chatter".to_string(), Some("1".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(true, message.returning_chatter);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_returning_chatter_is_false() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("returning-chatter".to_string(), Some("0".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(false, message.returning_chatter);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_subscriber_is_true() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("subscriber".to_string(), Some("1".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(true, message.subscriber);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_subscriber_is_false() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("subscriber".to_string(), Some("0".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(false, message.subscriber);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_moderator_is_true() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("mod".to_string(), Some("1".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(true, message.moderator);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_moderator_is_false() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("mod".to_string(), Some("0".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(false, message.moderator);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_first_msg_is_true() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("first-msg".to_string(), Some("1".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(true, message.first_msg);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
//
//     #[tokio::test]
//     async fn test_parse_first_msg_is_false() -> Result<(), Box<dyn Error>> {
//         let tag = Tag("first-msg".to_string(), Some("0".to_string()));
//         let tags = vec![tag];
//         let msg = Message::with_tags(
//             Some(tags),
//             Some(""),
//             "PRIVMSG",
//             vec!["#s9tpepper_", "This is a message from twitch"],
//         );
//
//         let twitch_message = parse(msg.unwrap()).await?;
//         match twitch_message {
//             crate::twitch::messages::TwitchMessage::PrivMessage { message } => {
//                 assert_eq!(false, message.first_msg);
//             }
//             _ => {}
//         }
//
//         Ok(())
//     }
// }
