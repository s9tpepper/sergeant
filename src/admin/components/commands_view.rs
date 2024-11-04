use std::{collections::HashMap, thread::sleep, time::Duration};

use anathema::{
    component::{Component, ComponentId, KeyCode::Char},
    prelude::{Context, ToSourceKind, TuiBackend},
    runtime::RuntimeBuilder,
    state::List,
};
use serde::{Deserialize, Serialize};

use crate::{
    admin::{
        messages::{
            CommandsViewReload, ComponentMessages, DeleteCommandConfirmMessage, DeleteCommandConfirmationDetails,
        },
        templates::LIST_VIEW_TEMPLATE,
        AppComponent,
    },
    commands::{get_list_with_contents, remove_chat_command, SgtFile},
};

use super::{
    app::{AppMessageHandler, FloatingWindow, MainDisplay},
    list_view::{Item, ListComponent, ListViewState},
    MessageSender,
};

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
            LIST_VIEW_TEMPLATE.to_template(),
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

impl AppMessageHandler for CommandsView {
    fn handle_message<F>(
        value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::app::AppState,
        mut context: Context<'_, super::app::AppState>,
        component_ids: &HashMap<String, ComponentId<String>>,
        fun: F,
    ) where
        F: Fn(&mut super::app::AppState, Context<'_, super::app::AppState>),
    {
        let event: String = ident.into();
        match event.as_str() {
            "commands__close_view" => {
                state.main_display.set(MainDisplay::Info);
                context.set_focus("id", "app");
            }

            "commands__add" => {
                state.floating_window.set(FloatingWindow::AddCommand);
                context.set_focus("id", "add_command_window");
            }

            "commands__edit_selection" => {
                if let Ok(item) = serde_json::from_str::<Cmd>(&value.to_string()) {
                    state.floating_window.set(FloatingWindow::EditCommand);
                    context.set_focus("id", "edit_command_window");

                    if let Some(id) = component_ids.get("cmd_name_input") {
                        let _ = context.emitter.emit(*id, item.name);
                    }

                    if let Some(id) = component_ids.get("cmd_output_input") {
                        let _ = context.emitter.emit(*id, item.contents);
                    }
                }
            }

            "commands__delete_selection" => {
                if let Ok(item) = serde_json::from_str::<Cmd>(&value.to_string()) {
                    if let Some(id) = component_ids.get("confirm_window") {
                        state.floating_window.set(FloatingWindow::Confirm);
                        context.set_focus("id", "confirm_window");

                        let message = format!("Are you sure you want to delete: {}", item.name);
                        let confirmation_details = DeleteCommandConfirmationDetails {
                            title: "Delete Command",
                            waiting: "commands_view",
                            message: &message,
                            item,
                        };

                        let _ = MessageSender::send_message(
                            *id,
                            ComponentMessages::DeleteCommandConfirmMessage(DeleteCommandConfirmMessage {
                                payload: confirmation_details,
                            }),
                            context.emitter.clone(),
                        );
                    }
                }
            }

            "commands__show_delete_error" => {
                state.floating_window.set(FloatingWindow::Error);
                state.error_message.set(String::from("Could not delete command"));
                context.set_focus("id", "error_window");

                if let Some(id) = component_ids.get("error_window") {
                    let _ = MessageSender::send_message(
                        *id,
                        ComponentMessages::CommandsViewReload(CommandsViewReload {}),
                        context.emitter.clone(),
                    );
                }

                sleep(Duration::from_secs(5));
                fun(state, context);
            }

            _ => {}
        }
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
                'a' => context.publish("commands__add", |state| &state.cursor),
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

impl From<SgtFile> for Cmd {
    fn from(value: SgtFile) -> Self {
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
