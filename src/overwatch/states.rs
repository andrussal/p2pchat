use crate::overwatch::services::ServerSettings;
use overwatch_rs::services::state::ServiceState;

#[derive(thiserror::Error, Debug)]
pub(crate) enum PeersStateError {}

#[derive(Clone)]
pub(crate) struct PeersState;

impl ServiceState for PeersState {
    type Settings = ServerSettings;
    type Error = PeersStateError;

    fn from_settings(_settings: &Self::Settings) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
