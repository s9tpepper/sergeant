use anathema::{
    component::Component,
    state::{State, Value},
};

use crate::admin::messages::ComponentMessages;

#[derive(Default)]
pub struct Confirm {
    message: String,
    delete_command_details: Option<String>,
}

impl Confirm {
    pub fn new() -> Self {
        Confirm {
            message: "".to_string(),
            delete_command_details: None,
        }
    }
}

#[derive(Default, State)]
pub struct ConfirmState {
    title: Value<String>,
    message: Value<String>,
    waiting: Value<String>,
}

impl ConfirmState {
    pub fn new() -> Self {
        ConfirmState {
            title: "".to_string().into(),
            message: "".to_string().into(),
            waiting: "".to_string().into(),
        }
    }
}

impl Component for Confirm {
    type State = ConfirmState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        self.message = message.to_string();

        match serde_json::from_str::<ComponentMessages>(&self.message) {
            Ok(msg) => match msg {
                ComponentMessages::DeleteCommandConfirmMessage(delete_msg) => {
                    state.title.set(delete_msg.payload.title.to_string());
                    state.message.set(delete_msg.payload.message.to_string());

                    if let Ok(payload) = serde_json::to_string(&delete_msg.payload) {
                        self.delete_command_details = Some(payload);
                    }
                }

                ComponentMessages::InfoViewLoad(_) => {}

                _ => {}
            },
            Err(err) => {
                // println!("Could not deserialize ComponentMessage: {}", self.message);
                println!("{}", err);
            }
        }
    }
}
