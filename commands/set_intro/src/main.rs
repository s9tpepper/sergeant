use std::{fs::File, io::BufReader};

use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use play::play;
use set::set;

mod db;
mod play;
mod set;

// TODO: remove this
mod tests_main;

#[derive(Subcommand)]
enum Cmds {
    /// Play the intro audio for a Twitch user
    Play {
        /// Your Twitch username
        name: String,
    },

    /// Set the intro audio for a Twitch user
    Set {
        ///YouTube URL to get audio from
        url: String,

        ///Start time of the video
        start: String,

        ///End time of the video
        end: String,

        ///Message to send to chat
        message: String,

        ///File name to save audio as
        file_name: String,

        ///Name of Twitch user
        name: String,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Cmds,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();
    match cli.commands {
        Cmds::Play { name } => play(&name).await?,

        Cmds::Set {
            url,
            start,
            end,
            message,
            file_name,
            name,
        } => {
            set(&url, &start, &end, &message, &file_name, &name).await?;
        }
    }

    Ok(())
}
