use std::{
    io::Error as IoError,
    net::SocketAddr,
};
use tokio::net::{TcpListener, TcpStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::processor::ChatMessage;
use crate::interpreter::Interpreter;

pub struct WebSocketServer {
    ip: String,
    port: u16,
    langeuage: String,
    access_key_id: Option<String>,
    access_key_secret: Option<String>,
    message_rx: async_channel::Receiver<ChatMessage>,
}

impl WebSocketServer {
    pub fn new(
        ip: String,
        port: u16,
        langeuage: String,
        access_key_id: Option<String>,
        access_key_secret: Option<String>,
        message_rx: async_channel::Receiver<ChatMessage>,
    ) -> WebSocketServer {
        WebSocketServer {
            ip,
            port,
            langeuage,
            access_key_id,
            access_key_secret,
            message_rx,
        }
    }

    async fn process_message(
        interpreter: &Interpreter,
        message: ChatMessage,
    ) -> String {
        let translated = interpreter.translate(message.message.clone()).await;
        // "[Time] Sender to Audience: Translated |Original|"
        format!(
            "[{:4.2}s] {:^30} to {:^30}: {} |{}|",
            message.clock,
            message.sender,
            message.audience,
            translated,
            message.message,
        )
    }

    async fn handle_connection(
        raw_stream: TcpStream,
        addr: SocketAddr,
        mut broadcast_rx: async_broadcast::Receiver<String>,
    ) {
        info!("WebSocket incoming TCP connection from: {}", addr);
    
        let ws_stream = tokio_tungstenite::accept_async(raw_stream)
            .await
            .expect("Error during the websocket handshake occurred");
        info!("WebSocket connection established: {}", addr);
    
        let (mut writer, _reader) = ws_stream.split();
    
        loop {
            info!("Waiting for broadcast message");
            let msg = broadcast_rx.recv().await;
            debug!("Received broadcast message: {:?}", msg);
            match msg {
                Ok(msg) => {
                    let msg = Message::Text(msg);
                    debug!("Sending message: {:?}", msg);
                    match writer.send(msg).await {
                        Ok(_) => {
                            debug!("Sent message successfully");
                        }
                        Err(e) => {
                            info!("Error sending message when handling connection: {:?}", e);
                            break;
                        }
                    }
                }
                Err(e) => {
                    info!("Error receiving message when handling connection: {:?}", e);
                    break;
                }
            }
        }
    
        info!("WebSocket connection closed: {}", addr);
    }
    
    async fn server(
        ip: String,
        port: u16,
        langeuage: String,
        access_key_id: Option<String>,
        access_key_secret: Option<String>,
        message_rx: async_channel::Receiver<ChatMessage>,
    ) -> Result<(), IoError> {
        let addr = format!("{}:{}", ip, port);
    
        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect("Failed to bind");
        info!("Listening on: {}", addr);
    
        // spawn a task to listen for incoming messages and broadcast them
        let (broadcast_tx, broadcast_rx) = async_broadcast::broadcast(128);
        let interpreter = Interpreter::new(
            langeuage,
            access_key_id,
            access_key_secret,
        );
        let message_repeater = tokio::spawn(async move {
            loop {
                info!("Waiting for message");
                let message = message_rx.recv().await;
                match message {
                    Ok(msg) => {
                        let msg = Self::process_message(&interpreter, msg).await;
                        debug!("Broadcasting message: {:?}", msg);
                        match broadcast_tx.broadcast(msg).await {
                            Ok(ok) => {
                                debug!("Broadcasted message: {:?}, listeners: {}", ok, broadcast_tx.receiver_count());
                            }
                            Err(e) => {
                                error!("Error broadcasting message: {:?}", e);
                            }
                        }
                    }
                    _ => {
                        break;
                    }
                }
            }
        });
    
        // Let's spawn the handling of each connection in a separate task.
        let messgae_handler = tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let broadcast_rx = broadcast_rx.clone();
                tokio::spawn(async move {
                    Self::handle_connection(stream, addr, broadcast_rx).await;
                });
            }
        });
    
        message_repeater.await.unwrap();
        messgae_handler.await.unwrap();
    
        Ok(())
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
                _ = Self::server(
                    self.ip.clone(),
                    self.port,
                    self.langeuage.clone(),
                    self.access_key_id.clone(),
                    self.access_key_secret.clone(),
                    self.message_rx.clone()
                ) => {}
            }
        });

        info!("WebSocket server is stopped");
    }
}