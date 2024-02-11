use dotenv::dotenv;
use twitch_chat_client::twitch::client::TwitchClient;
use std::env;
use std::error::Error;

const USERNAME_OAUTH_REQUIRED:&str = "You must provide either command line arguments, or environment variables, to connect to Twitch chat.";

fn get_credentials() -> (String, String) {
    dotenv().ok();

    let name_env_var = env::var("TWITCH_NAME").unwrap_or("".to_string());
    let token_env_var = env::var("OAUTH_TOKEN").unwrap_or("".to_string());

    let args: Vec<String> = env::args().collect();
    
    let blank_string = "".to_string();
    let name: String;
    let token: String;
    if args.len() == 3 {
        name = args[1].to_owned();
        token = args[2].to_owned();
    } else if name_env_var != blank_string && token_env_var != blank_string {
        name = name_env_var;
        token = token_env_var;
    } else {
        panic!("{USERNAME_OAUTH_REQUIRED}");
    }

    (name, token)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (name, token) = get_credentials();

    let mut twitch_client = TwitchClient::new(name, token, vec![]).await?;
    twitch_client.start_receiving().await?;

    Ok(())
}
