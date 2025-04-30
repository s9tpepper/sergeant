use std::{
    sync::{mpsc::channel, Arc},
    thread,
};

use log::{error, info};

use crate::{
    announcements::start_announcements,
    channel::ChannelMessages,
    chat::start_chat_frontend,
    twitch::{
        assets::{get_channel_badges, get_global_badges},
        auth::{get_credentials, refresh_token, validate},
        eventsub::start_eventsub,
    },
    websocket::start_websocket,
};

// TODO: Refactor this function to clean it up
pub fn start_chat(
    twitch_name: Option<&str>,
    oauth_token: Option<&str>,
    client_id: Option<&str>,
    skip_announcements: bool,
    test_mode: bool,
) -> anyhow::Result<()> {
    info!("start_chat()");

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
    info!("token status retrieved.");

    let global_badges = get_global_badges(&oauth_token, &client_id)?;
    let channel_badges = get_channel_badges(&client_id, &oauth_token)?;
    info!("badges created");

    // NOTE: Take a look at what is happening in ChannelMessages and remove unused/uneeded things
    let (transmitter, receiver) = channel::<ChannelMessages>();
    info!("channel message channel created");

    let (socket_transmitter, socket_receiver) = channel::<ChannelMessages>();
    info!("websocket channel created");

    let announce_tx = transmitter.clone();
    let announce_websocket_tx = socket_transmitter.clone();
    thread::spawn(
        move || match start_announcements(announce_tx, announce_websocket_tx, skip_announcements) {
            Ok(_) => info!("Bot announcements started."),
            Err(announcements_error) => error!("Bot annoucements errored: {announcements_error}"),
        },
    );

    thread::spawn(|| {
        start_websocket(socket_receiver);
    });

    // NOTE: Keep this
    let id = client_id.clone();
    let token = oauth_token.clone();
    let eventsub_transmitter = transmitter.clone();
    let eventsub_to_websocket_transmitter = socket_transmitter.clone();

    thread::spawn(|| {
        info!("started eventsub thread");
        start_eventsub(token, id, eventsub_transmitter, eventsub_to_websocket_transmitter);
    });

    // NOTE: Ratatui stuff, this should go away for Anathema
    // install_hooks()?;
    // App::new(&twitch_name).run(rx, socket_tx.clone())?;
    // restore()?;

    start_chat_frontend(twitch_name, receiver, global_badges, channel_badges, test_mode)?;

    Ok(())
}
