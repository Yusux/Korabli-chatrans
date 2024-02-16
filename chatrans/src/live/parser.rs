use std::{
    fs::{read_to_string, File},
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    // sync::mpsc,
    thread::sleep,
    time::Duration,
};
use anyhow::{Result, anyhow};
// use notify::{Watcher, recommended_watcher, RecursiveMode};
use serde_json;
use tracing::{info, debug};


use replay_parser::{
    parse_scripts,
    ReplayMeta,
    analyzer::{
        chat::ChatLoggerBuilder,
        AnalyzerBuilder,
    },
    packet2::Parser,
};

fn get_meta(
    info_json : &PathBuf
) -> Result<ReplayMeta> {
    let info_json = read_to_string(info_json)?;
    let meta: ReplayMeta = serde_json::from_str(&info_json)?;
    Ok(meta)
}

pub fn parse_live_chat(
    replay_dir: &PathBuf,
) -> Result<()> {
    // let replay_file = ReplayFile::from_file(replay)?;
    let temp_replay = replay_dir.join("temp.wowsreplay");
    let info_json = replay_dir.join("tempArenaInfo.json");
    info!("Parsing live chat from temp replay: {:?}", temp_replay);
    info!("Parsing live chat from json: {:?}", info_json);

    // check if the file exists
    if !temp_replay.exists() && !info_json.exists() {
        return Err(anyhow!("Temp Replay file not found"));
    }

    // get meta from tempArenaInfo.json
    let meta = get_meta(&info_json)?;
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
    let processor = chatlogger.build(&meta);
    let mut analyzer_set = replay_parser::analyzer::AnalyzerAdapter::new(vec![processor]);
    let mut p = Parser::new(&specs);

    // monitor the change to the temp.wowsreplay file
    // open the file
    let mut file = File::open(&temp_replay)?;
    // create a buffer using Vec<u8>
    let mut buffer: Vec<u8> = Vec::new();
    // current file offset
    let mut offset = 0;
    // create a watcher
    // let (tx, rx) = mpsc::channel();
    // let mut watcher = recommended_watcher(move |res| tx.send(res).unwrap())?;
    // watcher.watch(
    //     &temp_replay,
    //     RecursiveMode::NonRecursive,
    // )?;
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
            break;
        }

        // check if the file has been modified
        // loop {
        //     match rx.recv_timeout(Duration::from_secs(2)) {
        //         Ok(event) => {
        //             sleep(Duration::from_millis(500));
        //             // receive all events
        //             while let Ok(_) = rx.try_recv() {}
        //             debug!("File event: {:?}", event);
        //             break;
        //         }
        //         _ => {
        //             debug!("No file event");
        //             continue;
        //         }
        //     }
        // }
        sleep(Duration::from_secs(2));
    }

    info!("Parsing live chat from temp replay done");
    
    Ok(())
}
