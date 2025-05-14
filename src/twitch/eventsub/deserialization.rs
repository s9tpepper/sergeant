#![allow(unused)]

use anathema::state::AnyState;
use log::info;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Messages {
    Notification {
        metadata: NotificationMetadata,
        payload: NotificationPayload,
    },

    Welcome {
        metadata: WelcomeMetadata,
        payload: WelcomePayload,
    },

    Reconnect {
        metadata: ReconnectMetadata,
        payload: ReconnectPayload,
    },

    Revocation {
        metadata: RevocationMetadata,
        payload: RevocationPayload,
    },

    KeepAlive {
        metadata: KeepAliveMetadata,
        payload: KeepAlivePayload,
    },
}

#[derive(Deserialize, Debug)]
pub struct RevocationMetadata {
    message_id: String,
    message_type: MessageTypes,
    message_timestamp: String,
    subscription_type: SubscriptionType,
    subscription_version: String,
}

#[derive(Deserialize, Debug)]
pub struct RevocationPayload {
    id: String,
    status: String,
    r#type: SubscriptionType,
    version: String,
    cost: usize,
    condition: Condition,
    transport: Transport,
    created_at: String,
}

#[derive(Deserialize, Debug)]
pub struct ReconnectMetadata {
    message_id: String,
    message_type: MessageTypes,
    message_timestamp: String,
}

#[derive(Deserialize, Debug)]
pub struct ReconnectPayload {
    session: ReconnectPayloadSession,
}

#[derive(Deserialize, Debug)]
pub struct ReconnectPayloadSession {
    id: String,
    status: String,
    keepalive_timeout_seconds: usize,
    reconnect_url: Option<String>,
    recovery_url: Option<String>,
    connected_at: String,
}

#[derive(Deserialize, Debug)]
pub struct NotificationMetadata {
    message_id: String,
    message_type: MessageTypes,
    message_timestamp: String,
    subscription_type: SubscriptionType,
    subscription_version: String,
}

#[derive(Deserialize, Debug)]
pub struct NotificationPayload {
    pub subscription: NotificationPayloadSubscription,
    pub event: Box<NotificationEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NotificationEvent {
    ChannelChatMessage {
        broadcaster_user_id: String,
        broadcaster_user_name: String,
        broadcaster_user_login: String,
        chatter_user_id: String,
        chatter_user_name: String,
        chatter_user_login: String,
        message_id: String,
        message: Message,
        color: String,
        message_type: ChatMessageTypes,
        badges: Vec<Badge>,
        cheer: Option<Cheer>,
        reply: Option<Reply>,
        channel_points_animation_id: Option<String>, // Undocumented at https://dev.twitch.tv/docs/eventsub/eventsub-reference/#channel-chat-message-event
        channel_points_custom_reward_id: Option<String>,
        source_broadcaster_user_id: Option<String>,
        source_broadcaster_user_name: Option<String>,
        source_broadcaster_user_login: Option<String>,
        source_message_id: Option<String>,
        source_badges: Option<Vec<Badge>>,
    },

    ChannelAdBreak {
        duration_seconds: usize,
        started_at: String,
        is_automatic: bool,
        broadcaster_user_id: String,
        broadcaster_user_login: String,
        broadcaster_user_name: String,
        requester_user_id: String,
        requester_user_login: String,
        requester_user_name: String,
    },

    ChannelNotification {
        broadcaster_user_id: String,
        broadcaster_user_login: String,
        broadcaster_user_name: String,
        chatter_user_id: String,
        chatter_user_login: String,
        chatter_user_name: String,
        chatter_is_anonymous: bool,
        color: String,
        badges: Vec<Badge>,
        system_message: String,
        message_id: String,
        message: Box<Message>,
        notice_type: Box<NoticeType>,
        sub: Box<Option<Sub>>,
        resub: Box<Option<Resub>>,
        sub_gift: Box<Option<SubGift>>,
        community_sub_gift: Box<Option<CommunitySubGift>>,
        gift_paid_upgrade: Box<Option<GiftPaidUpgrade>>,
        prime_paid_upgrade: Box<Option<PrimePaidUpgrade>>,
        pay_it_forward: Box<Option<PayItForward>>,
        raid: Box<Option<Raid>>,
        unraid: Box<Option<Unraid>>,
        announcement: Box<Option<Announcement>>,
        bits_badge_tier: Box<Option<BitsBadgeTier>>,
        charity_donation: Box<Option<CharityDonation>>,
        source_broadcaster_user_id: Box<Option<String>>,
        source_broadcaster_user_name: Box<Option<String>>,
        source_broadcaster_user_login: Box<Option<String>>,
        source_message_id: Box<Option<String>>,
        source_badges: Box<Option<Vec<Emote>>>,
        shared_chat_sub: Box<Option<Sub>>,
        shared_chat_resub: Box<Option<Resub>>,
        shared_chat_sub_gift: Box<Option<SubGift>>,
        shared_chat_community_sub_gift: Box<Option<CommunitySubGift>>,
        shared_chat_gift_paid_upgrade: Box<Option<GiftPaidUpgrade>>,
        shared_chat_prime_paid_upgrade: Box<Option<PrimePaidUpgrade>>,
        shared_chat_pay_it_forward: Box<Option<PayItForward>>,
        shared_chat_raid: Box<Option<Raid>>,
        shared_chat_announcement: Box<Option<Announcement>>,
    },

    ChannelPointsCustomRewardRedemptionAdd {
        id: String,
        broadcaster_user_id: String,
        broadcaster_user_login: String,
        broadcaster_user_name: String,
        user_id: String,
        user_login: String,
        user_name: String,
        user_input: String,
        status: String,
        reward: Reward,
        redeemed_at: String,
    },

    ChannelChatClearUserMessages {
        broadcaster_user_id: String,
        broadcaster_user_name: String,
        broadcaster_user_login: String,
        target_user_id: String,
        target_user_name: String,
        target_user_login: String,
    },

    AutomodMessageHold {
        temp: String,
    },

    ChannelPointAutomaticRewardRedemption {
        broadcaster_user_id: String,
        broadcaster_user_login: String,
        broadcaster_user_name: String,
        user_id: String,
        user_login: String,
        user_name: String,
        id: String,
        reward: Reward,
        message: Message,
        redeemed_at: String,
    },
}

#[derive(Clone, Debug)]
pub enum ChatMessageTypes {
    Text,
    ChannelPointsHighlighted,
    ChannelPointsSubOnly,
    UserIntro,
    PowerUpsMessageEffect,
    PowerUpsGigantifiedEmote,
    Unknown,
}

impl Serialize for ChatMessageTypes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ChatMessageTypes::Text => serializer.serialize_str("text"),
            ChatMessageTypes::ChannelPointsHighlighted => serializer.serialize_str("channel_points_highlighted"),
            ChatMessageTypes::ChannelPointsSubOnly => serializer.serialize_str("channel_points_sub_only"),
            ChatMessageTypes::UserIntro => serializer.serialize_str("user_intro"),
            ChatMessageTypes::PowerUpsMessageEffect => serializer.serialize_str("power_ups_message_effect"),
            ChatMessageTypes::PowerUpsGigantifiedEmote => serializer.serialize_str("power_ups_gigantified_emote"),
            ChatMessageTypes::Unknown => serializer.serialize_str("unknown_type"),
        }
    }
}

