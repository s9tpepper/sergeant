use std::{
    net::TcpListener,
    sync::{
        mpsc::{channel, Receiver, Sender},
        OnceLock, RwLock,
    },
    thread::{self, spawn},
};

use log::{error, info};
use tungstenite::{accept, Message};

use crate::channel::ChannelMessages;

static SENDERS: OnceLock<RwLock<Vec<Sender<ChannelMessages>>>> = OnceLock::new();

pub fn websocket(messages_rx: Receiver<ChannelMessages>) {
    thread::spawn(|| {
        start_websocket(messages_rx);
    });
}

pub fn start_websocket(messages_rx: Receiver<ChannelMessages>) {
    info!("[websocket] Starting websocket server...");

    spawn(move || loop {
        match messages_rx.recv() {
            Ok(new_message) => {
                let senders = SENDERS.get_or_init(|| RwLock::new(Vec::new()));
                let senders_read = senders.read().unwrap();

                for sender in senders_read.iter() {
                    match sender.send(new_message.clone()) {
                        Ok(_) => {}
                        Err(error) => error!("[websocket] Error sending message to websocket client: {error}"),
                    }
                }
            }
            Err(error) => {
                error!("[websocket] Error receiving a message on messages_rx: {error}")
            }
        }
    });

    // TODO: Make the websocket server port configurable
    let Ok(server) = TcpListener::bind("0.0.0.0:8766") else {
        error!("[websocket] Error binding to port 8766 for websocket server.");
        return;
    };

    info!("[websocket] Server listening on port 8766");

    for stream in server.incoming() {
        info!("[websocket] Received incoming connection...");
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            info!("[websocket] connnection accepted.");

            let (sender, receiver) = channel::<ChannelMessages>();
            let senders = SENDERS.get_or_init(|| RwLock::new(Vec::new()));
            let mut senders_write = senders.write().unwrap();
            senders_write.push(sender.clone());
            info!("[websocket] Sender channel registered");

            drop(senders_write);

            loop {
                let new_message = receiver.recv();
                if let Ok(message) = new_message {
                    let json = serde_json::to_string(&message).unwrap();
                    let send_result = websocket.send(Message::Text(json.into()));

                    if let Err(send_error) = send_result {
                        error!("websocket: Error sending the message to the websocket: {send_error}");
                        break;
                    }
                } else {
                    error!("[websocket] Error receiving new message to relay to websocket clients");
                }
            }
        });
    }
}
