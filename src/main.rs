use dotenv::dotenv;
use ferris_twitch::twitch::client::TwitchClient;
use ferris_twitch::twitch::messages::{BadgeItem, TwitchApiResponse};
use std::env;
use std::error::Error;

type AsyncResult<T> = Result<T, Box<dyn Error>>;

const USERNAME_OAUTH_REQUIRED:&str = "You must provide either command line arguments, or environment variables, to connect to Twitch chat.";

fn get_credentials() -> (String, String, String) {
    dotenv().ok();

    let name_env_var = env::var("TWITCH_NAME").unwrap_or("".to_string());
    let token_env_var = env::var("OAUTH_TOKEN").unwrap_or("".to_string());
    let client_id_env_var = env::var("CLIENT_ID").unwrap_or("".to_string());

    let args: Vec<String> = env::args().collect();

    let blank_string = "".to_string();
    let name: String;
    let token: String;
    let client_id: String;
    if args.len() == 3 {
        name = args[1].to_owned();
        token = args[2].to_owned();
        client_id = args[3].to_owned();
    } else if name_env_var != blank_string && token_env_var != blank_string {
        name = name_env_var;
        token = token_env_var;
        client_id = client_id_env_var;
    } else {
        panic!("{USERNAME_OAUTH_REQUIRED}");
    }

    (name, token, client_id)
}

async fn get_badges(token: &str, client_id: &String) -> AsyncResult<Vec<BadgeItem>> {
    // Global badges: https://api.twitch.tv/helix/chat/badges/global
    // oauth:141241241241241
    //
    // scopes:
    // chat:read+chat:edit+channel:moderate+channel:read:redemptions+channel:bot+user:write:chat
    // base64: encoded app title
    // https://twitchtokengenerator.com/api/create
    //
    let response = reqwest::Client::new()
        .get("https://api.twitch.tv/helix/chat/badges/global")
        .header(
            "Authorization",
            format!("Bearer {}", token.replace("oauth:", "")),
        )
        .header("Client-Id", client_id)
        .send()
        .await?
        .json::<TwitchApiResponse<Vec<BadgeItem>>>()
        .await?;

    Ok(response.data)
}

#[tokio::main]
async fn main() -> AsyncResult<()> {
    let (name, token, client_id) = get_credentials();

    let badges = get_badges(&token, &client_id).await?;
    let mut twitch_client = TwitchClient::new(name, token, vec![], badges).await?;
    twitch_client.start_receiving().await?;

    Ok(())
}
