use hex_rgb::*;

pub struct Badges {
    broadcaster: bool,
    premium: bool,
}

pub struct Emote {
    id: String,
    start: u16,
    end: u16,
    url: String,
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
    } else if value == "1" {
        true
    } else {
        true
    }
}

pub mod messages {
    use irc::client::prelude::Message;
    use irc::proto::Command;

    use super::{TwitchMessage, Badges, Emote};

    pub fn parse(message: Message) -> TwitchMessage {
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
        }

        twitch_message
    } 

    // 303147449:0-13
    // id: text-position-for-emote
    //
    // https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0
    fn process_emotes(tag_value: Option<String>, twitch_message: &mut TwitchMessage) {
        if let Some(value) = tag_value {
            let emotes:Vec<&str> = value.split('/').collect();
            for emote_data in emotes.into_iter() {
                let mut emote_parts = emote_data.split(':');
                let emote_id = emote_parts.next();
                let mut emote_position_data = emote_parts.next().unwrap().split("-");
                let start = emote_position_data.next().unwrap().to_string().parse::<u16>().unwrap();
                let end = emote_position_data.next().unwrap().to_string().parse::<u16>().unwrap();
                let url = format!("https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0", emote_id.unwrap());

                let emote = Emote {
                    id: emote_id.unwrap().to_owned(),
                    start,
                    end,
                    url,
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

    #[test]
    fn test_parse_emotes_length() {
        let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags), 
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG", 
            vec!["#s9tpepper_", "This is a message from twitch"]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(2, twitch_message.emotes.len());
    }

    #[test]
    fn test_parse_emotes_url() {
        let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags), 
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG", 
            vec!["#s9tpepper_", "This is a message from twitch"]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!("https://static-cdn.jtvnw.net/emoticons/v2/303147449/default/dark/1.0", twitch_message.emotes[0].url);
    }

    #[test]
    fn test_parse_emotes_id() {
        let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags), 
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG", 
            vec!["#s9tpepper_", "This is a message from twitch"]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!("303147449", twitch_message.emotes[0].id);
    }

    #[test]
    fn test_parse_emotes_position() {
        let tag = Tag("emotes".to_string(), Some("303147449:0-13/emotesv2_a388006c5b8c4826906248a22b50d0a3:15-28".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags), 
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG", 
            vec!["#s9tpepper_", "This is a message from twitch"]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(0, twitch_message.emotes[0].start);
        assert_eq!(13, twitch_message.emotes[0].end);
    }

    #[test]
    fn test_parse_message() {
        let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(
            Some(tags), 
            Some("rayslash!rayslash@rayslash.tmi.twitch.tv"),
            "PRIVMSG", 
            vec!["#s9tpepper_", "This is a message from twitch"]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!("This is a message from twitch", twitch_message.message.unwrap());
    }

    #[test]
    fn test_parse_nickname() {
        let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some("rayslash!rayslash@rayslash.tmi.twitch.tv"), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!("rayslash", twitch_message.nickname.unwrap());
    }

    #[test]
    fn test_parse_display_name() {
        let tag = Tag("display-name".to_string(), Some("s9tpepper_".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!("s9tpepper_", twitch_message.display_name.unwrap());
    }

    #[test]
    fn test_parse_badge_broadcaster_true() {
        let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(true, twitch_message.badges.broadcaster);
    }

    #[test]
    fn test_parse_badge_broadcaster_false() {
        let tag = Tag("badges".to_string(), Some("broadcaster/0,premium/1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(false, twitch_message.badges.broadcaster);
    }

    #[test]
    fn test_parse_badge_premium_true() {
        let tag = Tag("badges".to_string(), Some("broadcaster/1,premium/1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(true, twitch_message.badges.premium);
    }

    #[test]
    fn test_parse_badge_premium_false() {
        let tag = Tag("badges".to_string(), Some("broadcaster/0,premium/0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(false, twitch_message.badges.premium);
    }

    #[test]
    fn test_parse_color() {
        let tag = Tag("color".to_string(), Some("#8A2BE2".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!("#8A2BE2", twitch_message.color.unwrap())
    }

    #[test]
    fn test_parse_returning_chatter_is_true() {
        let tag = Tag("returning-chatter".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(true, twitch_message.returning_chatter.unwrap())
    }

    #[test]
    fn test_parse_returning_chatter_is_false() {
        let tag = Tag("returning-chatter".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(false, twitch_message.returning_chatter.unwrap())
    }

    #[test]
    fn test_parse_subscriber_is_true() {
        let tag = Tag("subscriber".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(true, twitch_message.subscriber.unwrap())
    }

    #[test]
    fn test_parse_subscriber_is_false() {
        let tag = Tag("subscriber".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(false, twitch_message.subscriber.unwrap())
    }

    #[test]
    fn test_parse_moderator_is_true() {
        let tag = Tag("mod".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(true, twitch_message.moderator.unwrap())
    }

    #[test]
    fn test_parse_moderator_is_false() {
        let tag = Tag("mod".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(false, twitch_message.moderator.unwrap())
    }

    #[test]
    fn test_parse_first_msg_is_true() {
        let tag = Tag("first-msg".to_string(), Some("1".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(true, twitch_message.first_msg.unwrap())
    }

    #[test]
    fn test_parse_first_msg_is_false() {
        let tag = Tag("first-msg".to_string(), Some("0".to_string()));
        let tags = vec![tag];
        let msg = Message::with_tags(Some(tags), Some(""), "PRIVMSG", vec![]);

        let twitch_message = parse(msg.unwrap());

        assert_eq!(false, twitch_message.first_msg.unwrap())
    }
}
