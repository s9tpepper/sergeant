use clap::{Command, arg};

pub fn get_command() -> Command {
    Command::new("ferris_twitch")
        .about("Yet another twitch bot/client, but Rust...")
        .subcommand_required(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("chat")
                .about("Starts Twitch chat client")
                .arg(
                    arg!(<NAME> "Your Twitch username")
                        .env("NAME")
                        .required(true)
                )
                .arg(
                    arg!(<TOKEN> "Your Twitch OAuth token")
                        .env("TOKEN")
                        .required(true)
                )
                .arg(
                    arg!(<CLIENT_ID> "Your Twitch app client ID")
                        .env("CLIENT_ID")
                        .required(true)
                )
        )   
}
