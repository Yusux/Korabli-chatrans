use clap::Parser;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::{info, Level};
use tracing_subscriber;

use chatrans::live::LiveMonitor;
use chatrans::processor::ChatMessage;
use chatrans::server::WebSocketServer;

#[derive(Parser)]
#[command(
    name="Chatrans",
    author="Yusux",
    version=concat!("v", env!("CARGO_PKG_VERSION")),
    about="Translates chat from Korabli via the replay file",
    long_about=None,
    next_line_help = true,
)]
struct Client {
    #[arg(short, long, help = "The replay dir to use")]
    replay_dir: String,
    #[arg(short, long, help = "The target language, where `zh` stands for Chinese, `en` stands for English. Default is `zh`", default_value = "zh")]
    target_language: String,
    #[arg(short, long, help = "The server ip to use", default_value = "0.0.0.0")]
    ip: String,
    #[arg(short, long, help = "The server port to use", default_value = "38080")]
    port: u16,
    #[arg(long, help = "The Aliyun access key id")]
    access_key_id: Option<String>,
    #[arg(long, help = "The Aliyun access key secret")]
    access_key_secret: Option<String>,
}


fn main() {
    let _collector = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let client = Client::parse();
    let input = client.replay_dir;

    info!("Parsing live chat from replay dir: {:?}", input);
    info!("Target language: {:?}", client.target_language);
    if client.access_key_id.is_some() && client.access_key_secret.is_some() {
        info!("Aliyun API key provided, using Aliyun API for translation");
    } else {
        info!("No Aliyun API key provided, no translation will be done");
    }
    info!("Use `Ctrl+C` to stop the program");

    // create a channel to pass the chat messages
    let (tx, rx) = async_channel::bounded::<ChatMessage>(128);

    // create CancellationToken
    let token = CancellationToken::new();

    // start the monitor
    let monitor = LiveMonitor::new(input, tx);
    let token_clone = token.clone();
    let monitor_thread = std::thread::spawn(move || {
        monitor.run(token_clone);
    });

    // start the websocket server
    let server = WebSocketServer::new(
        client.ip,
        client.port,
        client.target_language, 
        client.access_key_id,
        client.access_key_secret,
        rx,
    );
    let token_clone = token.clone();
    let server_handle = std::thread::spawn(move || {
        server.run(token_clone);
    });

    // wait for the interrupt signal
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            match signal::ctrl_c().await {
                Ok(()) => {},
                Err(err) => {
                    eprintln!("Unable to listen for shutdown signal: {}", err);
                },
            }
        });

    info!("SIGINT received, shutting down monitor and server");
    token.cancel();
    monitor_thread.join().unwrap();
    server_handle.join().unwrap();
}
