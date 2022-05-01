use serenity::async_trait;
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::StandardFramework;
use serenity::futures::future::ready;
use serenity::model::{
    channel::{Channel, ChannelType, GuildChannel},
    gateway::Ready,
    id::GuildId,
    interactions::{
        application_command::{
            ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
            ApplicationCommandOptionType,
        },
        Interaction, InteractionResponseType,
    },
    voice::VoiceState,
};
use serenity::prelude::GatewayIntents;
use serenity::utils::Color;

use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

use sqlx::postgres::PgPoolOptions;

pub mod models;

struct Handler {
    db_pool: PgPool,
}

async fn search_notf_channels(
    ctx: &Context,
    db_pool: &PgPool,
    guild: GuildId,
) -> serenity::Result<Vec<GuildChannel>> {
    let mut notf_channels = Vec::new();
    let mut notf_channel_name = env::var("NOTF_CHANNEL_NAME").unwrap_or("vc-notf".to_string());
    let query_result = sqlx::query_as::<_, models::GuildNotfChannel>(
        "select * from guild_notf_channels where guild_id = $1",
    )
    .bind(guild.0 as i64)
    .fetch_one(db_pool)
    .await;
    if let Ok(guild_notf_channel) = query_result {
        notf_channel_name = guild_notf_channel.channel_name;
    } else {
        println!("{:?}", query_result);
    }
    let channel_map = guild.channels(ctx).await?;
    for (_channel_id, guild_channel) in channel_map {
        if guild_channel.name == notf_channel_name && guild_channel.kind == ChannelType::Text {
            notf_channels.push(guild_channel);
        }
    }
    Ok(notf_channels)
}

