use serde::{Deserialize, Serialize};

use super::components::{
    actions_view::Action, announcements::Announce, commands_view::Cmd, rewards_view::Reward, ComponentMessage,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteCommandConfirmationDetails<'msg> {
    pub title: &'msg str,
    pub message: &'msg str,
    pub waiting: &'msg str,
    pub item: Cmd,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteRewardConfirmationDetails<'msg> {
    pub title: &'msg str,
    pub message: &'msg str,
    pub waiting: &'msg str,
    pub item: Reward,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteAnnouncementConfirmationDetails<'msg> {
    pub title: &'msg str,
    pub message: &'msg str,
    pub waiting: &'msg str,
    pub item: Announce,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteActionConfirmationDetails<'msg> {
    pub title: &'msg str,
    pub message: &'msg str,
    pub waiting: &'msg str,
    pub item: Action,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentMessages<'msg> {
    #[serde(borrow)]
    DeleteCommandConfirmMessage(DeleteCommandConfirmMessage<'msg>),
    CommandsViewReload(CommandsViewReload),
    AnnouncementsViewReload(AnnouncementsViewReload),
    RewardsViewReload(RewardsViewReload),
    ActionsViewReload(ActionsViewReload),
    DeleteRewardConfirmMessage(DeleteRewardConfirmMessage<'msg>),
    DeleteAnnoucementConfirmMessage(DeleteAnnouncementConfirmMessage<'msg>),
    DeleteActionConfirmMessage(DeleteActionConfirmMessage<'msg>),
    InfoViewLoad(InfoViewLoad),
    AddCommandClear,
    AddActionClear,
    EditActionClear,
    AddRewardClear,
    AddAnnouncementClear,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteActionConfirmMessage<'msg> {
    #[serde(borrow)]
    pub payload: DeleteActionConfirmationDetails<'msg>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteAnnouncementConfirmMessage<'msg> {
    #[serde(borrow)]
    pub payload: DeleteAnnouncementConfirmationDetails<'msg>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteCommandConfirmMessage<'msg> {
    #[serde(borrow)]
    pub payload: DeleteCommandConfirmationDetails<'msg>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteRewardConfirmMessage<'msg> {
    #[serde(borrow)]
    pub payload: DeleteRewardConfirmationDetails<'msg>,
}

impl<'msg> ComponentMessage for DeleteCommandConfirmMessage<'msg> {
    fn get_type(&self) -> String {
        String::from("delete_command_confirmation")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionsViewReload;
impl ComponentMessage for ActionsViewReload {
    fn get_type(&self) -> String {
        String::from("reload_data")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnouncementsViewReload;
impl ComponentMessage for AnnouncementsViewReload {
    fn get_type(&self) -> String {
        String::from("reload_data")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewardsViewReload;
impl ComponentMessage for RewardsViewReload {
    fn get_type(&self) -> String {
        String::from("reload_data")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandsViewReload;
impl ComponentMessage for CommandsViewReload {
    fn get_type(&self) -> String {
        String::from("reload_data")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoViewLoad;

impl ComponentMessage for InfoViewLoad {
    fn get_type(&self) -> String {
        String::from("load_data")
    }
}
