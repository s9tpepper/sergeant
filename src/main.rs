use dotenv::dotenv;
use clap::{Parser, Subcommand};

use ferris_twitch::twitch::client::TwitchClient;
use ferris_twitch::twitch::messages::get_badges;
use std::error::Error;

type AsyncResult<T> = Result<T, Box<dyn Error>>;

#[derive(Subcommand)]
enum Commands {
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
    }
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


#[tokio::main]
async fn main() {
    // Load ENV vars with DotEnv
    dotenv().ok();

    let cli = Cli::parse();
    let _ = match cli.commands {
        Commands::Chat{ twitch_name, oauth_token, client_id } => start_chat(twitch_name, oauth_token, client_id).await
    };
}

