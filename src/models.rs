#[derive(sqlx::FromRow, Debug)]
pub struct GuildNotfChannel {
    pub guild_id: i64,
    pub channel_name: String,
}

pub enum VoiceStateEvent {
    Join = 0,
    Move = 1,
    Leave = 2,
}

pub struct GuildNotfDisabled {
    pub guild_id: u64,
    pub event: u8,
}
