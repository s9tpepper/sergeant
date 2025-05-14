use std::{
    process::{self, Command},
    sync::{mpsc::Sender, Arc},
};

use log::{error, info};

use crate::{
    channel::ChannelMessages,
    chat_commands::get_reward,
    twitch::{
        api::{get_user_by_login, User},
        eventsub::{
            deserialization::{NotificationEvent, NotificationPayload},
            notifications::prelude::send_to_channels,
        },
        redeems::{get_reward_fulfilled_message, refund_points, reward_fulfilled},
    },
};

pub fn channel_points_custom_reward_redemption_add(
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
    oauth_token: &Arc<String>,
    client_id: &Arc<String>,
) -> anyhow::Result<()> {
    info!("----- channel_points_custom_reward_redemption_add()");

    let NotificationEvent::ChannelPointsCustomRewardRedemptionAdd {
        id,
        broadcaster_user_id,
        user_name,
        reward,
        user_input,
        status,
        user_login,
        ..
    } = &*payload.event
    else {
        error!("Error trying to destructure NotificationEvent::ChannelPointsCustomRewardRedemptionAdd");
        return Ok(());
    };

    let Ok(cmd_mapping) = get_reward(&reward.title) else {
        error!("Error trying to get_reward()");
        return Ok(());
    };

    // Notify websocket about reward redeem
    info!("[Reward] {} fulfilled", reward.title);
    let message = get_reward_fulfilled_message(reward, user_name, user_input);

    let User {
        profile_image_url: profile_url,
        ..
    } = get_user_by_login(user_login, oauth_token, client_id).unwrap_or(User {
        id: "".to_string(),
        login: "".to_string(),
        display_name: "Unknown".to_string(),
        r#type: "".to_string(),
        broadcaster_type: "".to_string(),
        description: "".to_string(),
        profile_image_url: "".to_string(),
        offline_image_url: "".to_string(),
        created_at: "".to_string(),
    });

    let channel_message = ChannelMessages::RedeemMessage {
        message,
        profile_url,
        title: reward.title.to_string(),
        display_name: user_name.to_string(),
    };
    let _ = send_to_channels(channel_message, tui_tx, websocket_tx, "reward fulfilled");

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

    match command_result {
        Ok(command_result) => match command_result.status.success() {
            true => {
                // NOTE: values: unknown, unfulfilled, fulfilled, and canceled.
                let reward_status = status.to_lowercase();
                if reward_status == "unfulfilled" {
                    return reward_fulfilled(id, reward, broadcaster_user_id, oauth_token, client_id);
                }

                Ok(())
            }

            false => refund_points(
                id,
                user_name,
                broadcaster_user_id,
                reward,
                tui_tx,
                websocket_tx,
                oauth_token,
                client_id,
                command_result,
            ),
        },

        Err(ref command_error) => {
            error!("Error running reward command: {command_error}, command: {command:?}");

            if status.to_lowercase() == "unfulfilled" {
                return refund_points(
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

            Ok(())
        }
    }
}
