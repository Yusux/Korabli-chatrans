use blowfish::Blowfish;
use cipher::{BlockDecrypt, KeyInit};
use nom::bytes::complete::take;
use nom::number::complete::le_u32;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use tracing::debug;

use crate::error::*;

const REPLAY_HEADER: [u8; 4] = [
    0x12, 0x32, 0x34, 0x11
];

const BLOWFISH_KEY: [u8; 16] = [
    0x29, 0xB7, 0xC9, 0x09, 0x38, 0x3F, 0x84, 0x88, 
    0xFA, 0x98, 0xEC, 0x4E, 0x13, 0x19, 0x79, 0xFB,
];

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReplayMeta {
    pub filtersByShipConfigName: HashMap<String, String>,
    pub clientVersionFromExe: String,
    pub teamNames: Vec<String>,
    pub eventType: String,
    pub gameMode: u32,
    pub isObserver: bool,
    pub clientVersionFromXml: String,
    pub playersPerTeam: u32,
    pub duration: u32,
    pub gameTypeGameParamId: u64,
    pub playerName: String,
    pub mapName: String,
    pub mapBorderName: Option<String>,
    pub scenarioConfigId: u32,
    pub teamsCount: u32,
    pub isFogOfWar: u32,
    pub matchGroup: String,
    pub mapDisplayName: String,
    pub tournamentTag: String,
    pub scenarioUiCategoryId: u32,
    pub mapId: u32,
    pub weatherParams: HashMap<String, Vec<String>>,
    pub spawnLocations: Option<HashMap<String, Vec<f64>>>,
    pub name: String,
    pub scenario: String,
    pub gameType: String,
    pub dateTime: String,
    pub playerID: u32,
    pub disabledShipClasses: Vec<String>,
    pub playerVehicle: String,
    pub battleDuration: u32,
}

fn decode_meta(meta: &[u8]) -> Result<ReplayMeta, Error> {
    let meta = std::str::from_utf8(meta)?;
    debug!("Meta length: {}, content: {:?}", meta.len(), meta);
    let meta: ReplayMeta = serde_json::from_str(meta)?;
    Ok(meta)
}

fn parse_meta(i: &[u8]) -> IResult<&[u8], ReplayMeta> {
    let (i, meta_len) = le_u32(i)?;
    let (i, meta) = take(meta_len)(i)?;
    let meta = match decode_meta(meta) {
        Ok(x) => x,
        Err(e) => {
            return Err(nom::Err::Error(e.into()));
        }
    };
    Ok((i, meta))
}

/// Extra data block added in 12.6.0
fn process_unknown_meta(
    i: &[u8],
    blocks_count: u32
) -> IResult<&[u8], Vec<u8>> {
    let mut json_list: Vec<u8> = Vec::new();
    let mut i = i;
    for _ in 0..blocks_count - 1 {
        let (_i, unknown_len) = le_u32(i)?;
        let (_i, unknown) = take(unknown_len)(_i)?;
        i = _i;
        json_list.extend_from_slice(unknown);
    }

    Ok((i, json_list))
}

fn replay_format(i: &[u8]) -> IResult<&[u8], (ReplayMeta, Vec<u8>)> {
    let (i, header) = take(4usize)(i)?;
    if header != REPLAY_HEADER {
        return Err(failure_from_kind(ErrorKind::InvalidReplayHeader(
            format!("{:?}", header),
        )));
    }

    let (i, unknown_slides_count) = le_u32(i)?;
    let (i, meta) = parse_meta(i)?;
    let (i, unknown) = process_unknown_meta(i, unknown_slides_count)?;
    Ok((i, (meta, unknown)))
}

#[derive(Debug)]
pub struct ReplayFile {
    pub meta: ReplayMeta,
    pub unknown: Vec<u8>,
    pub packet_data: Vec<u8>,
}

impl ReplayFile {
    pub fn from_file(replay: &std::path::PathBuf) -> Result<ReplayFile, ErrorKind> {
        let mut f = std::fs::File::open(replay).unwrap();
        let mut contents = vec![];
        f.read_to_end(&mut contents).unwrap();

        // let (remaining, result) = replay_format(&contents)?;
        let (remaining, (meta, unknown)) = replay_format(&contents)?;

        // Decrypt
        let blowfish: Blowfish = Blowfish::new_from_slice(&BLOWFISH_KEY).unwrap();
        let encrypted = remaining[8..].to_vec();    // skip first chunk
        let mut decrypted = vec![];
        decrypted.resize(encrypted.len(), 0u8);
        let mut previous: [u8; 8] = [0; 8];

        let encrypted_chunks = encrypted.chunks(8);
        let decrypted_chunks = decrypted.chunks_mut(8);
        for (in_block, out_block) in encrypted_chunks.zip(decrypted_chunks) {
            blowfish.decrypt_block_b2b(in_block.into(), out_block.into());
            for (i, byte) in out_block.iter_mut().enumerate() {
                *byte ^= previous[i];
            }
            previous.copy_from_slice(out_block);
        }

        let mut deflater = flate2::read::ZlibDecoder::new(&decrypted[..]);
        let mut contents = vec![];
        deflater.read_to_end(&mut contents).unwrap();

        Ok(ReplayFile {
            meta: meta,
            unknown: unknown,
            packet_data: contents,
        })
    }
}