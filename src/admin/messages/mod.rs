use serde::{Deserialize, Serialize};

use super::components::{announcements::Announce, commands_view::Cmd, ComponentMessage};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteCommandConfirmationDetails<'msg> {
    pub title: &'msg str,
    pub message: &'msg str,
    pub waiting: &'msg str,
    pub item: Cmd,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DeleteAnnouncementConfirmationDetails<'msg> {
    pub title: &'msg str,
    pub message: &'msg str,
    pub waiting: &'msg str,
    pub item: Announce,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ComponentMessages<'msg> {
    #[serde(borrow)]
    DeleteCommandConfirmMessage(DeleteCommandConfirmMessage<'msg>),
    CommandsViewReload(CommandsViewReload),
    AnnouncementsViewReload(AnnouncementsViewReload),
    DeleteAnnoucementConfirmMessage(DeleteAnnouncementConfirmMessage<'msg>),
    InfoViewLoad(InfoViewLoad),
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

impl<'msg> ComponentMessage for DeleteCommandConfirmMessage<'msg> {
    fn get_type(&self) -> String {
        String::from("delete_command_confirmation")
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
