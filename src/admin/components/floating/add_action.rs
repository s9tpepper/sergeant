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
        messages::{ActionsViewReload, ComponentMessages},
        templates::ADD_ACTION_TEMPLATE,
        AppComponent,
    },
    commands::add_action,
};

#[derive(Default)]
pub struct AddAction {
    component_ids: HashMap<String, ComponentId<String>>,
}

impl AddAction {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_action::AddAction as AppComponent>::register_component(
            builder,
            "add_action_window",
            ADD_ACTION_TEMPLATE.to_template(),
            AddAction {
                component_ids: component_ids.to_owned(),
            },
            AddActionState::new(),
            component_ids,
        )
    }

    fn clear_inputs(&self, emitter: Emitter) {
        let inputs = [
            "add_action_name_input",
            "add_action_command_input",
            "add_action_option_input",
        ];

        inputs.iter().for_each(|ident| {
            if let Some(id) = self.component_ids.get(*ident) {
                let _ = emitter.emit(*id, String::from(""));
            }
        })
    }
}

impl AppComponent for AddAction {}

impl AppMessageHandler for AddAction {
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
            "add_action__cancel" => {
                if let Some(id) = component_ids.get("add_action_window") {
                    let _ =
                        MessageSender::send_message(*id, ComponentMessages::AddActionClear, context.emitter.clone());
                }

                fun(state, context);
            }

            "add_action__submit" => {
                let action: Action = value.into();

                let command = action.command.to_ref().clone();
                let option = action.option.to_ref().clone();
                let cli = format!("{command} {option}");

                match add_action(&action.name.to_ref(), cli.trim()) {
                    Ok(_) => {
                        if let Some(id) = component_ids.get("actions_view") {
                            let _ = MessageSender::send_message(
                                *id,
                                ComponentMessages::ActionsViewReload(ActionsViewReload {}),
                                context.emitter.clone(),
                            );
                        }
                    }

                    Err(_) => {
                        // TODO: bring up a message window with an error message
                    }
                };

                if let Some(id) = component_ids.get("add_action_window") {
                    let _ =
                        MessageSender::send_message(*id, ComponentMessages::AddActionClear, context.emitter.clone());
                }

                fun(state, context);
            }

            _ => {}
        }
    }
}

#[derive(Default, State)]
pub struct AddActionState {
    action: Value<Action>,
}

#[derive(Default, Debug)]
pub struct Action {
    pub name: Value<String>,
    pub command: Value<String>,
    pub option: Value<String>,
    pub common: Value<String>,
}

impl State for Action {
    fn to_common(&self) -> Option<CommonVal<'_>> {
        let str = self.common.to_ref().to_string().clone().into_boxed_str();

        Some(CommonVal::Str(Box::leak(str)))
    }
}

impl From<CommonVal<'_>> for Action {
    fn from(value: CommonVal) -> Self {
        let serialized_announcement = value.to_string();
        let fields: Vec<&str> = serialized_announcement.split("::::").collect();

        match fields.as_slice() {
            [name, command, option] => Action {
                name: String::from(*name).into(),
                command: String::from(*command).into(),
                option: String::from(*option).into(),
                common: serialized_announcement.into(),
            },

            _ => Action {
                name: String::from("").into(),
                command: String::from("").into(),
                option: String::from("").into(),
                common: String::from("::::").into(),
            },
        }
    }
}

impl AddActionState {
    pub fn new() -> Self {
        AddActionState {
            action: Action {
                name: String::from("").into(),
                command: String::from("").into(),
                option: String::from("").into(),
                common: String::from("::::").into(),
            }
            .into(),
        }
    }
}

impl Component for AddAction {
    type State = AddActionState;
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

        if let ComponentMessages::AddActionClear = component_message {
            state.action.to_mut().name.set(String::from(""));
            state.action.to_mut().command.set(String::from(""));
            state.action.to_mut().option.set(String::from(""));

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
            "add_action__name_update" => {
                state.action.to_mut().name.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    value,
                    *state.action.to_ref().command.to_ref(),
                    *state.action.to_ref().option.to_ref()
                );
                state.action.to_mut().common.set(common);
            }

            "add_action__command_update" => {
                state.action.to_mut().command.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    *state.action.to_ref().name.to_ref(),
                    value,
                    *state.action.to_ref().option.to_ref()
                );
                state.action.to_mut().common.set(common);
            }

            "add_action__option_update" => {
                let timing = value.to_string();
                state.action.to_mut().option.set(timing);

                let common = format!(
                    "{}::::{}::::{}",
                    *state.action.to_ref().name.to_ref(),
                    *state.action.to_ref().command.to_ref(),
                    value
                );
                state.action.to_mut().common.set(common);
            }

            "add_action__name_focus_change" => {
                context.set_focus("id", "add_action_window");
            }

            "add_action__command_focus_change" => {
                context.set_focus("id", "add_action_window");
            }

            "add_action__option_focus_change" => {
                context.set_focus("id", "add_action_window");
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
                's' => context.publish("add_action__submit", |state| &state.action),

                'c' => context.publish("add_action__cancel", |state| &state.action),

                'n' => context.set_focus("id", "add_action_name_input"),

                'h' => context.set_focus("id", "add_action_command_input"),

                'o' => context.set_focus("id", "add_action_option_input"),

                _ => {}
            },

            KeyCode::Esc => context.publish("add_action__cancel", |state| &state.action),

            _ => {}
        }
    }
}
