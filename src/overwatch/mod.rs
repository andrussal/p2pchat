use crate::messages::ProtocolMessage;
use overwatch_rs::services::relay::RelayMessage;

pub(crate) mod operators;
pub(crate) mod services;
pub(crate) mod states;

impl RelayMessage for ProtocolMessage {}
