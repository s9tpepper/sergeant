use clap::{Parser, Subcommand};
use dotenv::dotenv;
use sergeant::tui::{install_hooks, restore, App};
use sergeant::twitch::api::{refresh_token, validate};
use sergeant::twitch::{
    announcements::start_announcements, irc::TwitchIRC, parse::get_badges, pubsub::connect_to_pub_sub, ChannelMessages,
};

use sergeant::commands::{
    add_action, add_chat_command, add_reward, authenticate_with_twitch, get_list_announcements, get_list_commands,
    list_actions, list_rewards, remove_action, remove_chat_command, remove_reward, TokenStatus,
};

use sergeant::utils::get_data_directory;
use sergeant::websocket::start_websocket;
use std::{
    error::Error,
    fs,
    process::exit,
    sync::{mpsc::channel, Arc},
    thread,
};

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

        /// The timing for the message in minutes
        timing: Option<usize>,
    },

    /// Remove a command
    Remove {
        /// The name of the command to remove
        name: String,
    },
}

#[derive(Subcommand)]
enum IrcActionSubCmds {
    /// List IRC Actions
    List,

    /// Add an IRC ation command
    Add {
        /// The name of the IRC message type as it is named in the IRC protocol
        name: String,

        /// The cli command to execute for as the IRC action
        cli: String,
    },

    /// Remove an IRC action
    Remove {
        /// The name of the IRC message type to remove
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

        /// Set to turn off announcements
        #[arg(long, short = 's', env = "SKIP_ANNOUNCEMENTS", default_value_t = false)]
        skip_announcements: bool,
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

    /// Manage IRC actions
    IrcActions {
        #[command(subcommand)]
        cmd: IrcActionSubCmds,
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
    _refresh: Option<String>,
) -> Result<(String, String, String, String), Box<dyn Error>> {
    match (twitch_name, oauth_token, client_id) {
        (Some(twitch_name), Some(oauth_token), Some(client_id)) => {
            Ok((twitch_name, oauth_token, client_id, String::new()))
        }

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
                    token_status.refresh.unwrap(),
                ))
            } else {
                panic!("{}", error_message);
            }
        }
    }
}

fn start_chat(
    twitch_name: Arc<String>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    refresh: Arc<String>,
    skip_announcements: bool,
) -> AsyncResult<()> {
    let validate_token_response = validate(&oauth_token);
    let mut token_status: Option<TokenStatus> = None;
    if validate_token_response.is_err() {
        let token_status_result = refresh_token(&refresh);
        if token_status_result.is_err() {
            panic!("Token refresh failed, unable to validate Twitch API access. Please login again.");
        }

        token_status = Some(token_status_result.unwrap());
    }

    // Shadow the function arguments if the tokens were refreshed
    let oauth_token = if let Some(token_status) = &token_status {
        // NOTE: Not exactly sure why token needs a clone here and client_id does not
        let token = token_status.token.clone().unwrap();
        Arc::new(token)
    } else {
        oauth_token
    };

    let client_id = if let Some(token_status) = token_status {
        let client_id = token_status.client_id.unwrap();
        Arc::new(client_id)
    } else {
        client_id
    };

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
    let id = client_id.clone();
    thread::spawn(move || {
        let _ = start_announcements(name, token, id, announce_tx, skip_announcements);
    });

    let id = client_id.clone();
    let token = oauth_token.clone();
    thread::spawn(|| {
        let mut twitch_irc = TwitchIRC::new(twitch_name, token, id, chat_tx);
        twitch_irc.listen();
    });

    let (socket_tx, socket_rx) = channel::<ChannelMessages>();
    thread::spawn(|| {
        start_websocket(socket_rx);
    });

    install_hooks()?;
    App::new().run(rx, socket_tx.clone())?;
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
            skip_announcements,
        } => {
            let (name, token, id, refresh) = get_credentials(twitch_name, oauth_token, client_id, None).unwrap();

            let name = Arc::new(name);
            let id = Arc::new(id);
            let token = Arc::new(token);
            let refresh = Arc::new(refresh);

            let _ = start_chat(name, token, id, refresh, skip_announcements);
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

        Cmds::IrcActions { cmd } => match cmd {
            IrcActionSubCmds::List => {
                list_actions();
            }
            IrcActionSubCmds::Add { name, cli } => {
                let _ = add_action(&name, &cli);
            }
            IrcActionSubCmds::Remove { name } => {
                let _ = remove_action(&name);
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
