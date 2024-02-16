use dotenv::dotenv;
use clap::{Parser, Subcommand};

use ferris_twitch::twitch::client::TwitchClient;
use ferris_twitch::twitch::messages::get_badges;
use std::error::Error;

type AsyncResult<T> = Result<T, Box<dyn Error>>;

#[derive(Subcommand)]
enum Commands {
    /// Start Twitch Chat client
    Chat {
        /// Your Twitch username
        #[arg(long, short = 'n', env = "TWITCH_NAME")]
        twitch_name: String,

        /// Your Twitch OAuth Token
        #[arg(long, short = 't', env = "OAUTH_TOKEN")]
        oauth_token: String,

        /// Your Twitch app client ID
        #[arg(long, short, env = "CLIENT_ID")]
        client_id: String,
    },

    /// List chat commands
    ListCommands,

    /// Add a chat command
    AddCommand {
        /// The name of the chat command
        #[arg(long, short)]
        command_name: String,

        /// The message to send for the chat command
        #[arg(long, short)]
        message: String,
    },

    /// Remove a chat command
    RemoveCommand {
        /// The name of the chat command to remove
        #[arg(long, short)]
        command_name: String
    },

    /// List chat announcements
    ListAnnouncements,

    /// Add a recurring chat announcement
    AddAnnouncement {
        /// The name of the chat announcement
        #[arg(long, short)]
        announcement_name: String,

        /// How often to send the announcement 
        #[arg(long, short)]
        timing: String,

        /// The message body for the announcement
        #[arg(long, short)]
        message: String,
    },

    /// Remove a recurring chat announcement
    RemoveAnnouncement {
        /// The name of the chat announcement
        #[arg(long, short)]
        announcement_name: String,
    },

    /// Send a chat message
    SendMessage {
        /// The message body 
        #[arg(long, short)]
        message: String,
    },
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
} 

async fn start_chat(twitch_name: String, oauth_token: String, client_id: String) -> AsyncResult<()> {
    let badges = get_badges(&oauth_token, &client_id).await?;
    let mut twitch_client = TwitchClient::new(twitch_name, oauth_token, vec![], badges).await?;
    twitch_client.start_receiving().await?;

    Ok(())
}

fn list_commands() {
    println!("Chat commands go here");
    todo!();
}

fn add_command(command_name: String, message: String) {
    println!("Add command here {} {}", command_name, message);
    todo!();
}

fn remove_command(command_name: String) {
    println!("Remove command here {}", command_name);
    todo!();
}

fn list_announcements() {
    println!("List announcements goes here");
    todo!();
}

fn add_announcement(announcement_name: String, timing: String, message: String) {
    println!("Add announcement here {} {} {}", announcement_name, timing, message);
    todo!();
}

fn remove_announcement(announcement_name: String) {
    println!("Remove announcement here {}", announcement_name);
    todo!();
}

fn send_message(message: String) {
    println!("Send message {}", message);
    todo!();
}

#[tokio::main]
async fn main() {
    // Load ENV vars with DotEnv
    dotenv().ok();

    let cli = Cli::parse();
    let _ = match cli.commands {
        Commands::Chat{ twitch_name, oauth_token, client_id } => { 
            let _ = start_chat(twitch_name, oauth_token, client_id).await;
        },
        Commands::ListCommands => { list_commands(); },
        Commands::AddCommand{ command_name, message } => { add_command(command_name, message); },
        Commands::RemoveCommand{ command_name } => { remove_command(command_name); },
        Commands::ListAnnouncements => { list_announcements(); },
        Commands::AddAnnouncement{ announcement_name, timing, message } => {
            add_announcement(announcement_name, timing, message);
        },
        Commands::RemoveAnnouncement{ announcement_name } => {
            remove_announcement(announcement_name);
        },
        Commands::SendMessage{ message } => {
            send_message(message);
        },
    };
}

