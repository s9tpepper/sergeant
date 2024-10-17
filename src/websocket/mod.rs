use std::{
    net::TcpListener,
    sync::{
        mpsc::{channel, Receiver, Sender},
        OnceLock, RwLock,
    },
    thread::spawn,
};

use tungstenite::{accept, Message};

use crate::twitch::{pubsub::send_to_error_log, ChannelMessages};

static SENDERS: OnceLock<RwLock<Vec<Sender<ChannelMessages>>>> = OnceLock::new();

pub fn start_websocket(messages_rx: Receiver<ChannelMessages>) {
    spawn(move || loop {
        if let Ok(new_message) = messages_rx.try_recv() {
            let senders = SENDERS.get_or_init(|| RwLock::new(Vec::new()));
            let senders_read = senders.read().unwrap();

            for sender in senders_read.iter() {
                let _ = sender.send(new_message.clone());
            }
        }
    });

    // TODO: Make the websocket server port configurable
    let server = TcpListener::bind("0.0.0.0:8765").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let (sender, receiver) = channel::<ChannelMessages>();
            let senders = SENDERS.get_or_init(|| RwLock::new(Vec::new()));
            let mut senders_write = senders.write().unwrap();
            senders_write.push(sender.clone());
            drop(senders_write);

            loop {
                let new_message = receiver.try_recv();
                if let Ok(message) = new_message {
                    let json = serde_json::to_string(&message).unwrap();
                    let send_result = websocket.send(Message::Text(json));

                    if let Err(send_error) = send_result {
                        send_to_error_log(
                            "websocket: Error sending the message to the websocket".into(),
                            send_error.to_string(),
                        );
                        break;
                    }
                }
            }
        });
    }
}
