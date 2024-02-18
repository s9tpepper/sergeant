use dotenv::dotenv;
use clap::{Parser, Subcommand};

use ferris_twitch::commands::{add_chat_command, get_list_commands, remove_chat_command};
use ferris_twitch::twitch::client::TwitchClient;
use ferris_twitch::twitch::messages::get_badges;
use std::error::Error;
use std::process::exit;

type AsyncResult<T> = Result<T, Box<dyn Error>>;

#[derive(Subcommand)]
enum Cmds {
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
    commands: Cmds,
} 

async fn start_chat(twitch_name: String, oauth_token: String, client_id: String) -> AsyncResult<()> {
    get_badges(&oauth_token, &client_id).await?;
    let mut twitch_client = TwitchClient::new(twitch_name, oauth_token, vec![]).await?;
    twitch_client.start_receiving().await?;

    Ok(())
}

fn list_commands() {
    let result = get_list_commands();
    if result.is_err() {
        exit(2)
    }

    if let Ok(list) = &result {
        println!("Available chat commands:");
        for item in list {
            println!("- {}", item);
        }
    }
}


fn add_command(command_name: &str, message: &str) {
    let result = add_chat_command(command_name, message);
    if result.is_err() {
        exit(1)
    }
}

fn remove_command(command_name: &str) {
    let result = remove_chat_command(command_name);
    if result.is_err() {
        exit(3)
    }
}

fn list_announcements() {
    println!("List announcements goes here");
    todo!();
}

fn add_announcement(announcement_name: &str, timing: &str, message: &str) {
    println!("Add announcement here {} {} {}", announcement_name, timing, message);
    todo!();
}

fn remove_announcement(announcement_name: &str) {
    println!("Remove announcement here {}", announcement_name);
    todo!();
}

fn send_message(message: &str) {
    println!("Send message {}", message);
    todo!();
}

#[tokio::main]
async fn main() {
    // Load ENV vars with DotEnv
    dotenv().ok();

    let cli = Cli::parse();
    match cli.commands {
        Cmds::Chat{ twitch_name, oauth_token, client_id } => { 
            let _ = start_chat(twitch_name, oauth_token, client_id).await;
        },
        Cmds::ListCommands => { list_commands(); },
        Cmds::AddCommand{ command_name, message } => { add_command(&command_name, &message); },
        Cmds::RemoveCommand{ command_name } => { remove_command(&command_name); },
        Cmds::ListAnnouncements => { list_announcements(); },
        Cmds::AddAnnouncement{ announcement_name, timing, message } => {
            add_announcement(&announcement_name, &timing, &message);
        },
        Cmds::RemoveAnnouncement{ announcement_name } => {
            remove_announcement(&announcement_name);
        },
        Cmds::SendMessage{ message } => {
            send_message(&message);
        },
    };
}

