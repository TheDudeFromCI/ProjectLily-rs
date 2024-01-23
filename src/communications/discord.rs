use std::env;

use log::{info, warn};
use serenity::all::ChannelId;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use super::CommunicationManager;
use crate::communications::TwoWayChannel;
use crate::prompt::{ChatMessage, MessageAction, SystemMessageSeverity};

pub struct DiscordSettings {
    pub channel_id: Option<u64>,
    pub log_all: bool,
}

struct ProjectLilyDiscordHandler {
    channel: TwoWayChannel,
    settings: DiscordSettings,
}

pub fn run(settings: DiscordSettings, communications: &mut CommunicationManager) {
    let Ok(token) = env::var("DISCORD_TOKEN") else {
        warn!("Failed to find Discord token in environment. Discord will not be available.");
        return;
    };

    if settings.channel_id.is_none() {
        warn!("Discord channel ID was not provided. Discord will not be available.");
        return;
    };

    let channel = communications.open_two_way_channel("discord");

    tokio::spawn(async move {
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(&token, intents)
            .event_handler(ProjectLilyDiscordHandler { channel, settings })
            .await
            .expect("Error creating client");

        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    });
}

#[serenity::async_trait]
impl EventHandler for ProjectLilyDiscordHandler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected to Discord as {}", ready.user.name);

        let content = format!("You have connected to Discord as `{}`", ready.user.name);

        let channel_state = self
            .channel
            .send_message(ChatMessage::System {
                severity: SystemMessageSeverity::Info,
                content,
                tokens: None,
            })
            .await;

        if let Err(err) = channel_state {
            println!("Failed to send message to Agent: {}", err);
            return;
        }

        let channel_id = ChannelId::new(self.settings.channel_id.unwrap());

        loop {
            let message = self.channel.receive_message_blocking().await;
            let Ok(message) = message else {
                println!("Discord connection to Agent has been closed.");
                break;
            };

            if self.settings.log_all {
                if let ChatMessage::Assistant {
                    action, content, ..
                } = message
                {
                    let message = format!("{}: {}", action, content);
                    channel_id
                        .send_message(&ctx.http, CreateMessage::new().content(message))
                        .await
                        .unwrap();
                }
            } else if let ChatMessage::Assistant {
                action: MessageAction::Say,
                content,
                ..
            } = message
            {
                channel_id
                    .send_message(&ctx.http, CreateMessage::new().content(content))
                    .await
                    .unwrap();
            }
        }
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        let expected_channel_id = ChannelId::new(self.settings.channel_id.unwrap());
        if msg.channel_id != expected_channel_id {
            return;
        }

        if msg.author.bot {
            return;
        }

        let channel_state = self
            .channel
            .send_message(ChatMessage::User {
                username: msg.author.name,
                content: msg.content.clone(),
                tokens: None,
            })
            .await;

        if let Err(err) = channel_state {
            println!("Failed to send message to Agent: {}", err);
            return;
        }
    }
}
