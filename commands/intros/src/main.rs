use std::{
    fs::File,
    io::BufReader,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use play::play;
use set::set;

mod db;
mod play;
mod set;

#[derive(Subcommand)]
enum Cmds {
    /// Play the intro audio for a Twitch user
    Play {
        /// Your Twitch username
        name: String,
    },

    /// Set the intro audio for a Twitch user
    Set {
        /// Twitch name of user that claimed a redeem
        twitch_name: String,

        /// User input string from the Twitch redeem
        input: String,
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

        Cmds::Set { twitch_name, input } => {
            // Set an audio clip from a YouTube URL to play as your intro audio clip when you join
            // chat. Must follow format: youtube_url 00:00:10 00:00:18

            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?;
            let file_name = format!("{twitch_name}_{}", timestamp.as_secs());

            let mut args = input.split(' ');
            let extraction_params = (args.next(), args.next(), args.next());

            #[allow(clippy::single_match)]
            match extraction_params {
                (Some(url), Some(start), Some(end)) => {
                    set(url, start, end, &file_name, &twitch_name).await?;
                }

                _ => {}
            }
        }
    }

    Ok(())
}
