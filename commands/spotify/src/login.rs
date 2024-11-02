use demand::Input;
use lib::fs::get_project_directory;
use serde::{Deserialize, Serialize};
use spotify_rs::{
    auth::{CsrfVerifier, NoVerifier, Token, UnAuthenticated},
    client::Client,
    AuthCodeClient, AuthCodeFlow, RedirectUrl,
};
use url::Url;

#[derive(Debug)]
pub struct AuthCode {
    code: String,
    state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyDetails {
    id: String,
    secret: String,
    callback: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub spotify_details: SpotifyDetails,
}

pub async fn login() -> anyhow::Result<()> {
    let spotify_details = prompt_spotify_details();

    let (client, url) = get_auth_code_client(&spotify_details)?;

    let callback_url = prompt_auth_url(url.as_ref());
    let AuthCode { code, state } = get_auth_code(&callback_url)?;

    let spotify = client.authenticate(code, state).await?;

    save_tokens(spotify, spotify_details)?;

    println!("You are logged in to Spotify");

    Ok(())
}

fn save_tokens(
    spotify: Client<Token, AuthCodeFlow, NoVerifier>,
    spotify_details: SpotifyDetails,
) -> anyhow::Result<()> {
    let access_token = spotify.access_token();
    let refresh_token = spotify
        .refresh_token()
        .ok_or(anyhow::Error::msg("Missing refresh token"))?;

    let tokens = Tokens {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.to_string(),
        spotify_details,
    };

    let file_contents = serde_json::to_string(&tokens)?;
    let file_dir = get_project_directory("SgtSpotify", "tokens")?;
    let file_path = file_dir.join("tokens.json");

    std::fs::write(file_path, file_contents)?;

    Ok(())
}

fn get_auth_code(callback_url: &str) -> anyhow::Result<AuthCode> {
    let parsed_url = url::Url::parse(&callback_url)?;
    let query = parsed_url.query_pairs();

    let mut code = String::new();
    let mut state = String::new();
    query.into_iter().for_each(|(key, value)| {
        #[allow(clippy::single_match)]
        match key {
            std::borrow::Cow::Borrowed(k) => match k {
                "code" => {
                    code = value.into_owned();
                }

                "state" => {
                    state = value.into_owned();
                }

                _ => {}
            },

            _ => {}
        };
    });

    Ok(AuthCode { code, state })
}

pub fn get_auth_code_flow(spotify_details: &SpotifyDetails) -> anyhow::Result<AuthCodeFlow> {
    let scopes = vec![
        "user-library-read",
        "playlist-read-private",
        "user-modify-playback-state",
    ];

    Ok(AuthCodeFlow::new(
        spotify_details.id.to_owned(),
        spotify_details.secret.to_owned(),
        scopes,
    ))
}

fn get_auth_code_client(
    spotify_details: &SpotifyDetails,
) -> anyhow::Result<(Client<UnAuthenticated, AuthCodeFlow, CsrfVerifier>, Url)> {
    let auth_code_flow = get_auth_code_flow(&spotify_details)?;

    let redirect_url = RedirectUrl::new(spotify_details.callback.to_owned())?;
    let auto_refresh = true;

    Ok(AuthCodeClient::new(auth_code_flow, redirect_url, auto_refresh))
}

fn prompt_spotify_details() -> SpotifyDetails {
    let prompt = Input::new("Spotify Client ID")
        .placeholder("Spotify Client ID")
        .password(true)
        .prompt("Client ID: ");

    let id = prompt.run().expect("error running input");

    let prompt = Input::new("Spotify Client Secret")
        .placeholder("Spotify Client Secret")
        .password(true)
        .prompt("Client Secret: ");

    let secret = prompt.run().expect("error running input");

    let prompt = Input::new("Spotify Client Callback")
        .placeholder("Spotify Client Callback")
        .prompt("Client Callback URL: ");

    let callback = prompt.run().expect("error running input");

    SpotifyDetails { id, secret, callback }
}

fn prompt_auth_url(url: &str) -> String {
    println!("\n{url}\n");

    let prompt = Input::new("Visit this URL above ☝👆:")
        .description("Then enter the callback url it returns below")
        .placeholder("Enter callback url you get back from the auth  url")
        .password(true)
        .prompt("Callback URL: ");

    prompt.run().expect("error running input")
}
