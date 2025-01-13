use crate::messages::{decode_message, encode_message, ProtocolMessage};
use anyhow::bail;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug)]
pub(crate) struct PeerHandle {
    /// Stream to communicate with the peer
    pub(crate) stream: TcpStream,
    /// Channel to send messages to the server main loop
    to_main_loop: mpsc::Sender<ProtocolMessage>,
    /// Channel to receive messages from the server main loop
    from_main_loop: mpsc::Receiver<ProtocolMessage>,
}

impl PeerHandle {
    pub(crate) fn new(
        stream: TcpStream,
        to_main_loop: mpsc::Sender<ProtocolMessage>,
        from_main_loop: mpsc::Receiver<ProtocolMessage>,
    ) -> Self {
        Self {
            stream,
            to_main_loop,
            from_main_loop,
        }
    }

    pub(crate) async fn listen_messages(mut self) -> anyhow::Result<()> {
        loop {
            let mut read_buffer = [0; 4096];
            tokio::select! {
                message = self.stream.read(&mut read_buffer) => {
                    let bytes_read = message?;
                    if bytes_read == 0 {
                        return Ok(());
                    }

                    let protocol_message = decode_message(&read_buffer[..bytes_read])?;
                    info!("Received message {protocol_message:?}");

                    self.to_main_loop.send(protocol_message).await?;
                }
                message = self.from_main_loop.recv() => {
                    match message {
                        Some(message) => {
                            self.send_message(message).await?;
                        }
                        _ => {
                            bail!("Unknown message from main loop: {:?}", message);
                        }
                    }
                }
            }
        }
    }

    pub(crate) async fn receive_message(&mut self) -> anyhow::Result<ProtocolMessage> {
        let mut buffer = [0; 4096];
        let bytes_read = self.stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            bail!("Peer disconnected");
        }

        let protocol_message = decode_message(&buffer[..bytes_read])?;
        Ok(protocol_message)
    }

    pub(crate) async fn send_message(&mut self, message: ProtocolMessage) -> anyhow::Result<()> {
        let message = encode_message(message);
        self.stream.write_all(message.as_bytes()).await?;
        self.stream.flush().await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PeerInfo {
    pub(crate) address: SocketAddr,
    pub(crate) _to_peer: mpsc::Sender<ProtocolMessage>,
}

impl PeerInfo {
    pub(crate) fn new(address: SocketAddr, _to_peer: mpsc::Sender<ProtocolMessage>) -> Self {
        Self { address, _to_peer }
    }
}
