use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId, KeyCode},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{State, Value},
};

use crate::{
    admin::{
        components::{app::AppMessageHandler, MessageSender},
        messages::{ComponentMessages, RewardsViewReload},
        templates::EDIT_REWARD_TEMPLATE,
        AppComponent,
    },
    commands::add_reward,
};

use super::add_reward::NewReward;

#[derive(Default)]
pub struct EditReward;

impl AppComponent for EditReward {}
impl EditReward {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::edit_reward::EditReward as AppComponent>::register_component(
            builder,
            "edit_reward_window",
            EDIT_REWARD_TEMPLATE,
            EditReward,
            EditRewardState::new(),
            component_ids,
        )
    }
}

#[derive(Default, State)]
pub struct EditRewardState {
    reward: Value<NewReward>,
}

impl EditRewardState {
    pub fn new() -> Self {
        EditRewardState {
            reward: NewReward {
                name: String::from("").into(),
                shell_command: String::from("").into(),
                common: String::from("::::").into(),
            }
            .into(),
        }
    }
}

impl AppMessageHandler for EditReward {
    fn handle_message<F>(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut crate::admin::components::app::AppState,
        context: anathema::prelude::Context<'_, crate::admin::components::app::AppState>,
        component_ids: &HashMap<String, ComponentId<String>>,
        fun: F,
    ) where
        F: Fn(
            &mut crate::admin::components::app::AppState,
            anathema::prelude::Context<'_, crate::admin::components::app::AppState>,
        ),
    {
        let event: String = ident.into();
        match event.as_str() {
            "edit_reward__cancel" => {
                if let Some(id) = component_ids.get("edit_reward_name_input") {
                    let _ = context.emitter.emit(*id, String::from(""));
                }

                if let Some(id) = component_ids.get("edit_reward_shell_command_input") {
                    let _ = context.emitter.emit(*id, String::from(""));
                }

                fun(state, context);
            }

            "edit_reward__submit" => {
                let reward: NewReward = value.into();

                match add_reward(&reward.name.to_ref(), &reward.shell_command.to_ref()) {
                    Ok(_) => {
                        if let Some(id) = component_ids.get("rewards_view") {
                            let _ = MessageSender::send_message(
                                *id,
                                ComponentMessages::RewardsViewReload(RewardsViewReload {}),
                                context.emitter.clone(),
                            );
                        }
                    }

                    Err(_) => {
                        // TODO: bring up a message window with an error message
                    }
                };

                fun(state, context);
            }

            _ => {}
        }
    }
}

impl Component for EditReward {
    type State = EditRewardState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn receive(
        &mut self,
        ident: &str,
        value: anathema::state::CommonVal<'_>,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match ident {
            "edit_reward__name_update" => {
                state.reward.to_mut().name.set(value.to_string());

                let common = format!("{}::::{}", value, *state.reward.to_ref().shell_command.to_ref());
                state.reward.to_mut().common.set(common);
            }

            "edit_reward__shell_command_update" => {
                state.reward.to_mut().shell_command.set(value.to_string());

                let common = format!("{}::::{}", *state.reward.to_ref().name.to_ref(), value);
                state.reward.to_mut().common.set(common);
            }

            "edit_reward__name_focus_change" => {
                context.set_focus("id", "edit_reward_window");
            }

            "edit_reward__shell_command_focus_change" => {
                context.set_focus("id", "edit_reward_window");
            }

            _ => {}
        }
    }

    fn on_key(
        &mut self,
        key: anathema::component::KeyEvent,
        _: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: anathema::prelude::Context<'_, Self::State>,
    ) {
        match key.code {
            KeyCode::Char(char) => match char {
                's' => {
                    context.publish("edit_reward__submit", |state| &state.reward);
                }

                'c' => context.publish("edit_reward__cancel", |state| &state.reward),

                'n' => context.set_focus("id", "edit_reward_name_input"),

                'h' => context.set_focus("id", "edit_reward_shell_command_input"),

                _ => {}
            },

            KeyCode::Esc => context.publish("edit_reward__cancel", |state| &state.reward),

            _ => {}
        }
    }
}
