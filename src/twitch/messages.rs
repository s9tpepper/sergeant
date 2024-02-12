use hex_rgb::*;

pub struct Badges {
    broadcaster: bool,
    premium: bool,
}

pub struct Emote {
    id: String,
    start: usize,
    end: usize,
    url: String,
    name: String,
}

pub struct TwitchMessage {
    pub badges: Badges,
    pub emotes: Vec<Emote>,
    pub nickname: Option<String>,
    pub display_name: Option<String>,
    pub first_msg: Option<bool>,
    pub returning_chatter: Option<bool>,
    pub subscriber: Option<bool>,
    pub moderator: Option<bool>,
    pub message: Option<String>,
    pub color: Option<String>,
}

impl TwitchMessage {
    pub fn get_nickname_color(&self) -> (u8, u8, u8) {
        let hex_color = self.color.clone().unwrap_or("#FFFFFF".to_string());
        if hex_color == "" {
            return (167, 23, 124)
        }
        let color = convert_hexcode_to_rgb(hex_color).unwrap();

        (color.red, color.green, color.blue)
    }

    fn set_field_bool(&mut self, field: &str, tag_value: Option<String>) {
        if let Some(value) = tag_value {
            match field {
                "subscriber" => self.subscriber = Some(get_bool(&value)),
                "first_msg" => self.first_msg = Some(get_bool(&value)),
                "returning_chatter" => self.returning_chatter = Some(get_bool(&value)),
                "moderator" => self.moderator = Some(get_bool(&value)),
                _ => panic!() // ... at the disco!
            }
        }
    }

    fn set_badge_value(&mut self, badge: &str) {
        let mut badge_parts = badge.split("/");
        if let Some(key) = badge_parts.next() {
            let value = badge_parts.next().unwrap_or("0");
            match key {
                "broadcaster" => self.badges.broadcaster = get_bool(value),
                "premium" => self.badges.premium = get_bool(value),
                other => {
                    println!("{}", other);
                },
            }
        }
    }
}

fn get_bool(value: &str) -> bool {
    if value == "0" {
        false
    } else {
        true
    }
}

pub mod messages {
    use std::io::Cursor;
    use std::error::Error;

    use irc::client::prelude::Message;
    use irc::proto::Command;
    use base64::prelude::*;

    use super::{TwitchMessage, Badges, Emote};

    pub async fn parse(message: Message) -> Result<TwitchMessage, Box<dyn Error>> {
        let mut twitch_message = TwitchMessage {
            badges: Badges { broadcaster: false, premium: false },
            emotes: vec![],
            nickname: None,
            display_name: None,
            first_msg: None,
            returning_chatter: None,
            subscriber: None,
            message: None,
            moderator: None,
            color: None,
        };
 
        let nickname:String = message.source_nickname().unwrap_or("").to_owned();
        twitch_message.nickname = Some(nickname);

        if let Some(tags) = message.tags {
            for tag in tags {
                match tag.0.as_str() {
                   "badges" => set_badges(tag.1, &mut twitch_message),
                   "color" => set_color(tag.1, &mut twitch_message),
                   "display-name" => set_display_name(tag.1, &mut twitch_message),
                   "first-msg" => twitch_message.set_field_bool("first_msg", tag.1),
                   "subscriber" => twitch_message.set_field_bool("subscriber", tag.1),
                   "returning-chatter" => twitch_message.set_field_bool("returning_chatter", tag.1),
                   "mod" => twitch_message.set_field_bool("moderator", tag.1),
                   "emotes" => process_emotes(tag.1, &mut twitch_message),
                   _other => {},
                }
            }
        }

        if let Command::PRIVMSG(ref _message_sender, ref message) = message.command {
            twitch_message.message = Some(message.to_string());
            let _ = add_emotes(&mut twitch_message).await?;
        }

        Ok(twitch_message)
    } 

