use std::sync::{mpsc::Sender, Arc};

use crate::channel::ChannelMessages;

use super::deserialization::{NotificationMetadata, NotificationPayload, SubscriptionType};

pub mod prelude;

use log::info;
use prelude::*;

pub fn handle_notification(
    _metadata: &NotificationMetadata,
    payload: &NotificationPayload,
    tui_tx: &Sender<ChannelMessages>,
    websocket_tx: &Sender<ChannelMessages>,
    oauth_token: &Arc<String>,
    client_id: &Arc<String>,
) {
    info!("handle_notification()");
    info!("payload: {payload:?}");

    #[allow(clippy::single_match)]
    match payload.subscription.r#type {
        // SubscriptionType::AutomodMessageHold => todo!(),
        // SubscriptionType::AutomodMessageHoldV2 => todo!(),
        // SubscriptionType::AutomodMessageUpdate => todo!(),
        // SubscriptionType::AutomodMessageUpdateV2 => todo!(),
        // SubscriptionType::AutomodSettingsUpdate => todo!(),
        // SubscriptionType::AutomodTermsUpdate => todo!(),
        // SubscriptionType::ChannelUpdate => todo!(),
        // SubscriptionType::ChannelFollow => todo!(),
        SubscriptionType::ChannelAdBreakBegin => channel_ad_break_begin(payload, tui_tx, websocket_tx),
        // SubscriptionType::ChannelChatClear => todo!(),
        SubscriptionType::ChannelChatClearUserMessages => {
            channel_chat_clear_user_messages(payload, tui_tx, websocket_tx)
        }
        SubscriptionType::ChannelChatMessage => {
            channel_chat_message(payload, tui_tx, websocket_tx);
        }

        // SubscriptionType::ChannelChatMessageDelete => todo!(),
        SubscriptionType::ChannelChatNotification => channel_chat_notification(payload, tui_tx, websocket_tx),
        // SubscriptionType::ChannelChatSettingsUpdate => todo!(),
        // SubscriptionType::ChannelChatUserMessageHold => todo!(),
        // SubscriptionType::ChannelChatUserMessageUpdate => todo!(),
        // SubscriptionType::ChannelSharedChatSessionBegin => todo!(),
        // SubscriptionType::ChannelSharedChatSessionUpdate => todo!(),
        // SubscriptionType::ChannelSharedChatSessionEnd => todo!(),
        // SubscriptionType::ChannelSubscribe => todo!(),
        // SubscriptionType::ChannelSubscriptionEnd => todo!(),
        // SubscriptionType::ChannelSubscriptionGift => todo!(),
        // SubscriptionType::ChannelSubscriptionMessage => todo!(),
        // SubscriptionType::ChannelCheer => todo!(),
        // SubscriptionType::ChannelRaid => todo!(),
        // SubscriptionType::ChannelBan => todo!(),
        // SubscriptionType::ChannelUnban => todo!(),
        // SubscriptionType::ChannelUnbanRequestCreate => todo!(),
        // SubscriptionType::ChannelUnbanRequestResolve => todo!(),
        // SubscriptionType::ChannelModerate => todo!(),
        // SubscriptionType::ChannelModerateV2 => todo!(),
        // SubscriptionType::ChannelModeratorAdd => todo!(),
        // SubscriptionType::ChannelModeratorRemove => todo!(),
        // SubscriptionType::ChannelGuestStarSessionBegin => todo!(),
        // SubscriptionType::ChannelGuestStarSessionEnd => todo!(),
        // SubscriptionType::ChannelGuestStarGuestUpdate => todo!(),
        // SubscriptionType::ChannelGuestStarSettingsUpdate => todo!(),

        // A viewer has redeemed an automatic channel points reward on the specified channel.
        SubscriptionType::ChannelPointsAutomaticRewardRedemption => {
            channel_points_automatic_reward_redemption(payload, tui_tx, websocket_tx)
        }
        // SubscriptionType::ChannelPointsCustomRewardAdd => todo!(),
        // SubscriptionType::ChannelPointsCustomRewardUpdate => todo!(),
        // SubscriptionType::ChannelPointsCustomRewardRemove => todo!(),
        SubscriptionType::ChannelPointsCustomRewardRedemptionAdd => {
            channel_points_custom_reward_redemption_add(payload, tui_tx, websocket_tx, oauth_token, client_id)
        }

        // Event for channel redeems
        SubscriptionType::ChannelPointsCustomRewardRedemptionUpdate => {
            info!(">>>>ChannelPointsCustomRewardRedemptionUpdate()");
            info!(">>>>{payload:?}");
            channel_points_custom_reward_redemption_update(payload, tui_tx, websocket_tx, oauth_token, client_id)
        }
        // SubscriptionType::ChannelPollBegin => todo!(),
        // SubscriptionType::ChannelPollProgress => todo!(),
        // SubscriptionType::ChannelPollEnd => todo!(),
        // SubscriptionType::ChannelPredictionBegin => todo!(),
        // SubscriptionType::ChannelPredictionProgress => todo!(),
        // SubscriptionType::ChannelPredictionLock => todo!(),
        // SubscriptionType::ChannelPredictionEnd => todo!(),
        // SubscriptionType::ChannelSuspiciousUserMessage => todo!(),
        // SubscriptionType::ChannelSuspiciousUserUpdate => todo!(),
        // SubscriptionType::ChannelVIPAdd => todo!(),
        // SubscriptionType::ChannelVIPRemove => todo!(),
        // SubscriptionType::ChannelWarningAcknowledgement => todo!(),
        // SubscriptionType::ChannelWarningSend => todo!(),
        // SubscriptionType::CharityDonation => todo!(),
        // SubscriptionType::CharityCampaignStart => todo!(),
        // SubscriptionType::CharityCampaignProgress => todo!(),
        // SubscriptionType::CharityCampaignStop => todo!(),
        // SubscriptionType::ConduitShardDisabled => todo!(),
        // SubscriptionType::DropEntitlementGrant => todo!(),
        // SubscriptionType::ExtensionBitsTransactionCreate => todo!(),
        // SubscriptionType::GoalBegin => todo!(),
        // SubscriptionType::GoalProgress => todo!(),
        // SubscriptionType::GoalEnd => todo!(),
        // SubscriptionType::HypeTrainBegin => todo!(),
        // SubscriptionType::HypeTrainProgress => todo!(),
        // SubscriptionType::HypeTrainEnd => todo!(),
        // SubscriptionType::ShieldModeBegin => todo!(),
        // SubscriptionType::ShieldModeEnd => todo!(),
        // SubscriptionType::ShoutoutCreate => todo!(),
        // SubscriptionType::ShoutoutReceived => todo!(),
        // SubscriptionType::StreamOnline => todo!(),
        // SubscriptionType::StreamOffline => todo!(),
        // SubscriptionType::UserAuthorizationGrant => todo!(),
        // SubscriptionType::UserAuthorizationRevoke => todo!(),
        // SubscriptionType::UserUpdate => todo!(),
        // SubscriptionType::WhisperReceived => todo!(),
        // SubscriptionType::Unknown => todo!(),
        _ => {}
    }
}
