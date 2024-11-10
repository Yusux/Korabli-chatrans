use replay_parser::analyzer::decoder::{DecodedPacket, DecodedPacketPayload};
use replay_parser::analyzer::Analyzer;
use replay_parser::packet2::Packet;
use std::{
    collections::HashMap,
    convert::TryInto,
};
use async_channel::Sender;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub clock: f32,
    pub sender: String,
    pub audience: String,
    pub message: String,
}

pub struct ChatLoggerBuilder;

impl ChatLoggerBuilder {
    pub fn new() -> ChatLoggerBuilder {
        ChatLoggerBuilder
    }

    pub fn build(
        &self,
        tx:  Sender<ChatMessage>,
    ) -> Box<dyn Analyzer> {
        Box::new(ChatLogger {
            usernames: HashMap::new(),
            tx,
        })
    }
}

pub struct ChatLogger {
    usernames: HashMap<i32, String>,
    tx: Sender<ChatMessage>,
}

impl Analyzer for ChatLogger {
    fn finish(&self) {}

    fn process(&mut self, packet: &Packet<'_, '_>) {
        let decoded = DecodedPacket::from(false, packet);
        match decoded.payload {
            DecodedPacketPayload::Chat {
                sender_id,
                audience,
                message,
                ..
            } => {
                // Chat { entity_id: 409451, sender_id: -1, audience: "battle_prebattle", message: "IDS_OP_01_02_LEEROYY" } ?
                // If sender_id not in usernames
                if !self.usernames.contains_key(&sender_id) {
                    return;
                }
                debug!(
                    "{}: {}: {} {}",
                    decoded.clock,
                    self.usernames.get(&sender_id).unwrap(),
                    // sender_id,
                    audience,
                    message
                );
                let _ = self.tx.send_blocking(ChatMessage {
                    clock: decoded.clock,
                    sender: self.usernames.get(&sender_id).unwrap().clone(),
                    audience: audience.to_string(),
                    message: message.to_string(),
                });
            }
            DecodedPacketPayload::EntityInfo { players, .. } => {
                for player in players.iter() {
                    self.usernames
                        .insert(player.playerid.try_into().unwrap(), player.username.clone());
                }
            }
            _ => {}
        }
    }
}