    async fn add_emotes(twitch_message: &mut TwitchMessage) -> Result<(), Box<dyn Error>> {
        for emote in twitch_message.emotes.iter_mut() {
            let range = std::ops::Range {start: emote.start, end: emote.end + 1};
            let temp_msg = twitch_message.message.clone().expect("no message found");
            let emote_name = temp_msg.get(range);
            emote.name = emote_name.unwrap_or("").to_string();
        }

        for emote in twitch_message.emotes.iter() {
            let file_bytes: Vec<u8> = reqwest::get(&emote.url).await?.bytes().await?.to_vec();
            let size = file_bytes.len();

            let img_data = image::load_from_memory(&file_bytes)?;

            let mut buffer:Vec<u8> = Vec::new();
            img_data.write_to(&mut Cursor::new(&mut buffer), image::ImageOutputFormat::Png)?;
            let base64_emote = BASE64_STANDARD.encode(&buffer);

            //ESC]1337;File=size=FILESIZEINBYTES;inline=1:base-64 encoded file contents^G
            // works in iTerm
            let encoded_image = format!("\x1b]1337;File=size={};inline=1;height=20px;preserveAspectRatio=1:{}\x07", size, base64_emote.as_str());

            // TODO: Figure out the right encoding to make emotes work in tmux
            // let encoded_image = format!(" \x1b]tmux;\x1b]\x1b]1337;File=size={};inline=1;preserveAspectRatio=1:{}\x07", size, base64_emote.as_str());

            let mut msg = twitch_message.message.clone().unwrap();
            msg = msg.replace(&emote.name, encoded_image.as_str());
            twitch_message.message = Some(msg);
        }

        Ok(())
    }

    // 303147449:0-13
    // id: text-position-for-emote
    // https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0
    fn process_emotes(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
        if let Some(value) = tag_value {
            let emotes:Vec<&str> = value.split('/').collect();
            if emotes.len() == 0 {
                return
            }

            for emote_data in emotes.into_iter() {
                let mut emote_parts = emote_data.split(':');
                let emote_id = emote_parts.next();
                let Some(emote_id) = emote_id else { continue; };

                let positions = emote_parts.next();
                let Some(emote_position_data) = positions else { continue; };
                let mut emote_position_data = emote_position_data.split("-");
                let start = emote_position_data.next().unwrap().to_string().parse::<usize>().unwrap();
                let end = emote_position_data.next().unwrap().to_string().parse::<usize>().unwrap();

                let url = format!("https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0", emote_id);
                let name = "".to_string();

                let emote = Emote {
                    id: emote_id.to_owned(),
                    start,
                    end,
                    url,
                    name,
                };

                twitch_message.emotes.push(emote);
            }
        }
    }

    fn set_display_name(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
        if let Some(value) = tag_value {
            twitch_message.display_name = Some(value);
        }
    }

    fn set_color(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
        if let Some(value) = tag_value {
            twitch_message.color = Some(value);
        }
    }