#[async_trait]
impl EventHandler for Handler {
    async fn voice_state_update(
        &self,
        ctx: Context,
        old_vs_opt: Option<VoiceState>,
        new_vs: VoiceState,
    ) {
        if let Some(guild_id) = new_vs.guild_id {
            if let Ok(notf_channels) = search_notf_channels(&ctx, &self.db_pool, guild_id).await {
                if let Some(member) = new_vs.member {
                    let member_display_name = member.display_name();
                    let member_user_id = member.user.id;
                    let member_avatar_url = member.face();
                    if let Some(old_vs) = old_vs_opt {
                        if let Some(old_channel_id) = old_vs.channel_id {
                            if let Some(old_channel_name) = old_channel_id.name(&ctx).await {
                                if let Some(new_channel_id) = new_vs.channel_id {
                                    if let Some(new_channel_name) = new_channel_id.name(&ctx).await
                                    {
                                        if old_channel_id != new_channel_id {
                                            for notf_channel in notf_channels {
                                                notf_channel
                                                    .send_message(&ctx, |m| {
                                                        m.add_embed(|e| {
                                                            e.title(format!(
                                                                "{} moved VC!",
                                                                member_display_name
                                                            ))
                                                            .description(format!(
                                                                "<@{}> moved from {} to {}!",
                                                                member_user_id,
                                                                old_channel_name,
                                                                new_channel_name,
                                                            ))
                                                            .color(Color::from_rgb(23, 162, 184))
                                                            .thumbnail(&member_avatar_url)
                                                        })
                                                    })
                                                    .await
                                                    .ok();
                                            }
                                        }
                                    }
                                } else {
                                    for notf_channel in notf_channels {
                                        notf_channel
                                            .send_message(&ctx, |m| {
                                                m.add_embed(|e| {
                                                    e.title(format!(
                                                        "{} left VC!",
                                                        member_display_name
                                                    ))
                                                    .description(format!(
                                                        "<@{}> left {}!",
                                                        member_user_id, old_channel_name,
                                                    ))
                                                    .color(Color::from_rgb(220, 53, 59))
                                                    .thumbnail(&member_avatar_url)
                                                })
                                            })
                                            .await
                                            .ok();
                                    }
                                }
                            }
                        }
                    } else {
                        if let Some(new_channel_id) = new_vs.channel_id {
                            if let Some(new_channel_name) = new_channel_id.name(&ctx).await {
                                for notf_channel in notf_channels {
                                    notf_channel
                                        .send_message(&ctx, |m| {
                                            m.add_embed(|e| {
                                                e.title(format!(
                                                    "{} joined VC!",
                                                    member_display_name
                                                ))
                                                .description(format!(
                                                    "<@{}> joined {}!",
                                                    member_user_id, new_channel_name,
                                                ))
                                                .color(Color::from_rgb(40, 167, 69))
                                                .thumbnail(&member_avatar_url)
                                            })
                                        })
                                        .await
                                        .ok();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected guilds:");
        for guild in ready.guilds {
            println!("    {}", guild.id);
        }
        if let Err(_commands) =
            ApplicationCommand::set_global_application_commands(&ctx, |commands| {
                commands
                    .create_application_command(|command| {
                        command
                            .name("members")
                            .description("Get members of voice channel")
                            .create_option(|option| {
                                option
                                    .name("name")
                                    .description("voice channel")
                                    .kind(ApplicationCommandOptionType::Channel)
                                    .required(true)
                            })
                    })
                    .create_application_command(|command| {
                        command
                            .name("notfchannel")
                            .description("Specify channel to send notification")
                            .create_option(|option| {
                                option
                                    .name("name")
                                    .description("text channel")
                                    .kind(ApplicationCommandOptionType::Channel)
                                    .required(true)
                            })
                    })
                    .create_application_command(|command| {
                        command
                            .name("onjoin")
                            .description("Join notification settings")
                            .create_option(|option| {
                                option
                                    .name("send")
                                    .description("send or not")
                                    .kind(ApplicationCommandOptionType::Boolean)
                                    .required(true)
                            })
                    })
                    .create_application_command(|command| {
                        command
                            .name("onmove")
                            .description("Move notification settings")
                            .create_option(|option| {
                                option
                                    .name("send")
                                    .description("send or not")
                                    .kind(ApplicationCommandOptionType::Boolean)
                                    .required(true)
                            })
                    })
                    .create_application_command(|command| {
                        command
                            .name("onleave")
                            .description("Leave notification settings")
                            .create_option(|option| {
                                option
                                    .name("send")
                                    .description("send or not")
                                    .kind(ApplicationCommandOptionType::Boolean)
                                    .required(true)
                            })
                    })
            })
            .await
        {
            println!("Commands set failed");
        }
    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "members" => {
                    if let Some(option) = command.data.options.get(0) {
                        if let Some(option_resolved) = option.resolved.as_ref() {
                            if let ApplicationCommandInteractionDataOptionValue::Channel(pchannel) =
                                option_resolved
                            {
                                if pchannel.kind == ChannelType::Voice {
                                    if let Ok(channel) = pchannel.id.to_channel(&ctx).await {
                                        if let Channel::Guild(guild_channel) = channel {
                                            if let Ok(members) = guild_channel.members(&ctx).await {
                                                let mut members_str = String::new();
                                                for member in &members {
                                                    members_str.push_str(&format!(
                                                        "<@{}>\n",
                                                        member.user.id.as_u64()
                                                    ));
                                                }
                                                if let Err(why) = command
                                                    .create_interaction_response(&ctx, |response| {response.kind(InteractionResponseType::ChannelMessageWithSource)
                                                        .interaction_response_data(|m| m.embed(|e| {
                                                            e.title(format!("{} members in {}", members.len(), guild_channel.name())).field("Members", members_str, false).color(Color::from_rgb(40, 167, 69))
                                                        }))
                                                    })
                                                    .await
                                                    {
                                                        println!("Cannot respond to slash command: {}", why);
                                                    }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "notfchannel" => {
                    if let Some(option) = command.data.options.get(0) {
                        if let Some(option_resolved) = option.resolved.as_ref() {
                            if let ApplicationCommandInteractionDataOptionValue::Channel(pchannel) =
                                option_resolved
                            {
                                if pchannel.kind == ChannelType::Text {
                                    if let Ok(channel) = pchannel.id.to_channel(&ctx).await {
                                        if let Channel::Guild(guild_channel) = channel {
                                            let channel_name = guild_channel.name();
                                            let guild_id = guild_channel.guild_id;
                                            let query_result = sqlx::query("insert into guild_notf_channels (guild_id, channel_name) values ($1, $2) on conflict (guild_id) do update set channel_name = $2").bind(guild_id.0 as i64).bind(channel_name).execute(&self.db_pool).await;
                                            if let Ok(_) = query_result {
                                                if let Err(why) = command
                                                    .create_interaction_response(&ctx, |response| {response.kind(InteractionResponseType::ChannelMessageWithSource)
                                                        .interaction_response_data(|m| m.content(format!("I'll send notification to {}", channel_name)))
                                                    })
                                                    .await
                                                    {
                                                        println!("Cannot respond to slash command: {}", why);
                                                    }
                                            } else {
                                                println!("{:?}", query_result);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                "onjoin" => {}
                "onmove" => {}
                "onleave" => {}
                _ => {
                    if let Err(why) = command
                        .create_interaction_response(&ctx, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|m| {
                                    m.content("not implemented :(".to_string())
                                })
                        })
                        .await
                    {
                        println!("Cannot respond to slash command: {}", why);
                    }
                }
            };
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let token = env::var("DISCORD_BOT_TOKEN").expect("Discord bot token missing!");
    let application_id: u64 = env::var("DISCORD_APPLICATION_ID")
        .expect("Discord application ID missing!")
        .parse()
        .expect("Invalid application ID");
    let framework = StandardFramework::new();
    let intents = GatewayIntents::non_privileged();
    let mut client = Client::builder(token, intents)
        .event_handler(Handler { db_pool: db_pool })
        .application_id(application_id)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
