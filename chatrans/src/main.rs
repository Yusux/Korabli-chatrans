use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber;

use chatrans::parser::parse_live_chat;

mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}


#[derive(Parser)]
#[command(
    name="Chatrans",
    author="Yusux",
    version=built_info::PKG_VERSION,
    about="Translates chat from Korabli via the replay file",
    long_about=None,
    next_line_help = true,
)]
struct Client {
    #[arg(short, long, help = "The replay dir to use")]
    replay_dir: String,
    #[arg(short,long, help = "The target language, where `cn` stands for Chinese, `en` stands for English. Default is `cn`", default_value = "cn")]
    target_language: String,
}


fn main() {
    let _collector = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();

    let client = Client::parse();
    let input = client.replay_dir;

    info!("Parsing live chat from replay dir: {:?}", input);
    let _ = parse_live_chat(&std::path::PathBuf::from(input));
}
