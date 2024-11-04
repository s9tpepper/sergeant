use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId, Emitter, KeyCode},
    prelude::{ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::{CommonVal, State, Value},
};

use crate::{
    admin::{
        components::{app::AppMessageHandler, MessageSender},
        messages::{ComponentMessages, RewardsViewReload},
        templates::ADD_REWARD_TEMPLATE,
        AppComponent,
    },
    commands::add_reward,
};

#[derive(Default)]
pub struct AddReward {
    component_ids: HashMap<String, ComponentId<String>>,
}

impl AddReward {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_reward::AddReward as AppComponent>::register_component(
            builder,
            "add_reward_window",
            ADD_REWARD_TEMPLATE.to_template(),
            AddReward {
                component_ids: component_ids.to_owned(),
            },
            AddRewardState::new(),
            component_ids,
        )
    }

    fn clear_inputs(&self, emitter: Emitter) {
        let inputs = ["add_reward_name_input", "add_reward_command_input"];

        inputs.iter().for_each(|ident| {
            if let Some(id) = self.component_ids.get(*ident) {
                let _ = emitter.emit(*id, String::from(""));
            }
        })
    }

    fn send_clear(
        context: &anathema::prelude::Context<'_, crate::admin::components::app::AppState>,
        component_ids: &HashMap<String, ComponentId<String>>,
    ) {
        if let Some(id) = component_ids.get("add_reward_window") {
            let _ = MessageSender::send_message(*id, ComponentMessages::AddRewardClear, context.emitter.clone());
        }
    }
}

impl AppComponent for AddReward {}

impl AppMessageHandler for AddReward {
    fn handle_message<F>(
        value: CommonVal<'_>,
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
            "add_reward__cancel" => {
                AddReward::send_clear(&context, component_ids);
                fun(state, context);
            }

            "add_reward__submit" => {
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

                AddReward::send_clear(&context, component_ids);
                fun(state, context);
            }

            _ => {}
        }
    }
}

#[derive(Default, State)]
pub struct AddRewardState {
    reward: Value<NewReward>,
}

#[derive(Default, Debug)]
pub struct NewReward {
    pub name: Value<String>,
    pub shell_command: Value<String>,
    pub common: Value<String>,
}

impl State for NewReward {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        let str = self.common.to_ref().to_string().clone().into_boxed_str();

        Some(CommonVal::Str(Box::leak(str)))
    }
}

impl From<CommonVal<'_>> for NewReward {
    fn from(value: CommonVal) -> Self {
        if let Some((name, shell_command)) = value.to_string().split_once("::::") {
            return NewReward {
                name: name.to_string().into(),
                shell_command: shell_command.to_string().into(),
                common: format!("{}::::{}", name, shell_command).into(),
            };
        }

        NewReward {
            name: String::from("").into(),
            shell_command: String::from("").into(),
            common: String::from("::::").into(),
        }
    }
}

impl AddRewardState {
    pub fn new() -> Self {
        AddRewardState {
            reward: NewReward {
                name: String::from("").into(),
                shell_command: String::from("").into(),
                common: String::from("::::").into(),
            }
            .into(),
        }
    }
}

impl Component for AddReward {
    type State = AddRewardState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        context: anathema::prelude::Context<'_, Self::State>,
    ) {
        let Ok(component_message) = serde_json::from_str::<ComponentMessages>(&message) else {
            return;
        };

        if let ComponentMessages::AddRewardClear = component_message {
            state.reward.to_mut().name.set(String::from(""));
            state.reward.to_mut().shell_command.set(String::from(""));

            self.clear_inputs(context.emitter.clone());
        }
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
            "add_reward__name_update" => {
                state.reward.to_mut().name.set(value.to_string());

                let common = format!("{}::::{}", value, *state.reward.to_ref().shell_command.to_ref(),);
                state.reward.to_mut().common.set(common);
            }

            "add_reward__shell_command_update" => {
                state.reward.to_mut().shell_command.set(value.to_string());

                let common = format!("{}::::{}", *state.reward.to_ref().name.to_ref(), value,);
                state.reward.to_mut().common.set(common);
            }

            "add_reward__name_focus_change" => {
                context.set_focus("id", "add_reward_window");
            }

            "add_reward__shell_command_focus_change" => {
                context.set_focus("id", "add_reward_window");
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
                's' => context.publish("add_reward__submit", |state| &state.reward),

                'c' => context.publish("add_reward__cancel", |state| &state.reward),

                'n' => context.set_focus("id", "add_reward_name_input"),

                'h' => context.set_focus("id", "add_reward_shell_command_input"),

                _ => {}
            },

            KeyCode::Esc => context.publish("add_reward__cancel", |state| &state.reward),

            _ => {}
        }
    }
}
