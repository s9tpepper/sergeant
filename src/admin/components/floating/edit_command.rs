use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId, KeyCode},
    prelude::TuiBackend,
    runtime::RuntimeBuilder,
    state::{State, Value},
};

use crate::admin::{templates::EDIT_COMMAND_TEMPLATE, AppComponent};

use super::add_command::Command;

#[derive(Default)]
pub struct EditCommand;

impl AppComponent for EditCommand {}
impl EditCommand {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_command::AddCommand as AppComponent>::register_component(
            builder,
            "edit_command_window",
            EDIT_COMMAND_TEMPLATE,
            EditCommand,
            EditCommandState::new(),
            component_ids,
        )
    }
}

#[derive(Default, State)]
pub struct EditCommandState {
    command: Value<Command>,
}

impl EditCommandState {
    pub fn new() -> Self {
        EditCommandState {
            command: Command {
                name: String::from("").into(),
                output: String::from("").into(),
                common: String::from("::::").into(),
            }
            .into(),
        }
    }
}

impl Component for EditCommand {
    type State = EditCommandState;
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
            "name_update" => {
                state.command.to_mut().name.set(value.to_string());

                let common = format!("{}::::{}", value, *state.command.to_ref().output.to_ref());
                state.command.to_mut().common.set(common);
            }

            "output_update" => {
                state.command.to_mut().output.set(value.to_string());

                let common = format!("{}::::{}", *state.command.to_ref().name.to_ref(), value);
                state.command.to_mut().common.set(common);
            }

            "name_focus_change" => {
                context.set_focus("id", "add_command_window");
            }

            "output_focus_change" => {
                context.set_focus("id", "add_command_window");
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
                    context.publish("submit_edit_command", |state| &state.command);
                }

                'c' => context.publish("cancel_edit_command", |state| &state.command),

                'n' => context.set_focus("id", "edit_cmd_name_input"),

                'o' => context.set_focus("id", "edit_cmd_output_input"),

                _ => {}
            },

            KeyCode::Esc => context.publish("cancel_edit_command", |state| &state.command),

            _ => {}
        }
    }
}
