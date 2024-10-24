use std::collections::HashMap;

use anathema::{
    component::{Component, ComponentId, KeyCode::Char},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::List,
};
use serde::{Deserialize, Serialize};

use crate::{
    admin::{messages::ComponentMessages, templates::LIST_VIEW_TEMPLATE, AppComponent},
    commands::{get_list_with_contents, remove_chat_command, Command},
};

use super::list_view::{Item, ListComponent, ListViewState};

#[derive(Default)]
pub struct CommandsView {
    commands: Option<Vec<Cmd>>,
}

impl AppComponent for CommandsView {}
impl CommandsView {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::floating::add_command::AddCommand as AppComponent>::register_component(
            builder,
            "commands_view",
            LIST_VIEW_TEMPLATE,
            CommandsView::new(),
            ListViewState {
                item_row_fill: "‧".to_string().into(),
                current_last_index: 4.into(),
                visible_items: 5.into(),
                default_color: "#313131".to_string().into(),
                selected_color: "#ffffff".to_string().into(),
                min_width: 10.into(),
                title_background: "yellow".to_string().into(),
                title_foreground: "#131313".to_string().into(),
                title_heading: "Commands".to_string().into(),
                window_list: List::empty(),
                ..Default::default()
            },
            component_ids,
        )
    }
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

    fn message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
        _: anathema::widgets::Elements<'_, '_>,
        mut context: Context<'_, Self::State>,
    ) {
        if let Ok(msg) = serde_json::from_str::<ComponentMessages>(&message.to_string()) {
            match msg {
                ComponentMessages::CommandsViewReload(_) => self.load(state),

                ComponentMessages::DeleteCommandConfirmMessage(delete_confirmed) => {
                    match remove_chat_command(&delete_confirmed.payload.item.name) {
                        Ok(_) => {
                            self.load(state);
                            self.refresh(state);
                        }
                        Err(_) => context.publish("show_delete_command_error", |state| &state.cursor),
                    }
                }

                _ => {}
            }
        }
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

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct Cmd {
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
        match get_list_with_contents("chat_commands") {
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
        }
    }
}
