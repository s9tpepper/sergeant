use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId},
    prelude::{Context, ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, State, Value},
};

use crate::{
    admin::{
        components::{
            app::{AppMessageHandler, AppState},
            MessageSender, Messenger,
        },
        messages::ComponentMessages,
        templates::CONFIRM_TEMPLATE,
        AppComponent,
    },
    twitch::pubsub::send_to_error_log,
};

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
            CONFIRM_TEMPLATE.to_template(),
            Confirm::new(),
            ConfirmState::new(),
            component_ids,
        )
    }
}

impl Messenger for Confirm {}

impl AppMessageHandler for Confirm {
    fn handle_message<F>(
        value: CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut AppState,
        context: Context<'_, AppState>,
        component_ids: &HashMap<String, ComponentId<String>>,
        fun: F,
    ) where
        F: Fn(&mut AppState, Context<'_, AppState>),
    {
        let event: String = ident.into();
        match event.as_str() {
            "confirm__cancel" => {
                fun(state, context);
            }

            "confirm__action" => {
                match serde_json::from_str::<ComponentMessages>(&value.to_string()) {
                    Ok(component_messages) => match component_messages {
                        ComponentMessages::DeleteCommandConfirmMessage(delete_msg) => {
                            if let Some(id) = component_ids.get(delete_msg.payload.waiting) {
                                let _ = MessageSender::send_message(
                                    *id,
                                    ComponentMessages::DeleteCommandConfirmMessage(delete_msg),
                                    context.emitter.clone(),
                                );
                            }
                        }

                        ComponentMessages::DeleteAnnoucementConfirmMessage(delete_msg) => {
                            if let Some(id) = component_ids.get(delete_msg.payload.waiting) {
                                let _ = MessageSender::send_message(
                                    *id,
                                    ComponentMessages::DeleteAnnoucementConfirmMessage(delete_msg),
                                    context.emitter.clone(),
                                );
                            }
                        }

                        ComponentMessages::DeleteRewardConfirmMessage(delete_msg) => {
                            if let Some(id) = component_ids.get(delete_msg.payload.waiting) {
                                let _ = MessageSender::send_message(
                                    *id,
                                    ComponentMessages::DeleteRewardConfirmMessage(delete_msg),
                                    context.emitter.clone(),
                                );
                            }
                        }

                        ComponentMessages::DeleteActionConfirmMessage(delete_msg) => {
                            if let Some(id) = component_ids.get(delete_msg.payload.waiting) {
                                let _ = MessageSender::send_message(
                                    *id,
                                    ComponentMessages::DeleteActionConfirmMessage(delete_msg),
                                    context.emitter.clone(),
                                );
                            }
                        }

                        _ => (),
                    },

                    Err(error) => send_to_error_log(error.to_string(), format!("Could not deserialize {}", value)),
                }

                fun(state, context);
            }

            _ => {}
        }
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
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            anathema::component::KeyCode::Char(char) => match char {
                'y' => context.publish("submit_confirmation", |state| &state.component_message),
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

                ComponentMessages::DeleteAnnoucementConfirmMessage(delete_msg) => {
                    state.title.set(delete_msg.payload.title.to_string());
                    state.message.set(delete_msg.payload.message.to_string());
                    state.waiting.set(delete_msg.payload.waiting.to_string());
                }

                ComponentMessages::DeleteRewardConfirmMessage(delete_msg) => {
                    state.title.set(delete_msg.payload.title.to_string());
                    state.message.set(delete_msg.payload.message.to_string());
                    state.waiting.set(delete_msg.payload.waiting.to_string());
                }

                ComponentMessages::DeleteActionConfirmMessage(delete_msg) => {
                    state.title.set(delete_msg.payload.title.to_string());
                    state.message.set(delete_msg.payload.message.to_string());
                    state.waiting.set(delete_msg.payload.waiting.to_string());
                }

                _ => {}
            },

            Err(err) => {
                // println!("Could not deserialize ComponentMessage: {}", self.message);
                println!("{}", err);
            }
        }
    }
}
