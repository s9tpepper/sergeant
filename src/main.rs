use std::fs::File;

use anyhow::bail;
use cli::command;
use log::{info, LevelFilter};
use simplelog::{Config, WriteLogger};

mod announcements;
mod channel;
mod chat;
mod chat_commands;
mod cli;
mod fs;
mod image_protocols;
mod twitch;
mod websocket;

fn main() -> anyhow::Result<()> {
    logger();

    match command() {
        Ok(_response) => {
            // println!("command() returned?... {response:?}");
            Ok(())
        }
        Err(error) => {
            bail!("Error executing command: {error}")
        }
    }
}

fn logger() {
    // TODO: Move this log file into the application directory
    // TODO: Enable this block with an env var
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("sergeant.log").unwrap(),
    );

    info!("Logging has been enabled");
}
