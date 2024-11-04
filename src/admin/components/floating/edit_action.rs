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
        templates::EDIT_ACTION_TEMPLATE,
        AppComponent,
    },
    commands::add_action,
};

use super::add_action::Action;

#[derive(Default)]
pub struct EditAction {
    component_ids: HashMap<String, ComponentId<String>>,
}

impl EditAction {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_action::AddAction as AppComponent>::register_component(
            builder,
            "edit_action_window",
            EDIT_ACTION_TEMPLATE.to_template(),
            EditAction {
                component_ids: component_ids.to_owned(),
            },
            EditActionState::new(),
            component_ids,
        )
    }

    fn clear_inputs(&self, emitter: Emitter) {
        let inputs = [
            "edit_action_name_input",
            "edit_action_command_input",
            "edit_action_option_input",
        ];

        inputs.iter().for_each(|ident| {
            if let Some(id) = self.component_ids.get(*ident) {
                let _ = emitter.emit(*id, String::from(""));
            }
        })
    }
}

impl AppComponent for EditAction {}

impl AppMessageHandler for EditAction {
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
            "edit_action__cancel" => {
                if let Some(id) = component_ids.get("edit_action_window") {
                    let _ =
                        MessageSender::send_message(*id, ComponentMessages::EditActionClear, context.emitter.clone());
                }

                fun(state, context);
            }

            "edit_action__submit" => {
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

                if let Some(id) = component_ids.get("edit_action_window") {
                    let _ =
                        MessageSender::send_message(*id, ComponentMessages::EditActionClear, context.emitter.clone());
                }

                fun(state, context);
            }

            _ => {}
        }
    }
}

#[derive(Default, State)]
pub struct EditActionState {
    action: Value<Action>,
}

impl EditActionState {
    pub fn new() -> Self {
        EditActionState {
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

impl Component for EditAction {
    type State = EditActionState;
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
            "edit_action__name_update" => {
                state.action.to_mut().name.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    value,
                    *state.action.to_ref().command.to_ref(),
                    *state.action.to_ref().option.to_ref()
                );
                state.action.to_mut().common.set(common);
            }

            "edit_action__command_update" => {
                state.action.to_mut().command.set(value.to_string());

                let common = format!(
                    "{}::::{}::::{}",
                    *state.action.to_ref().name.to_ref(),
                    value,
                    *state.action.to_ref().option.to_ref()
                );
                state.action.to_mut().common.set(common);
            }

            "edit_action__option_update" => {
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

            "edit_action__name_focus_change" => {
                context.set_focus("id", "edit_action_window");
            }

            "edit_action__command_focus_change" => {
                context.set_focus("id", "edit_action_window");
            }

            "edit_action__option_focus_change" => {
                context.set_focus("id", "edit_action_window");
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
                's' => context.publish("edit_action__submit", |state| &state.action),

                'c' => context.publish("edit_action__cancel", |state| &state.action),

                'n' => context.set_focus("id", "edit_action_name_input"),

                'h' => context.set_focus("id", "edit_action_command_input"),

                'o' => context.set_focus("id", "edit_action_option_input"),

                _ => {}
            },

            KeyCode::Esc => context.publish("edit_action__cancel", |state| &state.action),

            _ => {}
        }
    }
}
