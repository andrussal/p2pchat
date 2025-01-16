# Simple p2p chat application

## Description

This is a simple p2p chat application that doesn't have a central server that coordinates communication. Instead, each
node sends messages to all other peers.

Currently only nodes connecting to all other nodes is implemented. Sending user messages is not yet done.

## TODO

- [ ] Allow server to accept user messages from command line and broadcast them to all other nodes.
  - [ ] Implement stdio reader as OverWatch service
- [ ] Instead of all nodes connecting with every other node, use gossip protocol to connect with a subset of nodes.
- [ ] Use tokio Frame to encode/decode messages. See [tokio documentation](https://tokio.rs/tokio/tutorial/framing) for
  more information.
- [ ] Use Serde to serialise/deserialise messages.
- [ ] Use a standard serialisation format like JSON or Protobuf.