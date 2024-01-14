use std::fmt;

use thiserror::Error;
use tokio::sync::mpsc::error::{SendError, TryRecvError};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::RwLock;

pub mod discord;

use crate::prompt::ChatMessage;

pub struct CommunicationManager {
    two_way_channels: Vec<TwoWayChannel>,
    incoming_channels: Vec<OneWayChannelReceiver>,
    outgoing_channels: Vec<OneWayChannelSender>,
}

impl CommunicationManager {
    pub fn new() -> Self {
        Self {
            two_way_channels: Vec::new(),
            incoming_channels: Vec::new(),
            outgoing_channels: Vec::new(),
        }
    }

    pub fn open_two_way_channel(&mut self, name: &str) -> TwoWayChannel {
        let (to_agent, agent) = open_channel(format!("{}_to_agent", name).as_str());
        let (to_external, external) = open_channel(format!("{}_to_external", name).as_str());

        self.two_way_channels.push(TwoWayChannel {
            name: format!("{}_internal", name),
            sender: to_external,
            receiver: agent,
        });

        TwoWayChannel {
            name: format!("{}_external", name),
            sender: to_agent,
            receiver: external,
        }
    }

    pub fn open_incoming_channel(&mut self, name: &str) -> OneWayChannelSender {
        let (to_agent, agent) = open_channel(name);

        self.incoming_channels.push(agent);
        to_agent
    }

    pub fn open_outgoing_channel(&mut self, name: &str) -> OneWayChannelReceiver {
        let (to_external, external) = open_channel(name);

        self.outgoing_channels.push(to_external);
        external
    }

    pub async fn receive_messages(&self) -> Vec<ChatMessage> {
        let mut messages = Vec::new();

        for channel in &self.two_way_channels {
            let Ok(mut msg_list) = channel.receive_messages().await else {
                continue;
            };
            messages.append(&mut msg_list);
        }

        for channel in &self.incoming_channels {
            let Ok(mut msg_list) = channel.receive_messages().await else {
                continue;
            };
            messages.append(&mut msg_list);
        }

        messages
    }

    pub async fn send_message(&self, message: &ChatMessage) {
        for channel in &self.two_way_channels {
            if (channel.send_message(message.clone()).await).is_err() {
                println!("Failed to send message to two-way channel: {}", channel);
            };
        }

        for channel in &self.outgoing_channels {
            if (channel.send_message(message.clone()).await).is_err() {
                println!("Failed to send message to two-way channel: {}", channel);
            };
        }
    }
}

impl Default for CommunicationManager {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TwoWayChannel {
    name: String,
    sender: OneWayChannelSender,
    receiver: OneWayChannelReceiver,
}

impl TwoWayChannel {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub async fn send_message(&self, message: ChatMessage) -> Result<(), CommunicationsError> {
        self.sender.send_message(message).await
    }

    pub async fn receive_messages(&self) -> Result<Vec<ChatMessage>, CommunicationsError> {
        self.receiver.receive_messages().await
    }

    pub async fn receive_message_blocking(&self) -> Result<ChatMessage, CommunicationsError> {
        self.receiver.receive_message_blocking().await
    }
}

impl fmt::Display for TwoWayChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Communications({})", self.name)
    }
}

pub struct OneWayChannelSender {
    name: String,
    tx: Sender<ChatMessage>,
}

impl OneWayChannelSender {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub async fn send_message(&self, message: ChatMessage) -> Result<(), CommunicationsError> {
        self.tx.send(message).await?;
        Ok(())
    }
}

impl fmt::Display for OneWayChannelSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Communications({})", self.name)
    }
}

pub struct OneWayChannelReceiver {
    name: String,
    rx: RwLock<Receiver<ChatMessage>>,
}

impl OneWayChannelReceiver {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub async fn receive_messages(&self) -> Result<Vec<ChatMessage>, CommunicationsError> {
        let mut messages = Vec::new();
        let mut receiver = self.rx.write().await;

        loop {
            match receiver.try_recv() {
                Ok(message) => messages.push(message),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(CommunicationsError::EndOfStream);
                }
            }
        }

        Ok(messages)
    }

    pub async fn receive_message_blocking(&self) -> Result<ChatMessage, CommunicationsError> {
        self.rx
            .write()
            .await
            .recv()
            .await
            .ok_or(CommunicationsError::EndOfStream)
    }
}

impl fmt::Display for OneWayChannelReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Communications({})", self.name)
    }
}

#[derive(Debug, Error)]
pub enum CommunicationsError {
    #[error("Channel closed: end of stream")]
    EndOfStream,
    #[error("Channel closed: failed to send")]
    ChannelClosed(#[from] SendError<ChatMessage>),
}

fn open_channel(name: &str) -> (OneWayChannelSender, OneWayChannelReceiver) {
    let (tx, rx) = mpsc::channel(64);

    (
        OneWayChannelSender {
            name: format!("{}_receiver", name),
            tx,
        },
        OneWayChannelReceiver {
            name: format!("{}_sender", name),
            rx: RwLock::new(rx),
        },
    )
}
