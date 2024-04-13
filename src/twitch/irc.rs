use std::{
    net::TcpStream,
    sync::{mpsc::Sender, Arc},
    thread::sleep,
    time::Duration,
};

use tungstenite::{stream::MaybeTlsStream, WebSocket};

use crate::twitch::parse::parse;

use super::{parse::TwitchMessage, pubsub::send_to_error_log, ChannelMessages};

pub struct TwitchIRC {
    tx: Sender<ChannelMessages>,
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    nickname: String,
    oauth_token: String,
}

const CONN_MAX_RETRIES: u8 = 3;
const MESSAGE_DELIMITER: &str = "\r\n";

fn connect(twitch_name: &Arc<String>, oauth_token: &Arc<String>, retry: u8) -> WebSocket<MaybeTlsStream<TcpStream>> {
    if retry == CONN_MAX_RETRIES {
        panic!("Could not connect to Twitch IRC servers, tried {retry} times.")
    }

    let delay = 5 * retry;
    let duration = Duration::new(delay.into(), 0);
    sleep(duration);

    match tungstenite::connect("wss://irc-ws.chat.twitch.tv:443") {
        Ok((mut socket, _)) => {
            if let Err(connection_error) = socket.send(format!("PASS {oauth_token}").into()) {
                send_to_error_log(connection_error.to_string(), "Error while sending PASS message".into());

                let next_retry = retry + 1;
                return connect(twitch_name, oauth_token, next_retry);
            }

            if let Err(nickname_error) = socket.send(format!("NICK {twitch_name}").into()) {
                send_to_error_log(nickname_error.to_string(), "Error while sending NICK message".into());

                let next_retry = retry + 1;
                return connect(twitch_name, oauth_token, next_retry);
            }

            if let Err(join_channel_error) = socket.send(format!("JOIN #{twitch_name}").into()) {
                send_to_error_log(
                    join_channel_error.to_string(),
                    "Error while sending JOIN message".into(),
                );

                let next_retry = retry + 1;
                return connect(twitch_name, oauth_token, next_retry);
            }

            if let Err(capabilities_error) =
                socket.send("CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".into())
            {
                send_to_error_log(
                    capabilities_error.to_string(),
                    "Error while requesting capabilities from Twitch".into(),
                );

                let next_retry = retry + 1;
                return connect(twitch_name, oauth_token, next_retry);
            }

            socket
        }

        Err(conn_error) => {
            send_to_error_log(conn_error.to_string(), "Error during connection to socket".into());

            let next_retry = retry + 1;
            connect(twitch_name, oauth_token, next_retry)
        }
    }
}

impl TwitchIRC {
    pub fn new(twitch_name: Arc<String>, oauth_token: Arc<String>, tx: Sender<ChannelMessages>) -> Self {
        let socket = connect(&twitch_name, &oauth_token, 0);

        TwitchIRC {
            socket,
            nickname: twitch_name.to_string(),
            oauth_token: oauth_token.to_string(),
            tx,
        }
    }

    pub fn send_privmsg(&mut self, message: &str) {
        let message = format!("PRIVMSG #{} :{message}", self.nickname);
        let _ = self.socket.send(message.into());
    }

    pub fn listen(&mut self) {
        loop {
            if let Ok(message) = self.socket.read() {
                // NOTE: Twitch could send multiple messages at once, so we need to split them
                // The messages are separated by '\r\n'
                let messages = message.to_text().unwrap().split(MESSAGE_DELIMITER);
                let messages = messages.map(tungstenite::Message::from);

                messages.for_each(|message| match message {
                    tungstenite::Message::Text(new_message) => match parse(&new_message) {
                        Ok(
                            message @ TwitchMessage::RedeemMessage { .. }
                            | message @ TwitchMessage::PrivMessage { .. }
                            | message @ TwitchMessage::RaidMessage { .. },
                        ) => {
                            let _ = self.tx.send(ChannelMessages::TwitchMessage(message));
                            // output(message, self);
                        }

                        Ok(TwitchMessage::PingMessage { message }) => {
                            let pong_message = format!("PONG {message}");

                            let _ = self.socket.send(pong_message.into());
                        }

                        Ok(TwitchMessage::UnknownMessage { message }) => {
                            send_to_error_log(message, "Unknown message".to_string())
                        }

                        Err(error) => send_to_error_log(
                            error.to_string(),
                            format!("Error while parsing message: {}", new_message),
                        ),
                    },

                    tungstenite::Message::Close(_) => {
                        send_to_error_log("Connection closed".to_string(), "Connection closed".to_string());

                        let name: Arc<String> = Arc::new(self.nickname.clone());
                        let token: Arc<String> = Arc::new(self.oauth_token.clone());

                        connect(&name, &token, 0);
                    }

                    unknown => send_to_error_log(unknown.to_string(), "Unhandled error from socket.read()".to_string()),
                });
            }
        }
    }
}
