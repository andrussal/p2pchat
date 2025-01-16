mod messages;
mod overwatch;
mod peer;
mod server;

use clap::Parser;
use overwatch::services::{ServerService, ServerSettings};
use overwatch_derive::Services;
use overwatch_rs::overwatch::OverwatchRunner;
use overwatch_rs::services::handle::ServiceHandle;

#[derive(Parser)]
struct Args {
    address: String,
    known_peer: Option<String>,
}

#[derive(Services)]
pub(crate) struct P2pChat {
    peers: ServiceHandle<ServerService>,
}

fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    let settings = P2pChatServiceSettings {
        peers: ServerSettings {
            address: args.address,
            known_peer: args.known_peer,
        },
    };

    let p2p_chat = OverwatchRunner::<P2pChat>::run(settings, None).expect("OverwatchRunner failed");
    p2p_chat.wait_finished();

    Ok(())
}
