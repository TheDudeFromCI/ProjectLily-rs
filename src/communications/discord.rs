use std::env;

use log::{error, info, warn};
use serenity::all::ChannelId;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use super::CommunicationManager;
use crate::communications::TwoWayChannel;
use crate::prompt::{ChatMessage, MessageAction, SystemMessageSeverity};

struct ProjectLilyDiscordHandler {
    channel_id: ChannelId,
    channel: TwoWayChannel,
}

pub fn run(communications: &mut CommunicationManager) {
    let Ok(token) = env::var("DISCORD_TOKEN") else {
        warn!("Failed to find Discord token in environment. Discord will not be available.");
        return;
    };

    let Ok(channel_id_str) = env::var("DISCORD_CHANNEL") else {
        warn!("Failed to find Discord channel ID in environment. Discord will not be available.");
        return;
    };

    let Ok(channel_id_index) = channel_id_str.parse::<u64>() else {
        error!("Failed to parse Discord channel ID as a number. Discord will not be available.");
        return;
    };

    let channel = communications.open_two_way_channel("discord");

    tokio::spawn(async move {
        let channel_id = ChannelId::new(channel_id_index);

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(&token, intents)
            .event_handler(ProjectLilyDiscordHandler {
                channel_id,
                channel,
            })
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
            })
            .await;

        if let Err(err) = channel_state {
            println!("Failed to send message to Agent: {}", err);
            return;
        }

        loop {
            let message = self.channel.receive_message_blocking().await;
            let Ok(message) = message else {
                println!("Discord connection to Agent has been closed.");
                break;
            };

            if let ChatMessage::Assistant {
                action: MessageAction::Say,
                content,
                ..
            } = message
            {
                self.channel_id
                    .send_message(&ctx.http, CreateMessage::new().content(content))
                    .await
                    .unwrap();
            }
        }
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let channel_state = self
            .channel
            .send_message(ChatMessage::User {
                username: msg.author.name,
                action: MessageAction::Say,
                content: msg.content.clone(),
            })
            .await;

        if let Err(err) = channel_state {
            println!("Failed to send message to Agent: {}", err);
            return;
        }
    }
}
