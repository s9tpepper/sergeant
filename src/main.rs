use clap::{Parser, Subcommand};
use dotenv::dotenv;
use sergeant::commands::add_reward;
use sergeant::commands::list_rewards;
use sergeant::commands::remove_reward;
use sergeant::tui::init;
use sergeant::tui::install_hooks;
use sergeant::tui::restore;
use sergeant::tui::App;
use sergeant::twitch::announcements::start_announcements;
use sergeant::twitch::irc::TwitchIRC;
use sergeant::twitch::parse::get_badges;
use sergeant::twitch::pubsub::connect_to_pub_sub;
use sergeant::twitch::ChannelMessages;
use std::fs;
use std::sync::mpsc::channel;
use std::thread;

use sergeant::commands::{
    add_chat_command, authenticate_with_twitch, get_list_announcements, get_list_commands, remove_chat_command,
    TokenStatus,
};

use sergeant::utils::get_data_directory;
use std::error::Error;
use std::process::exit;
use std::sync::Arc;

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
enum RewardSubCmds {
    /// List rewards
    List,

    /// Add a reward command
    Add {
        /// The name of the reward as it is named on Twitch
        name: String,

        /// The cli command to execute for the reward
        cli: String,
    },

    /// Remove a command
    Remove {
        /// The name of the reward to remove
        name: String,
    },
}

#[derive(Subcommand)]
enum Cmds {
    /// Start Twitch Chat client
    Chat {
        /// Your Twitch username
        #[arg(long, short = 'n', env = "TWITCH_NAME")]
        twitch_name: Option<String>,

        /// Your Twitch OAuth Token
        #[arg(long, short = 't', env = "OAUTH_TOKEN")]
        oauth_token: Option<String>,

        /// Your Twitch app client ID
        #[arg(long, short, env = "CLIENT_ID")]
        client_id: Option<String>,
    },

    /// Manage chat commands
    Commands {
        #[command(subcommand)]
        cmd: SubCmds,
    },

    /// Manage chat rewards
    Rewards {
        #[command(subcommand)]
        cmd: RewardSubCmds,
    },

    // Send a chat message
    // SendMessage {
    //     /// The message body
    //     #[arg(long, short)]
    //     message: String,
    // },
    /// Login to Twitch and get a token
    Login,
}

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    commands: Cmds,
}

fn get_credentials(
    twitch_name: Option<String>,
    oauth_token: Option<String>,
    client_id: Option<String>,
) -> Result<(String, String, String), Box<dyn Error>> {
    match (twitch_name, oauth_token, client_id) {
        (Some(twitch_name), Some(oauth_token), Some(client_id)) => Ok((twitch_name, oauth_token, client_id)),

        _ => {
            let error_message =
                "You need to provide credentials via positional args, env vars, or by running the login command";
            let mut data_dir = get_data_directory(Some("token")).expect(error_message);
            data_dir.push("oath_token.txt");
            let token_file = fs::read_to_string(data_dir)?;

            let token_status: TokenStatus = serde_json::from_str(&token_file)?;

            if token_status.success {
                Ok((
                    token_status.username.unwrap(),
                    format!("oauth:{}", token_status.token.unwrap()),
                    token_status.client_id.unwrap(),
                ))
            } else {
                panic!("{}", error_message);
            }
        }
    }
}

fn start_chat(twitch_name: Arc<String>, oauth_token: Arc<String>, client_id: Arc<String>) -> AsyncResult<()> {
    get_badges(&oauth_token, &client_id)?;

    let (pubsub_tx, rx) = channel::<ChannelMessages>();
    let announce_tx = pubsub_tx.clone();
    let chat_tx = pubsub_tx.clone();

    let token = oauth_token.clone();
    let id = client_id.clone();
    thread::spawn(|| {
        connect_to_pub_sub(token, id, pubsub_tx).unwrap();
    });

    let token = oauth_token.clone();
    let name = twitch_name.clone();
    thread::spawn(|| {
        let _ = start_announcements(name, token, announce_tx);
    });

    thread::spawn(|| {
        let mut twitch_irc = TwitchIRC::new(twitch_name, oauth_token, chat_tx);
        twitch_irc.listen();
    });

    install_hooks()?;
    let mut terminal = init()?;
    App::default().run(&mut terminal, rx)?;
    restore()?;

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

// fn send_message(message: &str) {
//     println!("Send message {}", message);
//     todo!();
// }

fn start_login_flow() {
    let result = authenticate_with_twitch();
    if result.is_err() {
        exit(5);
    }
}

fn main() {
    // Load ENV vars with DotEnv
    dotenv().ok();

    let cli = Cli::parse();
    match cli.commands {
        Cmds::Chat {
            twitch_name,
            oauth_token,
            client_id,
        } => {
            let (name, token, id) = get_credentials(twitch_name, oauth_token, client_id).unwrap();

            let name = Arc::new(name);
            let id = Arc::new(id);
            let token = Arc::new(token);

            let _ = start_chat(name, token, id);
        }

        Cmds::Commands { cmd } => match cmd {
            SubCmds::List => {
                list_commands();
            }
            SubCmds::Add { name, message, timing } => {
                add_command(&name, &message, timing);
            }
            SubCmds::Remove { name } => {
                remove_command(&name);
            }
        },

        Cmds::Rewards { cmd } => match cmd {
            RewardSubCmds::List => {
                list_rewards();
            }
            RewardSubCmds::Add { name, cli } => {
                let _ = add_reward(&name, &cli);
            }
            RewardSubCmds::Remove { name } => {
                let _ = remove_reward(&name);
            }
        },

        // Cmds::SendMessage { message } => {
        //     send_message(&message);
        // }
        Cmds::Login => {
            start_login_flow();
        }
    };
}
