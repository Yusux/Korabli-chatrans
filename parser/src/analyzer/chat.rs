use crate::analyzer::decoder::{DecodedPacket, DecodedPacketPayload};
use crate::analyzer::{Analyzer, AnalyzerBuilder};
use crate::packet2::Packet;
use std::collections::HashMap;
use std::convert::TryInto;

pub struct ChatLoggerBuilder;

impl ChatLoggerBuilder {
    pub fn new() -> ChatLoggerBuilder {
        ChatLoggerBuilder
    }
}

impl AnalyzerBuilder for ChatLoggerBuilder {
    fn build(&self, meta: &crate::ReplayMeta) -> Box<dyn Analyzer> {
        let version = crate::version::Version::from_client_exe(&meta.clientVersionFromExe);
        Box::new(ChatLogger {
            usernames: HashMap::new(),
            version,
        })
    }
}

pub struct ChatLogger {
    usernames: HashMap<i32, String>,
    version: crate::version::Version,
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
                // println!("{:?}", decoded.payload);
                // println!("usernames: {:?}", self.usernames);
                // if sender_id not in usernames
                if !self.usernames.contains_key(&sender_id) {
                    // println!("sender_id not in usernames: {}", sender_id);
                    return;
                }
                println!(
                    "{}: {}: {} {}",
                    decoded.clock,
                    self.usernames.get(&sender_id).unwrap(),
                    // sender_id,
                    audience,
                    message
                );
            }
            DecodedPacketPayload::VoiceLine {
                sender_id, message, ..
            } => {
                println!(
                    "{}: {}: voiceline {:#?}",
                    decoded.clock,
                    self.usernames.get(&sender_id).unwrap(),
                    message
                );
            }
            DecodedPacketPayload::OnArenaStateReceived { players, .. } => {
                for player in players.iter() {
                    // println!("player: {:#?}", player);
                    self.usernames
                        .insert(player.playerid.try_into().unwrap(), player.username.clone());
                }
            }
            _ => {}
        }
    }
}
