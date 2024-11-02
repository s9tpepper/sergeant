use clap::{Parser, Subcommand};
use login::login;
use queue::queue;
use song::song;

mod login;
mod queue;
mod song;

#[derive(Subcommand)]
enum Cmds {
    /// Play the intro audio for a Twitch user
    Login,

    /// Set the intro audio for a Twitch user
    Queue {
        /// Twitch name of user that claimed a redeem
        twitch_name: String,

        /// Spotify URI to queue in the current queue
        uri: String,
    },

    Song {
        /// Twitch name of user that claimed a redeem
        display_name: String,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Cmds,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _ = match cli.commands {
        Cmds::Login => login().await,

        Cmds::Queue { uri, .. } => queue(&uri).await,

        Cmds::Song { .. } => song().await,
    };

    Ok(())
}
