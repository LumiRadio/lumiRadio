use poise::serenity_prelude as serenity;
use sentry::Breadcrumb;
use serde_json::json;
use serenity::GatewayIntents;

use std::sync::Arc;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use crate::app_config::EmojiConfig;
use judeharley::communication::{ByersUnixStream, LiquidsoapCommunication};

lazy_static! {
    pub static ref INTENTS: GatewayIntents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS;
}

pub type Context<'a> = poise::Context<'a, Data<ByersUnixStream>, Error>;
pub type ApplicationContext<'a> = poise::ApplicationContext<'a, Data<ByersUnixStream>, Error>;
pub type Error = anyhow::Error;

pub struct Data<C>
where
    C: LiquidsoapCommunication,
{
    pub db: judeharley::sea_orm::DatabaseConnection,
    pub comms: Arc<Mutex<C>>,
    pub redis_pool: fred::pool::RedisPool,
    pub redis_subscriber: fred::clients::SubscriberClient,
    pub emoji: EmojiConfig,
}

pub struct BreadcrumbableContext<'a>(pub Context<'a>);

impl<'a> BreadcrumbableContext<'a> {
    pub async fn as_breadcrumbs(&self) -> Vec<Breadcrumb> {
        let command_breadcrumb =
            match &self.0 {
                poise::Context::Application(ctx) => Breadcrumb {
                    ty: "user".into(),
                    category: Some("slash_command".into()),
                    data: {
                        let mut map = sentry::protocol::Map::new();
                        let command = ctx.command();

                        map.insert("name".into(), command.name.clone().into());
                        if let Some(member) = ctx.author_member().await {
                            map.insert("user".into(), json!({
                            "id": member.user.id.get(),
                            "name": member.user.name,
                            "display_name": member.display_name(),
                            "roles": member.roles.iter().map(|r| r.get()).collect::<Vec<_>>(),
                        }));
                        } else {
                            map.insert(
                                "user".into(),
                                json!({
                                    "id": ctx.author().id.get(),
                                    "name": ctx.author().name
                                }),
                            );
                        }
                        map.insert(
                            "channel".into(),
                            json!({
                                "id": ctx.channel_id().get(),
                                "name": ctx.channel_id().name(ctx.http()).await.ok()
                            }),
                        );
                        map.insert(
                            "interaction_type".into(),
                            format!("{:?}", ctx.interaction_type).into(),
                        );
                        map.insert("interaction_id".into(), ctx.interaction.id.get().into());
                        map.insert(
                            "parameters".into(),
                            ctx.args
                                .iter()
                                .map(|a| {
                                    let j = json!({
                                        "name": a.name,
                                        "value": format!("{:?}", a.value)
                                    });

                                    j
                                })
                                .collect::<Vec<_>>()
                                .into(),
                        );
                        map.insert(
                            "parent_commands".into(),
                            ctx.parent_commands()
                                .iter()
                                .map(|c| c.name.clone())
                                .collect::<Vec<_>>()
                                .into(),
                        );

                        map
                    },
                    ..Default::default()
                },
                poise::Context::Prefix(ctx) => Breadcrumb {
                    ty: "user".into(),
                    category: Some("prefix_command".into()),
                    data: {
                        let mut map = sentry::protocol::Map::new();
                        map.insert("message".into(), ctx.msg.content.clone().into());
                        if let Some(member) = ctx.author_member().await {
                            map.insert("user".into(), json!({
                            "id": member.user.id.get(),
                            "name": member.user.name,
                            "display_name": member.display_name(),
                            "roles": member.roles.iter().map(|r| r.get()).collect::<Vec<_>>(),
                        }));
                        } else {
                            map.insert(
                                "user".into(),
                                json!({
                                    "id": ctx.author().id.get(),
                                    "name": ctx.author().name
                                }),
                            );
                        }
                        map.insert(
                            "channel".into(),
                            json!({
                                "id": ctx.channel_id().get(),
                                "name": ctx.channel_id().name(ctx.http()).await.ok()
                            }),
                        );
                        map
                    },
                    ..Default::default()
                },
            };

        vec![command_breadcrumb]
    }
}
