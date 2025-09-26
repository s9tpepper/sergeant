use std::{
    sync::{
        mpsc::{channel, Sender},
        Arc,
    },
    thread,
};

use log::{error, info};
use thiserror::Error;

use crate::{
    announcements::start_announcements,
    channel::ChannelMessages,
    chat::start_chat_frontend,
    twitch::{
        assets::{get_channel_badges, get_global_badges},
        auth::{get_credentials, refresh_token, validate},
        eventsub::start_eventsub,
    },
    websocket::{start_websocket, websocket},
};

#[derive(Debug, Error)]
pub enum OauthError {
    #[error("Error retrieving the Twitch OAuth credentials")]
    Credentials(#[from] anyhow::Error),
}

struct OauthToken {
    twitch_name: Arc<String>,
    oauth_token: Arc<String>,
    client_id: Arc<String>,
    refresh: Arc<String>,
}

fn get_oauth_credentials(
    twitch_name: Option<&str>,
    oauth_token: Option<&str>,
    client_id: Option<&str>,
) -> Result<OauthToken, OauthError> {
    let (twitch_name, token, id, refresh) = get_credentials(twitch_name, oauth_token, client_id)?;

    let token_status = match validate(&token) {
        Ok(_) => None,
        Err(_) => match refresh_token(&refresh) {
            Ok(token_status) => Some(token_status),
            Err(_) => panic!("Token refresh failed, unable to validate Twitch API access. Please login again."),
        },
    };
    info!("token validated.");

    let (oauth_token, client_id) = match token_status {
        Some(token_status) => (
            Arc::new(token_status.token.unwrap_or(token)),
            Arc::new(token_status.client_id.unwrap_or(id)),
        ),
        None => (Arc::new(token), Arc::new(id)),
    };

    Ok(OauthToken {
        twitch_name: twitch_name.into(),
        oauth_token,
        client_id,
        refresh: refresh.into(),
    })
}

fn announcements(announce_tx: Sender<ChannelMessages>, websocket_tx: Sender<ChannelMessages>, skip: bool) {
    thread::spawn(move || match start_announcements(announce_tx, websocket_tx, skip) {
        Ok(_) => info!("Bot announcements started."),
        Err(announcements_error) => error!("Bot annoucements errored: {announcements_error}"),
    });
}

fn event_sub(
    token: Arc<String>,
    id: Arc<String>,
    eventsub_transmitter: Sender<ChannelMessages>,
    eventsub_to_websocket_transmitter: Sender<ChannelMessages>,
) {
    thread::spawn(|| {
        info!("started eventsub thread");
        start_eventsub(token, id, eventsub_transmitter, eventsub_to_websocket_transmitter);
    });
}

// TODO: Refactor this function to clean it up
pub fn start_chat(
    twitch_name: Option<&str>,
    oauth_token: Option<&str>,
    client_id: Option<&str>,
    skip_announcements: bool,
    test_mode: bool,
) -> anyhow::Result<()> {
    info!("start_chat()");

    let Ok(OauthToken {
        twitch_name,
        oauth_token,
        client_id,
        ..
    }) = get_oauth_credentials(twitch_name, oauth_token, client_id)
    else {
        panic!("[ERROR] Could not get Twitch OAuth credentials");
    };

    let global_badges = get_global_badges(&oauth_token, &client_id)?;
    let channel_badges = get_channel_badges(&client_id, &oauth_token)?;

    // NOTE: Take a look at what is happening in ChannelMessages and remove unused/uneeded things
    let (transmitter, receiver) = channel::<ChannelMessages>();
    let (socket_transmitter, socket_receiver) = channel::<ChannelMessages>();

    announcements(transmitter.clone(), socket_transmitter.clone(), skip_announcements);
    websocket(socket_receiver);
    event_sub(
        oauth_token.clone(),
        client_id.clone(),
        transmitter.clone(),
        socket_transmitter.clone(),
    );
    start_chat_frontend(
        (*twitch_name).clone(),
        receiver,
        global_badges,
        channel_badges,
        test_mode,
    )?;

    // NOTE: Ratatui stuff, this should go away for Anathema
    // install_hooks()?;
    // App::new(&twitch_name).run(rx, socket_tx.clone())?;
    // restore()?;

    Ok(())
}
