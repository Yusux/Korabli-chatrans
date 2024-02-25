use std::{
    fs::{read_to_string, remove_file, File},
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
use tracing::{debug, info, warn};

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
        let temp_replay = self.replay_dir.join("temp.wowsreplay");
        let info_json = self.replay_dir.join("tempArenaInfo.json");
        info!("Parsing live chat from temp replay: {:?}", temp_replay);
        info!("Parsing live chat from json: {:?}", info_json);
    
        // Check if the file exists
        if !temp_replay.exists() && !info_json.exists() {
            return Err(anyhow!("Temp Replay file not found"));
        }
    
        // Get meta from tempArenaInfo.json
        let meta = Self::get_meta(&info_json)?;
        debug!("Meta: {:?}", meta);
    
        // Check if the version is valid
        let version_parts: Vec<_> = meta.clientVersionFromExe.split(",").collect();
        debug!("Version parts: {:?}", version_parts);
        if version_parts.len() != 4 {
            return Err(anyhow!("Invalid version"));
        }
    
        // Get datafiles and specs with the version
        let datafiles = replay_parser::version::Datafiles::new(
            PathBuf::from("versions"),
            replay_parser::version::Version::from_client_exe(&meta.clientVersionFromExe),
        )?;
        let specs = parse_scripts(&datafiles)?;
    
        // Assign processor and parser
        let chatlogger = ChatLoggerBuilder::new();
        let processor = chatlogger.build(&meta, self.tx.clone());
        let mut analyzer_set = replay_parser::analyzer::AnalyzerAdapter::new(vec![processor]);
        let mut p = Parser::new(&specs);
    
        // Monitor the change to the temp.wowsreplay file
        // Open the file
        let mut file = File::open(&temp_replay)?;
        // Create a buffer using Vec<u8>
        let mut buffer: Vec<u8> = Vec::new();
        // Current file offset
        let mut offset = 0;
        loop {
            // Read the file from the current offset to the end 
            file.seek(SeekFrom::Start(offset as u64))?;
            debug!("Offset: {:?}, File size: {:?}", offset, file.metadata()?.len());
            offset += file.read_to_end(&mut buffer)? as u64;
    
            // Parse the packets
            let parsed_bytes = p.parse_buffer(&buffer, &mut analyzer_set)? as usize;
            debug!("Parsed bytes number: {:?}", parsed_bytes);
            buffer.drain(0..parsed_bytes);
    
            // Determine whether to continue
            if !info_json.exists() {
                info!("tempArenaInfo.json not found, waiting for the next game");
                break;
            }
            
            // Sleep for 2 seconds
            sleep(Duration::from_secs(2)).await;
        }
    
        info!("Parsing live chat from temp replay done");
        
        Ok(())
    }

    async fn monitor(&self) -> Result<()> {
        let info_json = self.replay_dir.join("tempArenaInfo.json");
        info!("Parsing live chat from json: {:?}", info_json);

        loop {
            // Check if the file has been created
            if !info_json.exists() {
                // Create a watcher
                let (watcher_tx, watcher_rx) = async_channel::bounded(1);
                let mut watcher = RecommendedWatcher::new(
                    move |res| {
                        match watcher_tx.send_blocking(res) {
                            Ok(_) => {}
                            Err(e) => {
                                warn!("Error sending watcher event: {:?}", e);
                            }
                        }
                    },
                    Config::default(),
                )?;

                // Watch the file
                watcher.watch(
                    &self.replay_dir,
                    RecursiveMode::NonRecursive,
                )?;

                loop {
                    match watcher_rx.recv().await {
                        Ok(Ok(event)) => {
                            debug!("File event: {:?}", event);
                            // Check if create on the file
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

            // Parse live chat
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

    fn clean(&self) -> Result<()> {
        // Remove all files ending with `.temp`
        let files = self.replay_dir.read_dir()?;
        for file in files {
            let file = file?;
            let path = file.path();
            if let Some(ext) = path.extension() {
                if ext == "temp" {
                    debug!("Removing temp file: {:?}", path);
                    remove_file(path)?;
                }
            }
        }
        Ok(())
    }

    pub fn run(&self, token: CancellationToken) {
        // Start the websocket server
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async move {
            tokio::select! {
                // Using cloned token to listen to cancellation requests
                _ = token.cancelled() => {}
                _ = self.monitor() => {}
            }
        });

        // Clean up the temp files
        match self.clean() {
            Ok(_) => {
                info!("Temp files are cleaned");
            }
            Err(e) => {
                warn!("Error cleaning temp files: {:?}", e);
            }
        }

        info!("Live monitor is stopped");
    }
}
