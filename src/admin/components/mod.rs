use std::error::Error;

use anathema::component::{ComponentId, Emitter};

use super::messages::ComponentMessages;

pub mod app;
pub mod cmd_name_input;
pub mod cmd_output_input;
pub mod commands_view;
pub mod floating;
pub mod info_view;
pub mod inputs;
pub mod list_view;

pub trait ComponentMessage {
    #[allow(dead_code)]
    fn get_type(&self) -> String;
}

pub trait Messenger {
    fn send_message(
        &self,
        target: ComponentId<String>,
        message: ComponentMessages,
        emitter: Emitter,
    ) -> Result<(), Box<dyn Error>> {
        match serde_json::to_string(&message) {
            Ok(msg) => {
                emitter.emit(target, msg)?;
                Ok(())
            }
            Err(error) => Err(Box::new(error)),
        }
    }
}
