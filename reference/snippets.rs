
// @badge-info=;
// badges=broadcaster/1,premium/1;
// client-nonce=98d669c84201118e21161fa8c20c4ed1;
// color=#8A2BE2;display-name=s9tpepper_;emotes=;first-msg=0;flags=;id=20a7e212-b422-4fcd-8a14-701ec4c43bdf;mod=0;returning-chatter=0;room-id=961536166;subscriber=0;tmi-sent-ts=1707472368016;turbo=0;user-id=961536166;user-type= :s9tpepper_!s9tpepper_@s9tpepper_.tmi.twitch.tv PRIVMSG #s9tpepper_ :this is a message with lots of things to parse
pub fn parse1(message: &Message) -> TwitchMessage {
    let mut twitch_message = TwitchMessage {
        badges: None,
        nickname: None,
        display_name: None,
        first_msg: None,
        returning_chatter: None,
        subscriber: None,
        message: None,
        moderator: None,
    };

    let msg = message.to_string();
    for message_part in msg.split(';') {
        let mut key_value_pair = message_part.split('=');
        let key = key_value_pair.next().unwrap_or("no_key");
        let value = key_value_pair.next().unwrap_or("");
        match key {
            "badges" => set_badges(value.to_string(), &mut twitch_message),
            "display-name" => set_display_name(value.to_string(), &mut twitch_message),
            "subscriber" => twitch_message.set_field_bool("subscriber", value),
            "first-msg" => twitch_message.set_field_bool("first_msg", value), 
            "returning-chatter" => twitch_message.set_field_bool("returning_chatter", value),
            "mod" => twitch_message.set_field_bool("moderator", value),
            &_ => todo!(),
        }
    }

    twitch_message
}



fn sset_badges(value: String, twitch_message: &mut TwitchMessage) {
    let mut message_badges = Badges {
        broadcaster: false,
        premium: false,
    };

    let badges:Vec<&str> = value.split(',').collect();
    for badge in badges.into_iter() {
        match badge {
            "broadcaster" => message_badges.broadcaster = true,
            "premium" => message_badges.premium = true,
            &_ => todo!(),
        }
    }

    twitch_message.badges = Some(message_badges);
}
