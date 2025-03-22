use crate::chat::ratatui_app::ratatui;

use anathema::{prelude::*, state::List};

use anathema_app::{AnathemaApp, AnathemaAppState};
use log::{error, info};

use std::sync::mpsc::Receiver;

use crate::channel::ChannelMessages;

mod anathema_app;
mod ratatui_app;

pub fn start_chat_frontend(twitch_name: String, tui_receiver: Receiver<ChannelMessages>) -> anyhow::Result<()> {
    // anathema(twitch_name, tui_receiver)

    // TODO: Get the twitch user name from somewhere (login)
    ratatui(twitch_name, tui_receiver)
}

fn anathema(twitch_name: String, tui_receiver: Receiver<ChannelMessages>) -> anyhow::Result<()> {
    info!("App::run()");

    let tui = TuiBackend::builder()
        .enable_alt_screen()
        .enable_raw_mode()
        .hide_cursor()
        .finish();

    match tui {
        Ok(tui_backend) => {
            let doc = Document::new("@app");
            let mut runtime_builder = Runtime::builder(doc, tui_backend);

            runtime_builder.register_component(
                "app",
                "src/chat/templates/app.aml",
                AnathemaApp::new(twitch_name, tui_receiver),
                AnathemaAppState {
                    log: List::from_iter([]),
                    test_field: "This is a string".to_string().into(),
                },
            )?;

            let mut runtime = runtime_builder.finish().unwrap();
            runtime.run();

            // runtime_builder.finish(tui_backend.size(), |runtime| runtime.run(tui_backend))?;
        }

        Err(error) => {
            error!("Error starting TUI {error}");
            panic!("Error starting TUI {error}");
        }
    };

    Ok(())
}
