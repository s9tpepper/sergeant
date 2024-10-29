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
pub struct RewardsView {
    rewards: Option<Vec<Reward>>,
}

impl AppComponent for RewardsView {}
impl RewardsView {
    pub fn register(
        builder: &mut RuntimeBuilder<TuiBackend, ()>,
        component_ids: &mut HashMap<String, ComponentId<String>>,
    ) {
        <crate::admin::components::rewards_view::RewardsView as AppComponent>::register_component(
            builder,
            "rewards_view",
            LIST_VIEW_TEMPLATE,
            RewardsView::new(),
            ListViewState {
                item_row_fill: "‧".to_string().into(),
                current_last_index: 4.into(),
                visible_items: 5.into(),
                default_color: "#313131".to_string().into(),
                selected_color: "#ffffff".to_string().into(),
                min_width: 10.into(),
                title_foreground: "#131313".to_string().into(),
                title_heading: "Rewards".to_string().into(),
                window_list: List::empty(),
                ..Default::default()
            },
            component_ids,
        )
    }
}

impl RewardsView {
    pub fn new() -> Self {
        RewardsView { rewards: None }
    }
}

impl AppMessageHandler for RewardsView {
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
            "rewards__close" => {
                state.main_display.set(MainDisplay::Info);
                context.set_focus("id", "app");
            }

            "rewards__add" => {
                state.floating_window.set(FloatingWindow::AddReward);
                context.set_focus("id", "add_reward_window");
            }

            "rewards__edit_selection" => {
                if let Ok(item) = serde_json::from_str::<Reward>(&value.to_string()) {
                    state.floating_window.set(FloatingWindow::EditReward);
                    context.set_focus("id", "edit_reward_window");

                    if let Some(id) = component_ids.get("reward_name_input") {
                        let _ = context.emitter.emit(*id, item.name);
                    }

                    if let Some(id) = component_ids.get("reward_shell_command_input") {
                        let _ = context.emitter.emit(*id, item.command);
                    }
                }
            }

            "rewards__delete_selection" => {
                if let Ok(item) = serde_json::from_str::<Reward>(&value.to_string()) {
                    if let Some(id) = component_ids.get("confirm_window") {
                        state.floating_window.set(FloatingWindow::Confirm);
                        context.set_focus("id", "confirm_window");

                        let message = format!("Are you sure you want to delete: {}", item.name);
                        let confirmation_details = DeleteRewardConfirmationDetails {
                            title: "Delete Reward",
                            waiting: "rewards_view",
                            message: &message,
                            item,
                        };

                        let _ = MessageSender::send_message(
                            *id,
                            ComponentMessages::DeleteRewardConfirmMessage(DeleteRewardConfirmMessage {
                                payload: confirmation_details,
                            }),
                            context.emitter.clone(),
                        );
                    }
                }
            }

            "rewards__show_delete_error" => {
                state.floating_window.set(FloatingWindow::Error);
                state.error_message.set(String::from("Could not delete reward"));
                context.set_focus("id", "error_window");

                if let Some(id) = component_ids.get("error_window") {
                    let _ = MessageSender::send_message(
                        *id,
                        ComponentMessages::RewardsViewReload(RewardsViewReload {}),
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

impl Component for RewardsView {
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
                'a' => context.publish("rewards__add", |state| &state.cursor),
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
pub struct Reward {
    pub name: String,
    pub command: String,
    pub option: Option<String>,
    pub index: usize,
}

impl Reward {
    fn from_sgtfile(value: &SgtFile) -> Self {
        let contents = value.contents.to_ref().to_string();
        let values = contents.split_once(" ");

        match values {
            Some((command, option)) => Reward {
                index: 0,
                name: value.name.to_ref().to_string(),
                command: String::from(command),
                option: Some(String::from(option)),
            },
            None => Reward {
                index: 0,
                name: value.name.to_ref().to_string(),
                command: contents,
                option: None,
            },
        }
    }
}

impl From<SgtFile> for Reward {
    fn from(value: SgtFile) -> Self {
        Reward::from_sgtfile(&value)
    }
}

impl From<Reward> for Item {
    fn from(value: Reward) -> Self {
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

impl ListComponent<'_, Reward> for RewardsView {
    fn get_list(&self) -> Vec<Reward> {
        match &self.rewards {
            Some(commands) => commands.to_vec(),
            None => vec![],
        }
    }

    fn load(&mut self, _state: &mut super::list_view::ListViewState) {
        match get_list_with_contents("chat_rewards") {
            Ok(files) => {
                let rewards: Vec<Reward> = files
                    .iter()
                    .enumerate()
                    .map(|(index, command)| {
                        let mut reward = Reward::from_sgtfile(command);
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
