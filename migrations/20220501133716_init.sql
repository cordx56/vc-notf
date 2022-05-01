-- Add migration script here
CREATE TABLE guild_notf_channels (
    guild_id bigint PRIMARY KEY,
    channel_name text NOT NULL
);
