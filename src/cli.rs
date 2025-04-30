use std::fs::File;

use clap::{Parser, Subcommand};
use commands::start_chat;

use log::{info, LevelFilter};
use simplelog::{Config, WriteLogger};

use crate::{
    chat_commands::{
        add_action, add_command, add_reward, list_actions, list_commands, list_rewards, remove_action, remove_command,
        remove_reward,
    },
    twitch::auth::authenticate_with_twitch,
};

mod commands;

fn logger() {
    // TODO: Move this log file into the application directory
    // TODO: Enable this block with an env var
    let _ = WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("sergeant.log").unwrap(),
    );

    info!("Logging has been enabled");
}

pub fn command() -> anyhow::Result<()> {
    logger();

    let cli = Cli::parse();
    match cli.commands {
        // TODO: This needs to be moved in from v1
        // Cmds::Admin => start_admin(),
        Cmds::Chat {
            twitch_name,
            oauth_token,
            client_id,
            skip_announcements,
            test_mode,
        } => start_chat(
            twitch_name.as_deref(),
            oauth_token.as_deref(),
            client_id.as_deref(),
            skip_announcements,
            test_mode,
        ),

        Cmds::Commands { cmd } => match cmd {
            SubCmds::List => list_commands(),
            SubCmds::Add { name, message, timing } => add_command(&name, &message, timing),
            SubCmds::Remove { name } => remove_command(&name),
        },

        Cmds::IrcActions { cmd } => match cmd {
            IrcActionSubCmds::List => list_actions(),
            IrcActionSubCmds::Add { name, cli } => add_action(&name, &cli),
            IrcActionSubCmds::Remove { name } => remove_action(&name),
        },

        Cmds::Rewards { cmd } => match cmd {
            RewardSubCmds::List => list_rewards(),
            RewardSubCmds::Add { name, cli } => add_reward(&name, &cli),
            RewardSubCmds::Remove { name } => remove_reward(&name),
        },

        Cmds::Login => authenticate_with_twitch(),

        // NOTE: Remove this catchall when done with the previous commands
        _ => anyhow::Ok(()),
    }
}

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    commands: Cmds,
}

#[derive(Subcommand)]
enum Cmds {
    /// Open the admin dashboard
    Admin,

    /// Start Twitch Chat client
    Chat {
        /// Your Twitch username
        #[arg(long, short = 'n', env = "TWITCH_NAME")]
        twitch_name: Option<String>,

        /// Your Twitch OAuth Token
        #[arg(long, short = 't', env = "OAUTH_TOKEN")]
        oauth_token: Option<String>,

        /// Your Twitch app client ID
        #[arg(long, short, env = "CLIENT_ID")]
        client_id: Option<String>,

        /// Set to turn off announcements
        #[arg(long, short = 's', env = "SKIP_ANNOUNCEMENTS", default_value_t = false)]
        skip_announcements: bool,

        /// Run the chat TUI in testing mode with testing data
        #[arg(long, short = 'm', env = "TEST_MODE", default_value_t = false)]
        test_mode: bool,
    },

    /// Manage chat commands
    Commands {
        #[command(subcommand)]
        cmd: SubCmds,
    },

    /// Manage chat rewards
    Rewards {
        #[command(subcommand)]
        cmd: RewardSubCmds,
    },

    /// Manage IRC actions
    IrcActions {
        #[command(subcommand)]
        cmd: IrcActionSubCmds,
    },

    /// Login to Twitch and get a token
    Login,
}

#[derive(Subcommand)]
enum SubCmds {
    /// List commands
    List,

    /// Add a chat command
    Add {
        /// The name of the command
        name: String,

        /// The message to send for the command
        message: String,

        /// The timing for the message in minutes
        timing: Option<usize>,
    },

    /// Remove a command
    Remove {
        /// The name of the command to remove
        name: String,
    },
}

#[derive(Subcommand)]
enum RewardSubCmds {
    /// List rewards
    List,

    /// Add a reward command
    Add {
        /// The name of the reward as it is named on Twitch
        name: String,

        /// The cli command to execute for the reward
        cli: String,
    },

    /// Remove a command
    Remove {
        /// The name of the reward to remove
        name: String,
    },
}

#[derive(Subcommand)]
enum IrcActionSubCmds {
    /// List IRC Actions
    List,

    /// Add an IRC ation command
    Add {
        /// The name of the IRC message type as it is named in the IRC protocol
        name: String,

        /// The cli command to execute for as the IRC action
        cli: String,
    },

    /// Remove an IRC action
    Remove {
        /// The name of the IRC message type to remove
        name: String,
    },
}
