use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{State, Value},
};

use crate::admin::{messages::ComponentMessages, templates::CONFIRM_TEMPLATE, AppComponent};

#[derive(Default)]
pub struct Confirm {}

impl AppComponent for Confirm {}
impl Confirm {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_command::AddCommand as AppComponent>::register_component(
            builder,
            "confirm_window",
            CONFIRM_TEMPLATE,
            Confirm::new(),
            ConfirmState::new(),
            component_ids,
        )
    }
}

impl Confirm {
    pub fn new() -> Self {
        Confirm {}
    }
}

#[derive(Default, State)]
pub struct ConfirmState {
    title: Value<String>,
    message: Value<String>,
    waiting: Value<String>,
    component_message: Value<String>,
}

impl ConfirmState {
    pub fn new() -> Self {
        ConfirmState {
            title: "".to_string().into(),
            message: "".to_string().into(),
            waiting: "".to_string().into(),
            component_message: "".to_string().into(),
        }
    }
}

impl Component for Confirm {
    type State = ConfirmState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'y' => {
                    match serde_json::from_str::<ComponentMessages>(&state.component_message.to_ref()) {
                        Ok(msg) => match msg {
                            ComponentMessages::DeleteCommandConfirmMessage(_) => {
                                context.publish("confirm_delete_command", |state| &state.component_message);
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
                'n' => context.publish("cancel_confirmation", |state| &state.waiting),
                _ => {}
            },
            anathema::component::KeyCode::Esc => context.publish("cancel_confirmation", |state| &state.waiting),

            _ => {}
        }
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        _: anathema::prelude::Context<'_, Self::State>,
    ) {
        state.component_message.set(message.to_string());

        match serde_json::from_str::<ComponentMessages>(&message.to_string()) {
            Ok(msg) => match &msg {
                ComponentMessages::DeleteCommandConfirmMessage(delete_msg) => {
                    state.title.set(delete_msg.payload.title.to_string());
                    state.message.set(delete_msg.payload.message.to_string());
                    state.waiting.set(delete_msg.payload.waiting.to_string());
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
