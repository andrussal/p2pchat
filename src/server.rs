use crate::messages::ProtocolMessage;
use crate::peer::{PeerHandle, PeerInfo};
use anyhow::bail;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Handle;
use tokio::sync::mpsc;
use tracing::{debug, info};

pub(crate) struct Server {
    /// Address of the server
    address: SocketAddr,
    /// List of connected peers
    known_peers: Vec<PeerInfo>,
    /// Channel passed to peers to send messages to the server
    to_server: mpsc::Sender<ProtocolMessage>,
    /// Channel to receive messages from peers
    from_peers: mpsc::Receiver<ProtocolMessage>,
    /// Channel to notify when peer disconnected
    to_peer_disconnected: mpsc::Sender<SocketAddr>,
    /// Channel to receive notification when peer disconnected
    rcv_peer_disconnected: mpsc::Receiver<SocketAddr>,
    /// Current Runtime
    runtime_handle: Handle,
}

impl Server {
    pub(crate) fn new(address: &str, runtime_handle: Handle) -> anyhow::Result<Self> {
        let address = address.parse()?;
        let (to_server, from_peers) = mpsc::channel(32);
        let (to_peer_disconnected, rcv_peer_disconnected) = mpsc::channel(32);
        Ok(Self {
            address,
            known_peers: vec![],
            to_server,
            from_peers,
            to_peer_disconnected,
            rcv_peer_disconnected,
            runtime_handle,
        })
    }

    /// Bootstraps the server from a known peer. Known peer is a peer that is already connected to the network.
    /// It allows us to fetch the list of other peers connected to the network.
    pub(crate) async fn bootstrap_from_known_peer(
        &mut self,
        known_peer: SocketAddr,
    ) -> anyhow::Result<()> {
        info!(?known_peer, "Bootstrapping from known peer");

        let (peer, know_peers, info) = self.connect_to_peer(known_peer).await?;
        self.known_peers.push(info);

        self.spawn_peer_listener(peer);

        self.connect_to_peers(&know_peers).await?;

        Ok(())
    }

    /// Listens for incoming connections and messages from peers.
    pub(crate) async fn listen(mut self, address: &str) -> anyhow::Result<()> {
        info!(?address, "Listening on");

        let connection = TcpListener::bind(address).await?;

        loop {
            tokio::select! {
                Ok((stream, _)) = connection.accept() => {
                    let (peer, _, info) = self.perform_handshake(stream, false).await?;
                    info!(?info.address, "Peer connected");
                    self.known_peers.push(info);

                    self.spawn_peer_listener(peer);
                }
                Some(message) = self.from_peers.recv() => {
                    info!("Broadcasting user message to all peers: {message:?}");
                    // TODO: display user message
                }
                Some(disconnected_peer) = self.rcv_peer_disconnected.recv() => {
                    info!("Peer disconnected: {:?}", disconnected_peer);
                    self.known_peers.retain(|peer| peer.address != disconnected_peer);
                }
                // TODO: add stdio listener to send ProtocolMessage::ClientMessage to all peers
            }
        }
    }

    async fn connect_to_peers(&mut self, peers_list: &[SocketAddr]) -> anyhow::Result<()> {
        for peer_address in peers_list {
            if peer_address == &self.address {
                continue;
            }

            match self.connect_to_peer(*peer_address).await {
                Ok((peer, _, info)) => {
                    info!(?info.address, "Connected to peer");
                    self.spawn_peer_listener(peer);
                    self.known_peers.push(info);
                }
                Err(err) => {
                    info!(?peer_address, "Failed to connect to peer: {err:?}");
                }
            }
        }
        Ok(())
    }

    async fn connect_to_peer(
        &mut self,
        peer_address: SocketAddr,
    ) -> anyhow::Result<(PeerHandle, Vec<SocketAddr>, PeerInfo)> {
        info!(?peer_address, "Connecting to peer");
        match TcpStream::connect(peer_address).await {
            Ok(stream) => {
                let (peer, known_peers, info) = self.perform_handshake(stream, true).await?;
                Ok((peer, known_peers, info))
            }
            Err(err) => {
                bail!("Failed to connect to peer: {err:?}");
            }
        }
    }

    /// Performs handshake with the peer. In this system handshake is very simple.
    ///
    /// Initiator sends `HELLO` message with its listening address and list of known peers.
    /// Receiver replies with `HELLO` message with its listening address and list of known peers.
    async fn perform_handshake(
        &mut self,
        stream: TcpStream,
        initiator: bool,
    ) -> anyhow::Result<(PeerHandle, Vec<SocketAddr>, PeerInfo)> {
        let know_peers = self
            .known_peers
            .iter()
            .map(|peer| peer.address)
            .collect::<Vec<_>>();

        let (to_peer, from_main_loop) = mpsc::channel(32);
        let mut peer = PeerHandle::new(stream, self.to_server.clone(), from_main_loop);

        let (peer_address, known_peers) = if initiator {
            self.send_hello(&mut peer, &know_peers).await?;
            self.receive_hello(&mut peer).await?
        } else {
            let (peer_address, known_peers) = self.receive_hello(&mut peer).await?;
            self.send_hello(&mut peer, &know_peers).await?;
            (peer_address, known_peers)
        };

        let peer_info = PeerInfo::new(peer_address, to_peer);

        Ok((peer, known_peers, peer_info))
    }

    /// Starts new task to communicate with the peer.
    fn spawn_peer_listener(&self, peer: PeerHandle) {
        let send_disconnect = self.to_peer_disconnected.clone();
        self.runtime_handle.spawn(async move {
            let peer_address = peer.stream.peer_addr().unwrap();
            if let Err(e) = peer.listen_messages().await {
                info!(?peer_address, "Peer error: {e:?}");
            }
            send_disconnect.send(peer_address).await.unwrap();
        });
    }

    async fn send_hello(
        &mut self,
        peer: &mut PeerHandle,
        peers_list: &[SocketAddr],
    ) -> anyhow::Result<()> {
        let peer_address = peer.stream.peer_addr()?;
        info!(?peer_address, "Sending HELLO to peer",);
        let hello_message = ProtocolMessage::Hello(self.address, peers_list.to_vec());

        peer.send_message(hello_message).await?;

        Ok(())
    }

    async fn receive_hello(
        &mut self,
        peer: &mut PeerHandle,
    ) -> anyhow::Result<(SocketAddr, Vec<SocketAddr>)> {
        let response = peer.receive_message().await?;
        debug!("Received response: {response:?}");

        let ProtocolMessage::Hello(peer_address, peer_list) = response else {
            bail!("Expected Hello message from known peer, got: {response:?}");
        };

        info!(?peer_address, "Received HELLO from peer");

        Ok((peer_address, peer_list))
    }
}
