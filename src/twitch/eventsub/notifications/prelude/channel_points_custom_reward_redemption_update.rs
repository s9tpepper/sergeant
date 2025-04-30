use std::{
    process::{self, Command},
    sync::{mpsc::Sender, Arc},
};

use log::{error, info};

use crate::{
    channel::ChannelMessages,
    chat_commands::get_reward,
    twitch::{
        eventsub::deserialization::{NotificationEvent, NotificationPayload},
        redeems::{refund_points, reward_fulfilled},
    },
};

pub fn channel_points_custom_reward_redemption_update(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
    oauth_token: &Arc<String>,
    client_id: &Arc<String>,
) {
    info!("channel_points_custom_reward_redemption_add()");

    let NotificationEvent::ChannelPointsCustomRewardRedemptionAdd {
        id,
        broadcaster_user_id,
        user_name,
        reward,
        user_input,
        status,
        ..
    } = &*payload.event
    else {
        error!("Error trying to destructure NotificationEvent::ChannelPointsCustomRewardRedemptionAdd");
        return;
    };

    let Ok(cmd_mapping) = get_reward(&reward.title) else {
        error!("Error trying to get_reward()");
        return;
    };

    let (command_name, sub_command) = cmd_mapping.split_once(':').unwrap_or((&cmd_mapping, ""));

    let mut command = Command::new(command_name);
    if !sub_command.is_empty() {
        command.arg(sub_command);
        info!("Command with subcommand: {command:?}");
    }

    command.arg(user_name);

    if !user_input.is_empty() {
        command.arg(user_input);

        info!("Command with subcommand and user input: {command:?}");
    }

    let command_result = command
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .output();

    info!("[REWARD] - Command result: {command_result:?}");

    match command_result {
        Ok(command_result) => match command_result.status.success() {
            true => {
                info!("[REWARD] - Command status: {}", command_result.status);

                // NOTE: values: unknown, unfulfilled, fulfilled, and canceled.
                let reward_status = status.to_lowercase();
                if reward_status == "unfulfilled" {
                    reward_fulfilled(
                        id,
                        reward,
                        user_name,
                        user_input,
                        broadcaster_user_id,
                        oauth_token,
                        client_id,
                        tui_tx,
                        websocket_tx,
                    );
                }
            }

            false => {
                refund_points(
                    id,
                    user_name,
                    broadcaster_user_id,
                    reward,
                    tui_tx,
                    websocket_tx,
                    oauth_token,
                    client_id,
                    command_result,
                );
            }
        },

        Err(ref command_error) => {
            error!("Error running reward command: {command_error}, command: {command:?}");

            if status.to_lowercase() == "unfulfilled" {
                refund_points(
                    id,
                    user_name,
                    broadcaster_user_id,
                    reward,
                    tui_tx,
                    websocket_tx,
                    oauth_token,
                    client_id,
                    command_result.expect("Command results should be available"),
                );
            }
        }
    }
}
