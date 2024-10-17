use std::error::Error;

use anathema::{
    component::{
        Component,
        KeyCode::{Char, Esc},
    },
    state::{List, State, Value},
};

use crate::commands::{get_list_with_contents, Command};

#[derive(Default)]
pub struct CommandsView;

#[derive(State)]
pub struct CommandsViewState {
    commands: Value<List<Command>>,
    none: Value<u8>,
}

impl CommandsViewState {
    pub fn new() -> Self {
        CommandsViewState {
            none: 0.into(),
            commands: List::empty(),
        }
    }
}

impl CommandsView {
    fn load_commands(&self) -> Result<Vec<Command>, Box<dyn Error>> {
        get_list_with_contents("chat_commands")
    }
}

impl Component for CommandsView {
    type State = CommandsViewState;
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
        match self.load_commands() {
            Ok(commands) => {
                loop {
                    if state.commands.len() == 0 {
                        break;
                    }

                    state.commands.remove(0);
                }

                commands.iter().for_each(|command| {
                    state.commands.push(Command {
                        name: command.name.to_ref().clone().into(),
                        contents: command.contents.to_ref().clone().into(),
                    });
                });
            }

            Err(_err) => loop {
                if state.commands.len() == 0 {
                    break;
                }

                state.commands.remove(0);

                // TODO: Raise error message dialog
            },
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
            Char(_char) => {}

            Esc => context.publish("close_commands_view", |state| &state.none),

            _ => {}
        }
    }
}
