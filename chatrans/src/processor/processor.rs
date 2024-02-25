use replay_parser::analyzer::decoder::{DecodedPacket, DecodedPacketPayload};
use replay_parser::analyzer::Analyzer;
use replay_parser::packet2::Packet;
use std::{
    collections::HashMap,
    convert::TryInto,
};
use async_channel::Sender;
use tracing::{debug, info};

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
}

impl ChatLoggerBuilder {
    pub fn build(
        &self,
        meta: &replay_parser::ReplayMeta,
        tx:  Sender<ChatMessage>,
    ) -> Box<dyn Analyzer> {
        let version = replay_parser::version::Version::from_client_exe(&meta.clientVersionFromExe);
        Box::new(ChatLogger {
            usernames: HashMap::new(),
            version,
            tx,
        })
    }
}

pub struct ChatLogger {
    usernames: HashMap<i32, String>,
    version: replay_parser::version::Version,
    tx: Sender<ChatMessage>,
}

impl Analyzer for ChatLogger {
    fn finish(&self) {}

    fn process(&mut self, packet: &Packet<'_, '_>) {
        let decoded = DecodedPacket::from(&self.version, false, packet);
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
            DecodedPacketPayload::VoiceLine {
                sender_id, message, ..
            } => {
                info!(
                    "{}: {}: voiceline {:#?}",
                    decoded.clock,
                    self.usernames.get(&sender_id).unwrap(),
                    message
                );
            }
            DecodedPacketPayload::OnArenaStateReceived { players, .. } => {
                for player in players.iter() {
                    self.usernames
                        .insert(player.playerid.try_into().unwrap(), player.username.clone());
                }
            }
            _ => {}
        }
    }
}
