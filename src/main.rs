use clap::{Parser, Subcommand};
use dotenv::dotenv;

use ferris_twitch::commands::{
    add_chat_command, get_list_announcements, get_list_commands, remove_chat_command,
};
use ferris_twitch::twitch::client::TwitchClient;
use ferris_twitch::twitch::messages::get_badges;
use std::error::Error;
use std::process::exit;

type AsyncResult<T> = Result<T, Box<dyn Error>>;

#[derive(Subcommand)]
enum SubCmds {
    /// List commands
    List,

    /// Add a chat command
    Add {
        /// The name of the command
        name: String,
        /// The message to send for the command
        message: String,

        /// The timing for the message
        timing: Option<usize>,
    },

    /// Remove a command
    Remove {
        /// The name of the command to remove
        name: String,
    },
}

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

    Commands {
        #[command(subcommand)]
        cmd: SubCmds,
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

async fn start_chat(
    twitch_name: String,
    oauth_token: String,
    client_id: String,
) -> AsyncResult<()> {
    get_badges(&oauth_token, &client_id).await?;
    let mut twitch_client = TwitchClient::new(twitch_name, oauth_token, vec![]).await?;

    // TODO: Add a flag here to toggle announcements on/off
    let mut announcements = twitch_client.get_announcements()?;

    twitch_client.start_receiving(&mut announcements).await?;

    Ok(())
}

fn list_commands() {
    let result = get_list_commands();
    if result.is_err() {
        exit(2)
    }

    if let Ok(list) = &result {
        if list.is_empty() {
            println!("Currently no chat announcements have been added.");
            return;
        }

        println!("Available chat commands:");
        for item in list {
            println!("- {}", item);
        }
    }

    list_announcements();
}

fn add_command(command_name: &str, message: &str, timing: Option<usize>) {
    let result = add_chat_command(command_name, message, timing);
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
    let result = get_list_announcements();
    if result.is_err() {
        exit(4)
    }

    if let Ok(list) = &result {
        if list.is_empty() {
            println!("Currently no chat announcements have been added.");
            return;
        }

        println!("Current chat announcements:");
        for item in list {
            println!("- {}", item);
        }
    }
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
        Cmds::Chat {
            twitch_name,
            oauth_token,
            client_id,
        } => {
            let _ = start_chat(twitch_name, oauth_token, client_id).await;
        }

        Cmds::Commands { cmd } => match cmd {
            SubCmds::List => {
                list_commands();
            }
            SubCmds::Add {
                name,
                message,
                timing,
            } => {
                add_command(&name, &message, timing);
            }
            SubCmds::Remove { name } => {
                remove_command(&name);
            }
        },

        Cmds::SendMessage { message } => {
            send_message(&message);
        }
    };
}
