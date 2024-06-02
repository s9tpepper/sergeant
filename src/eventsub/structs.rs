use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Condition {
    pub broadcaster_user_id: Option<String>,
    pub moderator_user_id: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transport {
    pub method: String,
    pub session_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Subscription {
    pub r#type: String,
    pub version: String,
    pub condition: Condition,
    pub transport: Transport,
    pub status: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BadgeInfo {
    pub set_id: String,
    pub id: String,
    pub info: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionEventMessage {
    pub text: String,
    pub fragments: Vec<Fragment>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Fragment {
    pub r#type: String,
    pub text: String,
    pub cheermote: Option<Cheermote>,
    pub emote: Option<FragmentEmote>,
    pub mention: Option<Mention>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Mention {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FragmentEmote {
    pub id: String,
    pub emote_set_id: String,
    pub owner_id: String,
    pub format: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cheermote {
    pub prefix: String,
    pub bits: u64,
    pub tier: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sub {
    pub sub_tier: String,
    pub is_prime: bool,
    pub duration_months: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Resub {
    pub cumulative_months: u16,
    pub duration_months: u16,
    pub streak_months: u16,
    pub sub_tier: String,
    pub is_prime: Option<bool>,
    pub is_gift: bool,
    pub gifter_is_anonymous: Option<bool>,
    pub gifter_user_id: Option<String>,
    pub gifter_user_name: Option<String>,
    pub gifter_user_login: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubGift {
    pub duration_months: u16,
    pub cumulative_total: Option<u16>,
    pub recipient_user_id: String,
    pub recipient_user_name: String,
    pub recipient_user_login: String,
    pub sub_tier: String,
    pub community_gift_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommunitySubGift {
    pub id: String,
    pub total: u16,
    pub sub_tier: String,
    pub cumulative_total: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GiftPaidUpgrade {
    pub gifter_is_anonymous: bool,
    pub gifter_user_id: Option<String>,
    pub gifter_user_name: Option<String>,
    pub gifter_user_login: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrimePaidUpgrade {
    pub sub_tier: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Raid {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
    pub viewer_count: u64,
    pub profile_image_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Unraid {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Announcement {
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BitsBadgeTier {
    pub tier: u8,
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
pub struct SubscriptionEvent {
    pub announcement: Option<Announcement>,
    pub badges: Option<Vec<BadgeInfo>>,
    pub bits_badge_tier: Option<BitsBadgeTier>,
    pub broadcaster_user_id: String,
    pub broadcaster_user_login: String,
    pub broadcaster_user_name: String,
    pub charity_donation: Option<CharityDonation>,
    pub chatter_user_id: Option<String>,
    pub chatter_user_login: Option<String>,
    pub chatter_user_name: Option<String>,
    pub chatter_is_anonymouse: Option<bool>,
    pub color: Option<String>,
    pub community_sub_gift: Option<CommunitySubGift>,
    pub duration_seconds: Option<u64>,
    pub gift_paid_upgrade: Option<GiftPaidUpgrade>,
    pub is_automatic: bool,
    pub message: Option<SubscriptionEventMessage>,
    pub message_id: Option<String>,
    pub notice_type: Option<String>,
    pub pay_it_forward: Option<GiftPaidUpgrade>,
    pub prime_paid_upgrade: Option<PrimePaidUpgrade>,
    pub raid: Option<Raid>,
    pub requester_user_id: Option<String>,
    pub requester_user_login: Option<String>,
    pub requester_user_name: Option<String>,
    pub resub: Option<Resub>,
    pub started_at: String,
    pub sub: Option<Sub>,
    pub sub_gift: Option<SubGift>,
    pub system_message: Option<String>,
    pub target_user_id: Option<String>,
    pub target_user_login: Option<String>,
    pub target_user_name: Option<String>,
    pub unraid: Option<Unraid>,
}