impl<'de> Deserialize<'de> for ChatMessageTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "text" => Ok(ChatMessageTypes::Text),
            "channel_points_highlighted" => Ok(ChatMessageTypes::ChannelPointsHighlighted),
            "channel_points_sub_only" => Ok(ChatMessageTypes::ChannelPointsSubOnly),
            "user_intro" => Ok(ChatMessageTypes::UserIntro),
            "power_ups_message_effect" => Ok(ChatMessageTypes::PowerUpsMessageEffect),
            "power_ups_gigantified_emote" => Ok(ChatMessageTypes::PowerUpsGigantifiedEmote),
            _ => Ok(ChatMessageTypes::Unknown),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cheer {
    bits: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Reply {
    parent_message_id: String,
    parent_message_body: String,
    parent_user_id: String,
    parent_user_name: String,
    parent_user_login: String,
    thread_message_id: String,
    thread_user_id: String,
    thread_user_name: String,
    thread_user_login: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CharityDonation {
    pub charity_name: String,
    pub amount: Amount,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Amount {
    pub value: u32,
    pub decimal_place: u8,
    pub currency: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BitsBadgeTier {
    pub tier: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Announcement {
    color: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Unraid {}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct Raid {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
    pub viewer_count: u64,
    pub profile_image_url: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct PayItForward {
    pub gifter_is_anonymous: bool,
    pub gifter_user_id: Option<String>,
    pub gifter_user_name: Option<String>,
    pub gifter_user_login: Option<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct PrimePaidUpgrade {
    pub sub_tier: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct GiftPaidUpgrade {
    pub gifter_is_anonymous: bool,
    pub gifter_user_id: Option<String>,
    pub gifter_user_name: Option<String>,
    pub gifter_user_login: Option<String>,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct CommunitySubGift {
    pub id: String,
    pub total: u16,
    pub sub_tier: String, // 1000, 2000, 3000
    pub cumulative_total: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubGift {
    duration_months: u8,
    cumulative_total: u8,
    recipient_user_id: String,
    recipient_user_name: String,
    recipient_user_login: String,
    sub_tier: String, // 1000, 2000, 3000
    community_gift_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Resub {
    cumulative_months: u8,
    duration_months: u8,
    streak_months: u8,
    sub_tier: String, // 1000, 2000, 3000
    is_prime: Option<bool>,
    is_gift: bool,
    gifter_is_anonymous: Option<bool>,
    gifter_user_id: Option<String>,
    gifter_user_name: Option<String>,
    gifter_user_login: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sub {
    sub_tier: String, // 1000, 2000, 3000
    is_prime: bool,
    duration_months: u16,
}

#[derive(Debug, Clone)]
pub enum NoticeType {
    Sub,
    Resub,
    SubGift,
    CommunitySubGift,
    GiftPaidUpgrade,
    PrimePaidUpgrade,
    Raid,
    Unraid,
    PayItForward,
    Announcement,
    BitsBadgeTier,
    CharityDonation,
    SharedChatSub,
    SharedChatResub,
    SharedChatSubGift,
    SharedChatCommunitySubGift,
    SharedChatGiftPaidUpgrade,
    SharedChatPrimePaidUpgrade,
    SharedChatRaid,
    SharedChatPayItForward,
    SharedChatAnnouncement,
    Unknown,
}

impl Serialize for NoticeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            NoticeType::Sub => serializer.serialize_str("sub"),
            NoticeType::Resub => serializer.serialize_str("resub"),
            NoticeType::SubGift => serializer.serialize_str("sub_gift"),
            NoticeType::CommunitySubGift => serializer.serialize_str("community_sub_gift"),
            NoticeType::GiftPaidUpgrade => serializer.serialize_str("gift_paid_upgrade"),
            NoticeType::PrimePaidUpgrade => serializer.serialize_str("prime_paid_upgrade"),
            NoticeType::Raid => serializer.serialize_str("raid"),
            NoticeType::Unraid => serializer.serialize_str("unraid"),
            NoticeType::PayItForward => serializer.serialize_str("pay_it_forward"),
            NoticeType::Announcement => serializer.serialize_str("announcement"),
            NoticeType::BitsBadgeTier => serializer.serialize_str("bits_badge_tier"),
            NoticeType::CharityDonation => serializer.serialize_str("charity_donation"),
            NoticeType::SharedChatSub => serializer.serialize_str("shared_chat_sub"),
            NoticeType::SharedChatResub => serializer.serialize_str("shared_chat_resub"),
            NoticeType::SharedChatSubGift => serializer.serialize_str("shared_chat_sub_gift"),
            NoticeType::SharedChatCommunitySubGift => serializer.serialize_str("shared_chat_community_gift"),
            NoticeType::SharedChatGiftPaidUpgrade => serializer.serialize_str("shared_chat_gift_paid_upgrade"),
            NoticeType::SharedChatPrimePaidUpgrade => serializer.serialize_str("shared_prime_paid_upgrade"),
            NoticeType::SharedChatRaid => serializer.serialize_str("shared_chat_raid"),
            NoticeType::SharedChatPayItForward => serializer.serialize_str("shared_chat_pay_it_forward"),
            NoticeType::SharedChatAnnouncement => serializer.serialize_str("shared_chat_announcement"),
            NoticeType::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

impl<'de> Deserialize<'de> for NoticeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "sub" => Ok(NoticeType::Sub),
            "resub" => Ok(NoticeType::Resub),
            "sub_gift" => Ok(NoticeType::SubGift),
            "community_sub_gift" => Ok(NoticeType::CommunitySubGift),
            "gift_paid_upgrade" => Ok(NoticeType::GiftPaidUpgrade),
            "prime_paid_upgrade" => Ok(NoticeType::PrimePaidUpgrade),
            "raid" => Ok(NoticeType::Raid),
            "unraid" => Ok(NoticeType::Unraid),
            "pay_it_forward" => Ok(NoticeType::PayItForward),
            "announcement" => Ok(NoticeType::Announcement),
            "bits_badge_tier" => Ok(NoticeType::BitsBadgeTier),
            "charity_donation" => Ok(NoticeType::CharityDonation),
            "shared_chat_sub" => Ok(NoticeType::SharedChatSub),
            "shared_chat_resub" => Ok(NoticeType::SharedChatResub),
            "shared_chat_sub_gift" => Ok(NoticeType::SharedChatSubGift),
            "shared_chat_community_sub_gift" => Ok(NoticeType::SharedChatCommunitySubGift),
            "shared_chat_gift_paid_upgrade" => Ok(NoticeType::SharedChatGiftPaidUpgrade),
            "shared_chat_prime_paid_upgrade" => Ok(NoticeType::SharedChatPrimePaidUpgrade),
            "shared_chat_raid" => Ok(NoticeType::SharedChatRaid),
            "shared_chat_pay_it_forward" => Ok(NoticeType::SharedChatPayItForward),
            "shared_chat_announcement" => Ok(NoticeType::SharedChatAnnouncement),
            _ => Ok(NoticeType::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub text: String,
    pub fragments: Vec<Fragment>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fragment {
    pub r#type: FragmentType,
    pub text: String,
    pub cheermote: Option<Cheermote>,
    pub emote: Option<Emote>,
    pub mention: Option<Mention>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mention {
    user_id: String,
    pub user_name: String,
    user_login: String,
}

// https://dev.twitch.tv/docs/eventsub/eventsub-reference/#channel-chat-message-event
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Emote {
    pub id: String,
    pub name: Option<String>,
    pub emote_set_id: String,
    pub owner_id: String,
    pub mention: Option<Mention>,
    pub format: Vec<String>, // animated | static
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cheermote {
    prefix: String,
    bits: u16,
    tier: u8,
}

#[derive(Debug, Clone)]
pub enum FragmentType {
    Text,
    Cheermote,
    Emote,
    Mention,
    Unknown,
}

impl Serialize for FragmentType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FragmentType::Text => serializer.serialize_str("text"),
            FragmentType::Cheermote => serializer.serialize_str("cheermote"),
            FragmentType::Emote => serializer.serialize_str("emote"),
            FragmentType::Mention => serializer.serialize_str("mention"),
            FragmentType::Unknown => serializer.serialize_str("unknown_type"),
        }
    }
}

impl<'de> Deserialize<'de> for FragmentType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "text" => Ok(FragmentType::Text),
            "cheermote" => Ok(FragmentType::Cheermote),
            "emote" => Ok(FragmentType::Emote),
            "mention" => Ok(FragmentType::Mention),
            _ => Ok(FragmentType::Unknown),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Badge {
    pub set_id: String,
    pub id: String,
    pub info: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reward {
    pub id: String,
    pub title: String,
    pub cost: usize,
    pub prompt: String,

    // NOTE: Type is one of:
    // single_message_bypass_sub_mode
    // send_highlighted_message
    // random_sub_emote_unlock
    // chosen_sub_emote_unlock
    // chosen_modified_sub_emote_unlock
    pub r#type: Option<String>,

    pub channel_points: Option<i32>,
    pub emote: Option<Emote>,
}

#[derive(Serialize, Debug)]
pub struct Subscription<'a> {
    pub r#type: &'a SubscriptionType,
    pub condition: Condition,
    pub version: String,
    pub transport: Transport,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct NotificationPayloadSubscription {
    pub id: String,
    pub status: String,
    pub r#type: SubscriptionType,
    pub version: String,
    pub cost: usize,
    pub condition: Condition,
    pub transport: Transport,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Condition {
    pub to_broadcaster_user_id: Option<String>,
    pub broadcaster_user_id: Option<String>,
    pub moderator_user_id: Option<String>,
    pub user_id: Option<String>,
    pub reward_id: Option<String>,
    pub client_id: Option<String>,
    pub organization_id: Option<String>,
    pub category_id: Option<String>,
    pub campaign_id: Option<String>,
    pub extension_client_id: Option<String>,
}

#[derive(Debug)]
pub enum MethodType {
    WebSocket,
    WebHook,
    Unknown,
}

impl<'de> Deserialize<'de> for MethodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "websocket" => Ok(MethodType::WebSocket),
            "webhook" => Ok(MethodType::WebHook),
            _ => {
                info!("Unknown MethodType value: {s}");
                Ok(MethodType::Unknown)
            }
        }
    }
}

impl Serialize for MethodType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MethodType::WebSocket => serializer.serialize_str("websocket"),
            MethodType::WebHook => serializer.serialize_str("webhook"),
            MethodType::Unknown => serializer.serialize_str("unknown"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transport {
    pub method: MethodType,
    pub session_id: String,
}

#[derive(Deserialize, Debug)]
pub struct KeepAliveMetadata {
    message_id: String,
    message_type: MessageTypes,
    message_timestamp: String,
}

#[derive(Deserialize, Debug)]
pub struct KeepAlivePayload {}

#[derive(Deserialize, Debug)]
pub struct WelcomeMetadata {
    message_id: String,
    message_type: MessageTypes,
    message_timestamp: String,
}

#[derive(Deserialize, Debug)]
pub struct WelcomePayload {
    pub session: WelcomePayloadSession,
}

#[derive(Deserialize, Debug)]
pub struct WelcomePayloadSession {
    pub id: String,
    pub status: String,
    pub keepalive_timeout_seconds: usize,
    pub connected_at: String,
    pub reconnect_url: Option<String>,
    pub recovery_url: Option<String>,
}

#[derive(Debug)]
enum MessageTypes {
    SessionWelcome,
    SessionKeepalive,
    Notification,
    SessionReconnect,
    Revocation,
    Unknown,
}

impl<'de> Deserialize<'de> for MessageTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "session_welcome" => Ok(MessageTypes::SessionWelcome),
            "session_keepalive" => Ok(MessageTypes::SessionKeepalive),
            "notification" => Ok(MessageTypes::Notification),
            "session_reconnect" => Ok(MessageTypes::SessionReconnect),
            "revocation" => Ok(MessageTypes::Revocation),
            _ => {
                info!("Unknown MessageType value: {s}");

                Ok(MessageTypes::Unknown)
            } // Handle unknown variants gracefully
        }
    }
}

#[derive(Debug, Clone)]
pub enum SubscriptionType {
    AutomodMessageHold,
    AutomodMessageHoldV2,
    AutomodMessageUpdate,
    AutomodMessageUpdateV2,
    AutomodSettingsUpdate,
    AutomodTermsUpdate,
    ChannelUpdate,
    ChannelFollow,
    ChannelAdBreakBegin,
    ChannelChatClear,
    ChannelChatClearUserMessages,
    ChannelChatMessage,
    ChannelChatMessageDelete,
    ChannelChatNotification,
    ChannelChatSettingsUpdate,
    ChannelChatUserMessageHold,
    ChannelChatUserMessageUpdate,
    ChannelSharedChatSessionBegin,
    ChannelSharedChatSessionUpdate,
    ChannelSharedChatSessionEnd,
    ChannelSubscribe,
    ChannelSubscriptionEnd,
    ChannelSubscriptionGift,
    ChannelSubscriptionMessage,
    ChannelCheer,
    ChannelRaid,
    ChannelBan,
    ChannelUnban,
    ChannelUnbanRequestCreate,
    ChannelUnbanRequestResolve,
    ChannelModerate,
    ChannelModerateV2,
    ChannelModeratorAdd,
    ChannelModeratorRemove,
    ChannelGuestStarSessionBegin,
    ChannelGuestStarSessionEnd,
    ChannelGuestStarGuestUpdate,
    ChannelGuestStarSettingsUpdate,
    ChannelPointsAutomaticRewardRedemption,
    ChannelPointsCustomRewardAdd,
    ChannelPointsCustomRewardUpdate,
    ChannelPointsCustomRewardRemove,
    ChannelPointsCustomRewardRedemptionAdd,
    ChannelPointsCustomRewardRedemptionUpdate,
    ChannelPollBegin,
    ChannelPollProgress,
    ChannelPollEnd,
    ChannelPredictionBegin,
    ChannelPredictionProgress,
    ChannelPredictionLock,
    ChannelPredictionEnd,
    ChannelSuspiciousUserMessage,
    ChannelSuspiciousUserUpdate,
    ChannelVIPAdd,
    ChannelVIPRemove,
    ChannelWarningAcknowledgement,
    ChannelWarningSend,
    CharityDonation,
    CharityCampaignStart,
    CharityCampaignProgress,
    CharityCampaignStop,
    ConduitShardDisabled,
    DropEntitlementGrant,
    ExtensionBitsTransactionCreate,
    GoalBegin,
    GoalProgress,
    GoalEnd,
    HypeTrainBegin,
    HypeTrainProgress,
    HypeTrainEnd,
    ShieldModeBegin,
    ShieldModeEnd,
    ShoutoutCreate,
    ShoutoutReceived,
    StreamOnline,
    StreamOffline,
    UserAuthorizationGrant,
    UserAuthorizationRevoke,
    UserUpdate,
    WhisperReceived,
    Unknown,
}

impl Serialize for SubscriptionType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SubscriptionType::AutomodMessageHold => serializer.serialize_str("automod.message.hold"),
            SubscriptionType::AutomodMessageHoldV2 => serializer.serialize_str("automod.message.hold"),
            SubscriptionType::AutomodMessageUpdate => serializer.serialize_str("automod.message.update"),
            SubscriptionType::AutomodMessageUpdateV2 => serializer.serialize_str("automod.message.update"),
            SubscriptionType::AutomodSettingsUpdate => serializer.serialize_str("automod.settings.update"),
            SubscriptionType::AutomodTermsUpdate => serializer.serialize_str("automod.terms.update"),
            SubscriptionType::ChannelUpdate => serializer.serialize_str("channel.update"),
            SubscriptionType::ChannelFollow => serializer.serialize_str("channel.follow"),
            SubscriptionType::ChannelAdBreakBegin => serializer.serialize_str("channel.ad_break.begin"),
            SubscriptionType::ChannelChatClear => serializer.serialize_str("channel.chat.clear"),
            SubscriptionType::ChannelChatClearUserMessages => {
                serializer.serialize_str("channel.chat.clear_user_messages")
            }
            SubscriptionType::ChannelChatMessage => serializer.serialize_str("channel.chat.message"),
            SubscriptionType::ChannelChatMessageDelete => serializer.serialize_str("channel.chat.message_delete"),
            SubscriptionType::ChannelChatNotification => serializer.serialize_str("channel.chat.notification"),
            SubscriptionType::ChannelChatSettingsUpdate => serializer.serialize_str("channel.chat_settings.update"),
            SubscriptionType::ChannelChatUserMessageHold => serializer.serialize_str("channel.chat.user_message_hold"),
            SubscriptionType::ChannelChatUserMessageUpdate => {
                serializer.serialize_str("channel.chat.user_message_update")
            }
            SubscriptionType::ChannelSharedChatSessionBegin => serializer.serialize_str("channel.shared_chat.begin"),
            SubscriptionType::ChannelSharedChatSessionUpdate => serializer.serialize_str("channel.shared_chat.update"),
            SubscriptionType::ChannelSharedChatSessionEnd => serializer.serialize_str("channel.shared_chat.end"),
            SubscriptionType::ChannelSubscribe => serializer.serialize_str("channel.subscribe"),
            SubscriptionType::ChannelSubscriptionEnd => serializer.serialize_str("channel.subscription.end"),
            SubscriptionType::ChannelSubscriptionGift => serializer.serialize_str("channel.subscription.gift"),
            SubscriptionType::ChannelSubscriptionMessage => serializer.serialize_str("channel.subscription.message"),
            SubscriptionType::ChannelCheer => serializer.serialize_str("channel.cheer"),
            SubscriptionType::ChannelRaid => serializer.serialize_str("channel.raid"),
            SubscriptionType::ChannelBan => serializer.serialize_str("channel.ban"),
            SubscriptionType::ChannelUnban => serializer.serialize_str("channel.unban"),
            SubscriptionType::ChannelUnbanRequestCreate => serializer.serialize_str("channel.unban_request.create"),
            SubscriptionType::ChannelUnbanRequestResolve => serializer.serialize_str("channel.unban_request.resolve"),
            SubscriptionType::ChannelModerate => serializer.serialize_str("channel.moderate"),
            SubscriptionType::ChannelModerateV2 => serializer.serialize_str("channel.moderate"),
            SubscriptionType::ChannelModeratorAdd => serializer.serialize_str("channel.moderator.add"),
            SubscriptionType::ChannelModeratorRemove => serializer.serialize_str("channel.moderator.remove"),
            SubscriptionType::ChannelGuestStarSessionBegin => {
                serializer.serialize_str("channel.guest_star_session.begin")
            }
            SubscriptionType::ChannelGuestStarSessionEnd => serializer.serialize_str("channel.guest_star_session.end"),
            SubscriptionType::ChannelGuestStarGuestUpdate => {
                serializer.serialize_str("channel.guest_star_guest.update")
            }
            SubscriptionType::ChannelGuestStarSettingsUpdate => {
                serializer.serialize_str("channel.guest_star_settings.update")
            }
            SubscriptionType::ChannelPointsAutomaticRewardRedemption => {
                serializer.serialize_str("channel.channel_points_automatic_reward_redemption.add")
            }
            SubscriptionType::ChannelPointsCustomRewardAdd => {
                serializer.serialize_str("channel.channel_points_custom_reward.add")
            }
            SubscriptionType::ChannelPointsCustomRewardUpdate => {
                serializer.serialize_str("channel.channel_points_custom_reward.update")
            }
            SubscriptionType::ChannelPointsCustomRewardRemove => {
                serializer.serialize_str("channel.channel_points_custom_reward.remove")
            }
            SubscriptionType::ChannelPointsCustomRewardRedemptionAdd => {
                serializer.serialize_str("channel.channel_points_custom_reward_redemption.add")
            }
            SubscriptionType::ChannelPointsCustomRewardRedemptionUpdate => {
                serializer.serialize_str("channel.channel_points_custom_reward_redemption.update")
            }
            SubscriptionType::ChannelPollBegin => serializer.serialize_str("channel.poll.begin"),
            SubscriptionType::ChannelPollProgress => serializer.serialize_str("channel.poll.progress"),
            SubscriptionType::ChannelPollEnd => serializer.serialize_str("channel.poll.end"),
            SubscriptionType::ChannelPredictionBegin => serializer.serialize_str("channel.prediction.begin"),
            SubscriptionType::ChannelPredictionProgress => serializer.serialize_str("channel.prediction.progress"),
            SubscriptionType::ChannelPredictionLock => serializer.serialize_str("channel.prediction.lock"),
            SubscriptionType::ChannelPredictionEnd => serializer.serialize_str("channel.prediction.end"),
            SubscriptionType::ChannelSuspiciousUserMessage => {
                serializer.serialize_str("channel.suspicious_user.message")
            }
            SubscriptionType::ChannelSuspiciousUserUpdate => serializer.serialize_str("channel.suspicious_user.update"),
            SubscriptionType::ChannelVIPAdd => serializer.serialize_str("channel.vip.add"),
            SubscriptionType::ChannelVIPRemove => serializer.serialize_str("channel.vip.remove"),
            SubscriptionType::ChannelWarningAcknowledgement => serializer.serialize_str("channel.warning.acknowledge"),
            SubscriptionType::ChannelWarningSend => serializer.serialize_str("channel.warning.send"),
            SubscriptionType::CharityDonation => serializer.serialize_str("channel.charity_campaign.donate"),
            SubscriptionType::CharityCampaignStart => serializer.serialize_str("channel.charity_campaign.start"),
            SubscriptionType::CharityCampaignProgress => serializer.serialize_str("channel.charity_campaign.progress"),
            SubscriptionType::CharityCampaignStop => serializer.serialize_str("channel.charity_campaign.stop"),
            SubscriptionType::ConduitShardDisabled => serializer.serialize_str("conduit.shard.disabled"),
            SubscriptionType::DropEntitlementGrant => serializer.serialize_str("drop.entitlement.grant"),
            SubscriptionType::ExtensionBitsTransactionCreate => {
                serializer.serialize_str("extension.bits_transaction.create")
            }
            SubscriptionType::GoalBegin => serializer.serialize_str("channel.goal.begin"),
            SubscriptionType::GoalProgress => serializer.serialize_str("channel.goal.progress"),
            SubscriptionType::GoalEnd => serializer.serialize_str("channel.goal.end"),
            SubscriptionType::HypeTrainBegin => serializer.serialize_str("channel.hype_train.begin"),
            SubscriptionType::HypeTrainProgress => serializer.serialize_str("channel.hype_train.progress"),
            SubscriptionType::HypeTrainEnd => serializer.serialize_str("channel.hype_train.end"),
            SubscriptionType::ShieldModeBegin => serializer.serialize_str("channel.shield_mode.begin"),
            SubscriptionType::ShieldModeEnd => serializer.serialize_str("channel.shield_mode.end"),
            SubscriptionType::ShoutoutCreate => serializer.serialize_str("channel.shoutout.create"),
            SubscriptionType::ShoutoutReceived => serializer.serialize_str("channel.shoutout.receive"),
            SubscriptionType::StreamOnline => serializer.serialize_str("stream.online"),
            SubscriptionType::StreamOffline => serializer.serialize_str("stream.offline"),
            SubscriptionType::UserAuthorizationGrant => serializer.serialize_str("user.authorization.grant"),
            SubscriptionType::UserAuthorizationRevoke => serializer.serialize_str("user.authorization.revoke"),
            SubscriptionType::UserUpdate => serializer.serialize_str("user.update"),
            SubscriptionType::WhisperReceived => serializer.serialize_str("user.whisper.message"),
            SubscriptionType::Unknown => serializer.serialize_str("Unknown"),
        }
    }
}

impl<'de> Deserialize<'de> for SubscriptionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        match s.as_str() {
            "automod.message.hold" => Ok(SubscriptionType::AutomodMessageHold),
            "automod.message.hold" => Ok(SubscriptionType::AutomodMessageHoldV2),
            "automod.message.update" => Ok(SubscriptionType::AutomodMessageUpdate),
            "automod.message.update" => Ok(SubscriptionType::AutomodMessageUpdateV2),
            "automod.settings.update" => Ok(SubscriptionType::AutomodSettingsUpdate),
            "automod.terms.update" => Ok(SubscriptionType::AutomodTermsUpdate),
            "channel.update" => Ok(SubscriptionType::ChannelUpdate),
            "channel.follow" => Ok(SubscriptionType::ChannelFollow),
            "channel.ad_break.begin" => Ok(SubscriptionType::ChannelAdBreakBegin),
            "channel.chat.clear" => Ok(SubscriptionType::ChannelChatClear),
            "channel.chat.clear_user_messages" => Ok(SubscriptionType::ChannelChatClearUserMessages),
            "channel.chat.message" => Ok(SubscriptionType::ChannelChatMessage),
            "channel.chat.message_delete" => Ok(SubscriptionType::ChannelChatMessageDelete),
            "channel.chat.notification" => Ok(SubscriptionType::ChannelChatNotification),
            "channel.chat_settings.update" => Ok(SubscriptionType::ChannelChatSettingsUpdate),
            "channel.chat.user_message_hold" => Ok(SubscriptionType::ChannelChatUserMessageHold),
            "channel.chat.user_message_update" => Ok(SubscriptionType::ChannelChatUserMessageUpdate),
            "channel.shared_chat.begin" => Ok(SubscriptionType::ChannelSharedChatSessionBegin),
            "channel.shared_chat.update" => Ok(SubscriptionType::ChannelSharedChatSessionUpdate),
            "channel.shared_chat.end" => Ok(SubscriptionType::ChannelSharedChatSessionEnd),
            "channel.subscribe" => Ok(SubscriptionType::ChannelSubscribe),
            "channel.subscription.end" => Ok(SubscriptionType::ChannelSubscriptionEnd),
            "channel.subscription.gift" => Ok(SubscriptionType::ChannelSubscriptionGift),
            "channel.subscription.message" => Ok(SubscriptionType::ChannelSubscriptionMessage),
            "channel.cheer" => Ok(SubscriptionType::ChannelCheer),
            "channel.raid" => Ok(SubscriptionType::ChannelRaid),
            "channel.ban" => Ok(SubscriptionType::ChannelBan),
            "channel.unban" => Ok(SubscriptionType::ChannelUnban),
            "channel.unban_request.create" => Ok(SubscriptionType::ChannelUnbanRequestCreate),
            "channel.unban_request.resolve" => Ok(SubscriptionType::ChannelUnbanRequestResolve),
            "channel.moderate" => Ok(SubscriptionType::ChannelModerate),
            "channel.moderate" => Ok(SubscriptionType::ChannelModerateV2),
            "channel.moderator.add" => Ok(SubscriptionType::ChannelModeratorAdd),
            "channel.moderator.remove" => Ok(SubscriptionType::ChannelModeratorRemove),
            "channel.guest_star_session.begin" => Ok(SubscriptionType::ChannelGuestStarSessionBegin),
            "channel.guest_star_session.end" => Ok(SubscriptionType::ChannelGuestStarSessionEnd),
            "channel.guest_star_guest.update" => Ok(SubscriptionType::ChannelGuestStarGuestUpdate),
            "channel.guest_star_settings.update" => Ok(SubscriptionType::ChannelGuestStarSettingsUpdate),
            "channel.channel_points_automatic_reward_redemption.add" => {
                Ok(SubscriptionType::ChannelPointsAutomaticRewardRedemption)
            }
            "channel.channel_points_custom_reward.add" => Ok(SubscriptionType::ChannelPointsCustomRewardAdd),
            "channel.channel_points_custom_reward.update" => Ok(SubscriptionType::ChannelPointsCustomRewardUpdate),
            "channel.channel_points_custom_reward.remove" => Ok(SubscriptionType::ChannelPointsCustomRewardRemove),
            "channel.channel_points_custom_reward_redemption.add" => {
                Ok(SubscriptionType::ChannelPointsCustomRewardRedemptionAdd)
            }
            "channel.channel_points_custom_reward_redemption.update" => {
                Ok(SubscriptionType::ChannelPointsCustomRewardRedemptionUpdate)
            }
            "channel.poll.begin" => Ok(SubscriptionType::ChannelPollBegin),
            "channel.poll.progress" => Ok(SubscriptionType::ChannelPollProgress),
            "channel.poll.end" => Ok(SubscriptionType::ChannelPollEnd),
            "channel.prediction.begin" => Ok(SubscriptionType::ChannelPredictionBegin),
            "channel.prediction.progress" => Ok(SubscriptionType::ChannelPredictionProgress),
            "channel.prediction.lock" => Ok(SubscriptionType::ChannelPredictionLock),
            "channel.prediction.end" => Ok(SubscriptionType::ChannelPredictionEnd),
            "channel.suspicious_user.message" => Ok(SubscriptionType::ChannelSuspiciousUserMessage),
            "channel.suspicious_user.update" => Ok(SubscriptionType::ChannelSuspiciousUserUpdate),
            "channel.vip.add" => Ok(SubscriptionType::ChannelVIPAdd),
            "channel.vip.remove" => Ok(SubscriptionType::ChannelVIPRemove),
            "channel.warning.acknowledge" => Ok(SubscriptionType::ChannelWarningAcknowledgement),
            "channel.warning.send" => Ok(SubscriptionType::ChannelWarningSend),
            "channel.charity_campaign.donate" => Ok(SubscriptionType::CharityDonation),
            "channel.charity_campaign.start" => Ok(SubscriptionType::CharityCampaignStart),
            "channel.charity_campaign.progress" => Ok(SubscriptionType::CharityCampaignProgress),
            "channel.charity_campaign.stop" => Ok(SubscriptionType::CharityCampaignStop),
            "conduit.shard.disabled" => Ok(SubscriptionType::ConduitShardDisabled),
            "drop.entitlement.grant" => Ok(SubscriptionType::DropEntitlementGrant),
            "extension.bits_transaction.create" => Ok(SubscriptionType::ExtensionBitsTransactionCreate),
            "channel.goal.begin" => Ok(SubscriptionType::GoalBegin),
            "channel.goal.progress" => Ok(SubscriptionType::GoalProgress),
            "channel.goal.end" => Ok(SubscriptionType::GoalEnd),
            "channel.hype_train.begin" => Ok(SubscriptionType::HypeTrainBegin),
            "channel.hype_train.progress" => Ok(SubscriptionType::HypeTrainProgress),
            "channel.hype_train.end" => Ok(SubscriptionType::HypeTrainEnd),
            "channel.shield_mode.begin" => Ok(SubscriptionType::ShieldModeBegin),
            "channel.shield_mode.end" => Ok(SubscriptionType::ShieldModeEnd),
            "channel.shoutout.create" => Ok(SubscriptionType::ShoutoutCreate),
            "channel.shoutout.receive" => Ok(SubscriptionType::ShoutoutReceived),
            "stream.online" => Ok(SubscriptionType::StreamOnline),
            "stream.offline" => Ok(SubscriptionType::StreamOffline),
            "user.authorization.grant" => Ok(SubscriptionType::UserAuthorizationGrant),
            "user.authorization.revoke" => Ok(SubscriptionType::UserAuthorizationRevoke),
            "user.update" => Ok(SubscriptionType::UserUpdate),
            "user.whisper.message" => Ok(SubscriptionType::WhisperReceived),

            _ => {
                info!("Unknown MessageType value: {s}");

                Ok(SubscriptionType::Unknown)
            } // Handle unknown variants gracefully
        }
    }
}