    fn set_badges(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
        if let Some(value) = tag_value {
            let badges:Vec<&str> = value.split(',').collect();
            for badge in badges.into_iter() {
                twitch_message.set_badge_value(badge);
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use irc::proto::Message;
    use irc::proto::message::Tag;
    use super::messages::parse;

    use std::error::Error;

    #[tokio::test]
    async fn test_parse_emotes_attaching() -> Result<(), Box<dyn Error>> {
        let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags), 
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG", 
            vec!["#s9tpepper_", "This is a message from twitch"]);

        let twitch_message = parse(msg.unwrap()).await?;
        

        let parsed_message = twitch_message.message.unwrap();
        println!("{}", parsed_message);

        assert_eq!("hi", parsed_message);

        Ok(())
    }

    // #[test]
    // fn test_parse_emotes_length() {
    //     let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(
    //         Some(tags), 
    //         Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
    //         "PRIVMSG", 
    //         vec!["#s9tpepper_", "This is a message from twitch"]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(2, twitch_message.emotes.len());
    // }
    //
    // #[test]
    // fn test_parse_emotes_url() {
    //     let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(
    //         Some(tags), 
    //         Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
    //         "PRIVMSG", 
    //         vec!["#s9tpepper_", "This is a message from twitch"]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!("https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0", twitch_message.emotes[0].url);
    // }
    //
    // #[test]
    // fn test_parse_emotes_id() {
    //     let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(
    //         Some(tags), 
    //         Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
    //         "PRIVMSG", 
    //         vec!["#s9tpepper_", "This is a message from twitch"]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!("303147449", twitch_message.emotes[0].id);
    // }
    //
    // #[test]
    // fn test_parse_emotes_position() {
    //     let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(
    //         Some(tags), 
    //         Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
    //         "PRIVMSG", 
    //         vec!["#s9tpepper_", "This is a message from twitch"]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(0, twitch_message.emotes[0].start);
    //     assert_eq!(13, twitch_message.emotes[0].end);
    // }
    //
    // #[test]
    // fn test_parse_message() {
    //     let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(
    //         Some(tags), 
    //         Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
    //         "PRIVMSG", 
    //         vec!["#s9tpepper_", "This is a message from twitch"]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!("This is a message from twitch", twitch_message.message.unwrap());
    // }
    //
    // #[test]
    // fn test_parse_nickname() {
    //     let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some("rayslash!rayslash@rayslash.tmi.twitch.tv"), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!("rayslash", twitch_message.nickname.unwrap());
    // }
    //
    // #[test]
    // fn test_parse_display_name() {
    //     let tag = Tag("display-name".to_string(), Some("s9tpepper_".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!("s9tpepper_", twitch_message.display_name.unwrap());
    // }
    //
    // #[test]
    // fn test_parse_badge_broadcaster_true() {
    //     let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(true, twitch_message.badges.broadcaster);
    // }
    //
    // #[test]
    // fn test_parse_badge_broadcaster_false() {
    //     let tag = Tag("badges".to_string(), Some("broadcaster/0,premium/1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(false, twitch_message.badges.broadcaster);
    // }
    //
    // #[test]
    // fn test_parse_badge_premium_true() {
    //     let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(true, twitch_message.badges.premium);
    // }
    //
    // #[test]
    // fn test_parse_badge_premium_false() {
    //     let tag = Tag("badges".to_string(), Some("broadcaster/0,premium/0".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(false, twitch_message.badges.premium);
    // }
    //
    // #[test]
    // fn test_parse_color() {
    //     let tag = Tag("color".to_string(), Some("#8A2BE2".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!("#8A2BE2", twitch_message.color.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_returning_chatter_is_true() {
    //     let tag = Tag("returning-chatter".to_string(), Some("1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(true, twitch_message.returning_chatter.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_returning_chatter_is_false() {
    //     let tag = Tag("returning-chatter".to_string(), Some("0".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(false, twitch_message.returning_chatter.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_subscriber_is_true() {
    //     let tag = Tag("subscriber".to_string(), Some("1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(true, twitch_message.subscriber.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_subscriber_is_false() {
    //     let tag = Tag("subscriber".to_string(), Some("0".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(false, twitch_message.subscriber.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_moderator_is_true() {
    //     let tag = Tag("mod".to_string(), Some("1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(true, twitch_message.moderator.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_moderator_is_false() {
    //     let tag = Tag("mod".to_string(), Some("0".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(false, twitch_message.moderator.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_first_msg_is_true() {
    //     let tag = Tag("first-msg".to_string(), Some("1".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(true, twitch_message.first_msg.unwrap())
    // }
    //
    // #[test]
    // fn test_parse_first_msg_is_false() {
    //     let tag = Tag("first-msg".to_string(), Some("0".to_string()));
    //     let tags = vec![tag];
    //     let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);
    //
    //     let twitch_message = parse(msg.unwrap());
    //
    //     assert_eq!(false, twitch_message.first_msg.unwrap())
    // }
}
