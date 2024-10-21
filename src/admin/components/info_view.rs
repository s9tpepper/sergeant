use std::error::Error;

use anathema::{
    component::Component,
    state::{State, Value},
};

use crate::{
    commands::{get_list, get_list_commands},
    twitch::{announcements::get_announcements, pubsub::send_to_error_log},
    utils::read_auth_token,
};

use super::ComponentMessage;

#[derive(Default)]
pub struct InfoView;

#[derive(Default, State)]
pub struct InfoViewState {
    username: Value<String>,
    socket_server_port: Value<u16>,
    commands_count: Value<String>,
    announcements_count: Value<String>,
    rewards_count: Value<String>,
    irc_actions_count: Value<String>,
}

impl InfoViewState {
    pub fn new() -> Self {
        InfoViewState {
            username: String::from("[Anonymous]").into(),
            socket_server_port: 8765.into(),
            commands_count: String::from("0").into(),
            announcements_count: String::from("0").into(),
            rewards_count: String::from("0").into(),
            irc_actions_count: String::from("0").into(),
        }
    }
}

fn get_field_value<T, F>(list: Result<T, Box<dyn Error>>, field: &str, getter: F) -> String
where
    F: FnOnce(T) -> String,
{
    match list {
        Ok(list) => getter(list),
        Err(e) => {
            send_to_error_log(e.to_string(), format!("[ERROR] loading {field} in load_info()"));

            "!".to_string()
        }
    }
}

fn get_count<T>(list: Result<Vec<T>, Box<dyn Error>>, field: &str) -> String {
    get_field_value(list, field, |list| list.len().to_string())
}

impl InfoView {
    fn load_info(&self, state: &mut InfoViewState) {
        state
            .announcements_count
            .set(get_count(get_announcements(), "announcements"));

        state.commands_count.set(get_count(get_list_commands(), "commands"));
        state.rewards_count.set(get_count(get_list("chat_rewards"), "rewards"));
        state
            .irc_actions_count
            .set(get_count(get_list("irc_actions"), "irc actions"));

        state
            .username
            .set(get_field_value(read_auth_token(), "username", |token_status| {
                token_status.username.unwrap_or("Not logged in".to_string())
            }))

        // TODO: Update socket server port after its been changed to be configurable
    }
}

impl Component for InfoView {
    type State = InfoViewState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.load_info(state);
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        let component_message = serde_json::from_str::<ComponentMessage>(&message);

        match component_message {
            Ok(msg) => match msg.r#type {
                "load_data" => self.load_info(state),
                "todo" => {}

                _ => {}
            },
            Err(error) => send_to_error_log(error.to_string(), "Could not deserialize message to info_view".into()),
        }
    }
}
