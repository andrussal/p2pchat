mod messages;
mod peer;
mod server;

use crate::server::Server;
use clap::Parser;

#[derive(Parser)]
struct Args {
    address: String,
    known_peer: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    tracing::info!("Starting server on: {}", args.address);

    let mut server = Server::new(&args.address)?;

    if let Some(known_peer) = args.known_peer {
        let known_peer = known_peer.parse()?;
        server.bootstrap_from_known_peer(known_peer).await?;
    }

    server.listen(&args.address).await?;

    Ok(())
}
