use std::{net::TcpStream, sync::Arc};

use tungstenite::{stream::MaybeTlsStream, WebSocket};

pub struct TwitchIRC {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    nickname: String,
}

impl TwitchIRC {
    pub fn new(twitch_name: Arc<String>, oauth_token: Arc<String>) -> Self {
        let socket = match tungstenite::connect("wss://irc-ws.chat.twitch.tv:443") {
            Ok((mut socket, _)) => {
                if let Err(connection_error) = socket.send(format!("PASS {oauth_token}").into()) {
                    // TODO: fix the panic
                    panic!("{connection_error}") // at the disco!
                }

                if let Err(nickname_error) = socket.send(format!("NICK {twitch_name}").into()) {
                    // TODO: fix the panic
                    panic!("{nickname_error}") // at the disco!
                }

                if let Err(join_channel_error) = socket.send(format!("JOIN #{twitch_name}").into()) {
                    // TODO: fix the panic
                    panic!("{join_channel_error}") // at the disco!
                }

                // ask for capabilities
                let _ = socket.send("CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".into());

                socket
            }

            Err(conn_error) => {
                // TODO: fix the panic
                panic!("{conn_error}");
            }
        };

        TwitchIRC {
            socket,
            nickname: twitch_name.to_string(),
        }
    }

    pub fn send_privmsg(&mut self, message: &str) {
        let message = format!("PRIVMSG #{} :{message}", self.nickname);
        let _ = self.socket.send(message.into());
    }

    pub fn listen(&mut self) {
        loop {
            if let Ok(message) = self.socket.read() {
                match message {
                    tungstenite::Message::Text(new_message) => {
                        // TODO: Start parsing messages and apply colors/emotes/etc
                        println!("{}", new_message);
                    }

                    // TODO: Reconnect?
                    // tungstenite::Message::Close(_) => todo!(),
                    _ => {
                        // TODO: fix the panic
                        panic!("YE YE YE");
                    }
                }
            }
        }
    }
}
