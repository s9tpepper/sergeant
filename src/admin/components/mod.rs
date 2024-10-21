use std::error::Error;

use anathema::component::{ComponentId, Emitter};
use serde::{Deserialize, Serialize};

pub mod app;
pub mod commands_view;
pub mod info_view;
pub mod list_view;

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentMessage<'msg> {
    r#type: &'msg str,
    payload: &'msg str,
}

pub trait Messenger {
    fn send_message(
        &self,
        target: ComponentId<String>,
        message: ComponentMessage,
        emitter: Emitter,
    ) -> Result<(), Box<dyn Error>> {
        if let Ok(msg) = serde_json::to_string(&message) {
            emitter.emit(target, msg)?
        }

        Err("Unable to send message".into())
    }
}
