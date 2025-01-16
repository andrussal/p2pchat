# Simple p2p chat application

## Description

This is a simple p2p chat application that doesn't have a central server that coordinates communication. Instead, each
node sends messages to all other peers.

Currently only nodes connecting to all other nodes is implemented. Sending user messages is not yet done.

## Design overview

The application main component is **Server** which listens connections from other p2p peers. Currently, all server
instances connect to all other server instances. A server gets full list of network peers from a bootstrap server(if
bootstrap server command line argument is provided).

At startup peers follow a very simple negotiation protocol. Connecting peer sends HELLO message to the (bootstrap)
server. That server
responds also with HELLO message which also includes all other peers that are connected to it.

## How to run

Any already running server can be used as a bootstrap server given that it has obtained list of all other peers.

1. Start a (bootstrap) server
    ```shell
    cargo run 127.0.0.1:9000
    ```
2. Start another server that connects to the (bootstrap) server
    ```shell
    cargo run 127.0.0.1:9001 127.0.0.1:9000
    ```
3. Start another server that connects to any of the previous servers(both previous servers have the full list of peers,
   in this case local processes running on ports 9000 and 9001)
    ```shell
   cargo run 127.0.0.1:9002 127.0.0.1:9001
    ```

## TODO

- [ ] Allow server to accept user messages from command line and broadcast them to all other nodes.
    - [ ] Implement stdio reader as [Overwatch](https://github.com/logos-co/Overwatch) service
- [ ] Instead of all nodes connecting with every other node, use gossip protocol to connect with a subset of nodes.
- [ ] Use tokio Frame to encode/decode messages. See [tokio documentation](https://tokio.rs/tokio/tutorial/framing) for
  more information.
- [ ] Use Serde to serialise/deserialize messages.
- [ ] Use a standard serialisation format like JSON or Protobuf.