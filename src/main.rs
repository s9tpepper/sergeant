use clap::{Parser, Subcommand};
use colored::Colorize;
use dotenv::dotenv;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::any::type_name;
use std::fs;
use std::thread;
use tungstenite::Message::Text;

use serde_json::json;
use sergeant::commands::{
    add_chat_command, authenticate_with_twitch, get_list_announcements, get_list_commands,
    remove_chat_command, TokenStatus,
};
use sergeant::twitch::client::{TwitchClient, User};
use sergeant::twitch::messages::{get_badges, TwitchApiResponse};
use sergeant::utils::get_data_directory;
use std::error::Error;
use std::process::exit;
use std::sync::Arc;

use tungstenite::connect;

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
        (Some(twitch_name), Some(oauth_token), Some(client_id)) => {
            Ok((twitch_name, oauth_token, client_id))
        }

        _ => {
            let error_message = "You need to provide credentials via positional args, env vars, or by running the login command";
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

#[derive(Deserialize, Serialize)]
struct SocketMessage {
    r#type: String,
    data: SocketMessageData,
}

#[derive(Deserialize, Serialize)]
struct SocketMessageData {
    topic: String,
    message: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct MessageData {
    pub data: SubMessage,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
enum SubMessage {
    Points(Box<ChannelPointsData>),
    Sub(SubscribeEvent),
    Bits(BitsEvent),
    // Bits {},
    // BitsUnlocks {},
}

#[derive(Deserialize, Serialize, Debug)]
struct BitsEvent {
    pub is_anonymous: bool,
    pub message_type: String,
    pub data: BitsEventData,
}

#[derive(Deserialize, Serialize, Debug)]
struct BitsEventData {
    pub user_name: String,
    pub chat_message: String,
    pub bits_used: u64,
    pub total_bits_used: u64,
    pub context: String, // cheer
}

#[derive(Deserialize, Serialize, Debug)]
struct SubscribeEvent {
    pub topic: String,
    pub message: SubscribeMessage,
}

#[derive(Deserialize, Serialize, Debug)]
struct SubscribeMessage {
    pub display_name: String,   // some_person
    pub cumulative_months: u64, // 9
    pub streak_months: u64,     // 3
    pub context: String,        // subgift, resub
    pub sub_message: String,    // A message, possibly with emotes
}

#[derive(Deserialize, Serialize, Debug)]
struct ChannelPointsData {
    pub timestamp: String,
    pub redemption: Redemption,
}

#[derive(Deserialize, Serialize, Debug)]
struct UserReference {
    pub id: String,
    pub login: String,
    pub display_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Redemption {
    pub user: UserReference,
    // user_input: String,
    pub status: String,
    pub reward: Reward,
}

#[derive(Deserialize, Serialize, Debug)]
struct Reward {
    pub title: String,
    pub prompt: String,
    pub cost: u64,
}

fn connect_to_pub_sub(
    _twitch_name: Arc<String>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
) -> Result<(), Box<dyn Error>> {
    let get_users_url = "https://api.twitch.tv/helix/users";
    let mut response = reqwest::blocking::Client::new()
        .get(get_users_url)
        .header(
            "Authorization",
            format!("Bearer {}", oauth_token.replace("oauth:", "")),
        )
        .header("Client-Id", client_id.to_string())
        .send()?
        .json::<TwitchApiResponse<Vec<User>>>()?;

    let user = response.data.swap_remove(0);
    let twitch_pub_sub = "wss://pubsub-edge.twitch.tv";

    match connect(twitch_pub_sub) {
        Ok((mut socket, _response)) => {
            let channel_bits = "channel-bits-events-v2.".to_string() + &user.id;
            // let channel_bits_unlocks = "channel-bits-badge-unlocks.".to_string() + &user.id;
            let channel_points = "channel-points-channel-v1.".to_string() + &user.id;
            let channel_subscribe = "channel-subscribe-events-v1.".to_string() + &user.id;

            let auth_token = oauth_token.to_string().replace("oauth:", "");

            let topics_message = json!({
                "type": "LISTEN",
                "nonce": "182947398358192374",
                "data": {
                    "auth_token": auth_token,
                    "topics": [channel_bits, channel_points, channel_subscribe]
                }
            });

            socket.send(topics_message.to_string().into()).unwrap();

            loop {
                if let Ok(Text(message)) = socket.read() {
                    if !message.contains("MESSAGE") {
                        continue;
                    }

                    let socket_message: SocketMessage = serde_json::from_str(&message.to_string())?;
                    let sub_message = &socket_message.data.message;
                    let sub_message: MessageData = serde_json::from_str(sub_message)?;

                    match sub_message.data {
                        SubMessage::Points(sub_message) => {
                            let message = format!(
                                "{} redeemed {} for {}",
                                sub_message.redemption.user.display_name,
                                sub_message.redemption.reward.title,
                                sub_message.redemption.reward.cost
                            );

                            println!("{}", message.to_string().green().bold());
                        }

                        SubMessage::Sub(sub_message) => {
                            let message = format!(
                                "{} has subscribed for {} months, currently on a {} month steak.",
                                sub_message.message.display_name,
                                sub_message.message.cumulative_months,
                                sub_message.message.streak_months
                            );

                            println!("{}", message.to_string().blue().bold());
                        }

                        SubMessage::Bits(sub_message) => {
                            let message = format!(
                                "{} has cheered {} bits",
                                sub_message.data.user_name, sub_message.data.bits_used
                            );

                            println!("{}", message.to_string().white().on_green().bold());
                        }
                    }
                }
            }
        }

        Err(error) => {
            println!("I got an error...");
            println!("{}", error);
        }
    }

    Ok(())
}

async fn start_chat(
    twitch_name: Arc<String>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
) -> AsyncResult<()> {
    get_badges(&oauth_token, &client_id).await?;

    let name = twitch_name.clone();
    let token = oauth_token.clone();
    let id = client_id.clone();
    thread::spawn(|| {
        connect_to_pub_sub(name, token, id).unwrap();
    });

    let mut twitch_client = TwitchClient::new(twitch_name, oauth_token, client_id, vec![]).await?;

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

async fn start_login_flow() {
    let result = authenticate_with_twitch().await;
    if result.is_err() {
        exit(5);
    }
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
            let (name, token, id) = get_credentials(twitch_name, oauth_token, client_id).unwrap();

            let name = Arc::new(name);
            let id = Arc::new(id);
            let token = Arc::new(token);

            let _ = start_chat(name, token, id).await;
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

        // Cmds::SendMessage { message } => {
        //     send_message(&message);
        // }
        Cmds::Login => {
            start_login_flow().await;
        }
    };
}
