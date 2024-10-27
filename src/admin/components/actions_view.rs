use std::{collections::HashMap, thread::sleep, time::Duration};

use anathema::{
    component::{Component, ComponentId, KeyCode::Char},
    prelude::{Context, TuiBackend},
    runtime::RuntimeBuilder,
    state::List,
};
use serde::{Deserialize, Serialize};

use crate::{
    admin::{
        messages::{ComponentMessages, DeleteRewardConfirmMessage, DeleteRewardConfirmationDetails, RewardsViewReload},
        templates::LIST_VIEW_TEMPLATE,
        AppComponent,
    },
    commands::{get_list_with_contents, remove_reward, SgtFile},
};

use super::{
    app::{AppMessageHandler, FloatingWindow, MainDisplay},
    list_view::{Item, ListComponent, ListViewState},
    MessageSender,
};

#[derive(Default)]
pub struct ActionsView {
    rewards: Option<Vec<Action>>,
}

impl AppComponent for ActionsView {}
impl ActionsView {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::actions_view::ActionsView as AppComponent>::register_component(
            builder,
            "actions_view",
            LIST_VIEW_TEMPLATE,
            ActionsView::new(),
            ListViewState {
                item_row_fill: "‧".to_string().into(),
                current_last_index: 4.into(),
                visible_items: 5.into(),
                default_color: "#313131".to_string().into(),
                selected_color: "#ffffff".to_string().into(),
                min_width: 10.into(),
                title_foreground: "#131313".to_string().into(),
                title_heading: "Actions".to_string().into(),
                window_list: List::empty(),
                ..Default::default()
            },
            component_ids,
        )
    }
}

impl ActionsView {
    pub fn new() -> Self {
        ActionsView { rewards: None }
    }
}

impl AppMessageHandler for ActionsView {
    fn handle_message<F>(
        _value: anathema::state::CommonVal<'_>,
        ident: impl Into<String>,
        state: &mut super::app::AppState,
        mut context: Context<'_, super::app::AppState>,
        _component_ids: &HashMap<String, ComponentId<String>>,
        _fun: F,
    ) where
        F: Fn(&mut super::app::AppState, Context<'_, super::app::AppState>),
    {
        let event: String = ident.into();
        match event.as_str() {
            "actions__close" => {
                state.main_display.set(MainDisplay::InfoView);
                context.set_focus("id", "app");
            }

            "actions__add" => {
                state.floating_window.set(FloatingWindow::AddAction);
                context.set_focus("id", "add_action_window");
            }

            // "actions__edit_selection" => {
            //     if let Ok(item) = serde_json::from_str::<Action>(&value.to_string()) {
            //         state.floating_window.set(FloatingWindow::Editaction);
            //         context.set_focus("id", "edit_action_window");
            //
            //         if let Some(id) = component_ids.get("action_name_input") {
            //             let _ = context.emitter.emit(*id, item.name);
            //         }
            //
            //         if let Some(id) = component_ids.get("action_shell_command_input") {
            //             let _ = context.emitter.emit(*id, item.command);
            //         }
            //     }
            // }
            //
            // "actions__delete_selection" => {
            //     if let Ok(item) = serde_json::from_str::<Action>(&value.to_string()) {
            //         if let Some(id) = component_ids.get("confirm_window") {
            //             state.floating_window.set(FloatingWindow::Confirm);
            //             context.set_focus("id", "confirm_window");
            //
            //             let message = format!("Are you sure you want to delete: {}", item.name);
            //             let confirmation_details = DeleteactionConfirmationDetails {
            //                 title: "Delete action",
            //                 waiting: "actions_view",
            //                 message: &message,
            //                 item,
            //             };
            //
            //             let _ = MessageSender::send_message(
            //                 *id,
            //                 ComponentMessages::DeleteactionConfirmMessage(DeleteactionConfirmMessage {
            //                     payload: confirmation_details,
            //                 }),
            //                 context.emitter.clone(),
            //             );
            //         }
            //     }
            // }
            //
            // "actions__show_delete_error" => {
            //     state.floating_window.set(FloatingWindow::Error);
            //     state.error_message.set(String::from("Could not delete action"));
            //     context.set_focus("id", "error_window");
            //
            //     if let Some(id) = component_ids.get("error_window") {
            //         let _ = MessageSender::send_message(
            //             *id,
            //             ComponentMessages::actionsViewReload(actionsViewReload {}),
            //             context.emitter.clone(),
            //         );
            //     }
            //
            //     sleep(Duration::from_secs(5));
            //     fun(state, context);
            // }
            _ => {}
        }
    }
}

impl Component for ActionsView {
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
                ComponentMessages::RewardsViewReload(_) => self.load(state),

                ComponentMessages::DeleteRewardConfirmMessage(delete_confirmed) => {
                    match remove_reward(&delete_confirmed.payload.item.name) {
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
                'a' => context.publish("actions__add", |state| &state.cursor),
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
pub struct Action {
    pub name: String,
    pub command: String,
    pub option: Option<String>,
    pub index: usize,
}

impl Action {
    fn from_sgtfile(value: &SgtFile) -> Self {
        let contents = value.contents.to_ref().to_string();
        let values = contents.split_once(" ");

        match values {
            Some((command, option)) => Action {
                index: 0,
                name: value.name.to_ref().to_string(),
                command: String::from(command),
                option: Some(String::from(option)),
            },
            None => Action {
                index: 0,
                name: value.name.to_ref().to_string(),
                command: contents,
                option: None,
            },
        }
    }
}

impl From<SgtFile> for Action {
    fn from(value: SgtFile) -> Self {
        Action::from_sgtfile(&value)
    }
}

impl From<Action> for Item {
    fn from(value: Action) -> Self {
        let details = match value.option {
            Some(option) => format!("{}: {}", value.command, option),
            None => value.command,
        };

        Item {
            name: value.name.into(),
            details: details.into(),
            index: value.index.into(),
            color: "#333333".to_string().into(),
        }
    }
}

impl ListComponent<'_, Action> for ActionsView {
    fn get_list(&self) -> Vec<Action> {
        match &self.rewards {
            Some(commands) => commands.to_vec(),
            None => vec![],
        }
    }

    fn load(&mut self, _state: &mut super::list_view::ListViewState) {
        match get_list_with_contents("irc_actions") {
            Ok(files) => {
                let rewards: Vec<Action> = files
                    .iter()
                    .enumerate()
                    .map(|(index, command)| {
                        let mut reward = Action::from_sgtfile(command);
                        reward.index = index;

                        reward
                    })
                    .collect();

                self.rewards = Some(rewards);
            }

            Err(_) => {
                self.rewards = Some(vec![]);
            }
        }
    }
}
