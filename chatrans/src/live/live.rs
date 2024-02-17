use std::{
    fs::{read_to_string, File},
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
};
use anyhow::{Result, anyhow};
use notify::{
    event::{CreateKind, EventKind},
    Config,
    RecommendedWatcher,
    RecursiveMode,
    Watcher,
};
use async_channel::Sender;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::processor::{ChatMessage, ChatLoggerBuilder};

use replay_parser::{
    parse_scripts,
    ReplayMeta,
    packet2::Parser,
};

pub struct LiveMonitor {
    replay_dir: PathBuf,
    tx: Sender<ChatMessage>,
}

impl LiveMonitor {
    pub fn new(replay_dir: String, tx: Sender<ChatMessage>) -> LiveMonitor {
        LiveMonitor {
            replay_dir: PathBuf::from(replay_dir),
            tx,
        }
    }

    fn get_meta(
        info_json : &PathBuf
    ) -> Result<ReplayMeta> {
        let info_json = read_to_string(info_json)?;
        let meta: ReplayMeta = serde_json::from_str(&info_json)?;
        Ok(meta)
    }

    async fn parse_live_chat(&self) -> Result<()> {
        // let replay_file = ReplayFile::from_file(replay)?;
        let temp_replay = self.replay_dir.join("temp.wowsreplay");
        let info_json = self.replay_dir.join("tempArenaInfo.json");
        info!("Parsing live chat from temp replay: {:?}", temp_replay);
        info!("Parsing live chat from json: {:?}", info_json);
    
        // check if the file exists
        if !temp_replay.exists() && !info_json.exists() {
            return Err(anyhow!("Temp Replay file not found"));
        }
    
        // get meta from tempArenaInfo.json
        let meta = Self::get_meta(&info_json)?;
        debug!("Meta: {:?}", meta);
    
        // check if the version is valid
        let version_parts: Vec<_> = meta.clientVersionFromExe.split(",").collect();
        debug!("Version parts: {:?}", version_parts);
        if version_parts.len() != 4 {
            return Err(anyhow!("Invalid version"));
        }
    
        // get datafiles and specs with the version
        let datafiles = replay_parser::version::Datafiles::new(
            PathBuf::from("versions"),
            replay_parser::version::Version::from_client_exe(&meta.clientVersionFromExe),
        )?;
        let specs = parse_scripts(&datafiles)?;
    
        // assign processor and parser
        let chatlogger = ChatLoggerBuilder::new();
        let processor = chatlogger.build(&meta, self.tx.clone());
        let mut analyzer_set = replay_parser::analyzer::AnalyzerAdapter::new(vec![processor]);
        let mut p = Parser::new(&specs);
    
        // monitor the change to the temp.wowsreplay file
        // open the file
        let mut file = File::open(&temp_replay)?;
        // create a buffer using Vec<u8>
        let mut buffer: Vec<u8> = Vec::new();
        // current file offset
        let mut offset = 0;
        loop {
            // read the file from the current offset to the end 
            file.seek(SeekFrom::Start(offset as u64))?;
            debug!("Offset: {:?}, File size: {:?}", offset, file.metadata()?.len());
            offset += file.read_to_end(&mut buffer)? as u64;
    
            // parse the packets
            let parsed_bytes = p.parse_buffer(&buffer, &mut analyzer_set)? as usize;
            debug!("Parsed bytes number: {:?}", parsed_bytes);
            buffer.drain(0..parsed_bytes);
    
            // determine whether to continue
            if !info_json.exists() {
                info!("tempArenaInfo.json not found, waiting for the next game");
                break;
            }
            
            // sleep for 2 seconds
            sleep(Duration::from_secs(2)).await;
        }
    
        info!("Parsing live chat from temp replay done");
        
        Ok(())
    }

    async fn monitor(&self) -> Result<()> {
        let info_json = self.replay_dir.join("tempArenaInfo.json");
        info!("Parsing live chat from json: {:?}", info_json);

        loop {
            // check if the file has been created
            if !info_json.exists() {
                // create a watcher
                let (watcher_tx, watcher_rx) = async_channel::bounded(1);
                let mut watcher = RecommendedWatcher::new(
                    move |res| {
                        watcher_tx.send_blocking(res).unwrap();
                    },
                    Config::default(),
                )?;

                // watch the file
                watcher.watch(
                    &self.replay_dir,
                    RecursiveMode::NonRecursive,
                )?;

                loop {
                    match watcher_rx.recv().await {
                        Ok(Ok(event)) => {
                            debug!("File event: {:?}", event);
                            // is create on the file
                            if event.kind == EventKind::Create(CreateKind::Any) && event.paths[0] == info_json {
                                info!("Entering the game");
                                break;
                            }
                        }
                        _ => {
                            debug!("Watcher error");
                        }
                    }
                }
            }

            // parse live chat
            match self.parse_live_chat().await {
                Ok(_) => {
                    info!("The game has ended, waiting for the next game");
                }
                Err(e) => {
                    info!("Error parsing live chat: {:?}", e);
                }
            }
        }
    }

    pub fn run(&self, token: CancellationToken) {
        // start the websocket server
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async move {
            tokio::select! {
                // Step 3: Using cloned token to listen to cancellation requests
                _ = token.cancelled() => {}
                _ = self.monitor() => {}
            }
        });

        info!("Live monitor is stopped");
    }
}
