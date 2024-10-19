use anathema::{
    component::{
        Component,
        KeyCode::{Char, Esc},
    },
    prelude::Context,
};
use serde::{Deserialize, Serialize};

use crate::commands::{get_list_with_contents, Command};

use super::list_view::{Item, ListComponent, ListViewState};

#[derive(Default)]
pub struct CommandsView {
    commands: Option<Vec<Cmd>>,
}

impl CommandsView {
    pub fn new() -> Self {
        CommandsView { commands: None }
    }
}

impl Component for CommandsView {
    type State = ListViewState;
    type Message = String;

    fn accept_focus(&self) -> bool {
        true
    }

    fn resize(
        &mut self,
        state: &mut Self::State,
        _elements: anathema::widgets::Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let size = context.viewport.size();
        if size.height == 0 {
            return;
        }

        let visible_items: u8 = (size.height.saturating_sub(5)) as u8;
        state.visible_items.set(visible_items);
        state.current_last_index.set(visible_items.saturating_sub(1));
    }

    fn on_focus(
        &mut self,
        state: &mut Self::State,
        elements: anathema::widgets::Elements<'_, '_>,
        context: Context<'_, Self::State>,
    ) {
        let size = context.viewport.size();
        if size.height == 0 {
            return;
        }

        let visible_items: u8 = (size.height.saturating_sub(5)) as u8;
        state.visible_items.set(visible_items);
        state.current_last_index.set(visible_items.saturating_sub(1));

        ListComponent::on_focus(self, state, elements, context);
    }

    fn on_key(
        &mut self,
        event: anathema::component::KeyEvent,
        state: &mut Self::State,
        elements: anathema::widgets::Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        match event.code {
            Char(char) => match char {
                'a' => context.publish("add_command", |state| &state.cursor),
                'e' => self.send_item_selection(state, context),
                'd' => self.send_delete_selection(state, context),
                'b' => self.send_cancel_view(context),

                _ => ListComponent::on_key(self, event, state, elements, context),
            },

            _ => ListComponent::on_key(self, event, state, elements, context),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Cmd {
    pub name: String,
    pub contents: String,
    pub index: usize,
}

impl From<Command> for Cmd {
    fn from(value: Command) -> Self {
        Cmd {
            name: value.name.to_ref().to_string(),
            contents: value.contents.to_ref().to_string(),
            index: 0,
        }
    }
}

impl From<Cmd> for Item {
    fn from(value: Cmd) -> Self {
        Item {
            name: value.name.into(),
            details: value.contents.into(),
            index: value.index.into(),
            color: "#333333".to_string().into(),
        }
    }
}

impl ListComponent<'_, Cmd> for CommandsView {
    fn get_list(&self) -> Vec<Cmd> {
        match &self.commands {
            Some(commands) => commands.to_vec(),
            None => vec![],
        }
    }

    fn load(&mut self, _state: &mut super::list_view::ListViewState) {
        match self.commands {
            Some(_) => {}
            None => match get_list_with_contents("chat_commands") {
                Ok(commands) => {
                    let cmds: Vec<Cmd> = commands
                        .iter()
                        .enumerate()
                        .map(|(index, command)| Cmd {
                            name: command.name.to_ref().clone(),
                            contents: command.contents.to_ref().clone(),
                            index,
                        })
                        .collect();

                    self.commands = Some(cmds);
                }
                Err(_) => {
                    self.commands = Some(vec![]);
                }
            },
        }
    }
}
