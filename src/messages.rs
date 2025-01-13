use anyhow::bail;
use std::net::SocketAddr;
use tracing::trace;

const HELLO_PREFIX: &str = "Hello";
const PEER_LIST_PREFIX: &str = "PeerList";
const CLIENT_MESSAGE_PREFIX: &str = "ClientMessage";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProtocolMessage {
    Hello(SocketAddr, Vec<SocketAddr>),
    PeerList(Vec<String>),
    ClientMessage(String),
}

//TODO: Potentially use https://tokio.rs/tokio/tutorial/framing instead of this
pub(crate) fn encode_message(message: ProtocolMessage) -> String {
    match message {
        ProtocolMessage::Hello(peer_address, peer_list) => {
            let peer_list = peer_list
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(",");
            format!("{HELLO_PREFIX}{peer_address},{peer_list}")
        }
        ProtocolMessage::PeerList(peer_list) => {
            let peer_list = peer_list.join(",");
            format!("{PEER_LIST_PREFIX}{peer_list}")
        }
        ProtocolMessage::ClientMessage(message) => {
            format!("{CLIENT_MESSAGE_PREFIX}{message}")
        }
    }
}
pub(crate) fn decode_message(data: &[u8]) -> anyhow::Result<ProtocolMessage> {
    let message = String::from_utf8_lossy(data);

    let protocol_message = if message.starts_with(HELLO_PREFIX) {
        decode_hello(&message)?
    } else if message.starts_with(PEER_LIST_PREFIX) {
        decode_peer_list(&message)
    } else if let Some(message) = message.strip_prefix(CLIENT_MESSAGE_PREFIX) {
        ProtocolMessage::ClientMessage(message.to_string())
    } else {
        bail!("Unknown message from peer: {:?}", message);
    };

    Ok(protocol_message)
}

fn decode_hello(message: &str) -> anyhow::Result<ProtocolMessage> {
    let rest = &message[HELLO_PREFIX.len()..]
        .split(',')
        .collect::<Vec<&str>>();

    let peer_address = rest[0].parse::<SocketAddr>()?;

    let mut peer_list = vec![];
    if !rest.is_empty() {
        peer_list = rest[1..]
            .iter()
            .filter_map(|s| {
                if let Ok(addr) = s.parse::<SocketAddr>() {
                    Some(addr)
                } else {
                    trace!("Invalid address in peer list: {s:?}");
                    None
                }
            })
            .collect();
    }
    Ok(ProtocolMessage::Hello(peer_address, peer_list))
}

fn decode_peer_list(message: &str) -> ProtocolMessage {
    let peer_list = message[PEER_LIST_PREFIX.len()..]
        .split(",")
        .map(ToString::to_string)
        .collect();
    ProtocolMessage::PeerList(peer_list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_encode_hello() {
        let peer_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let peer_list = vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082),
        ];

        let message = ProtocolMessage::Hello(peer_address, peer_list);
        let encoded = encode_message(message);
        assert_eq!(encoded, "Hello127.0.0.1:8080,127.0.0.1:8081,127.0.0.1:8082");
    }

    #[test]
    fn test_decode_hello() {
        let encoded = "Hello127.0.0.1:8080,127.0.0.1:8081,127.0.0.1:8082";
        let decoded = decode_message(encoded.as_bytes()).unwrap();

        let peer_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let peer_list = vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082),
        ];

        assert_eq!(decoded, ProtocolMessage::Hello(peer_address, peer_list));
    }
}
