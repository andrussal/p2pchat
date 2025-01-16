use crate::overwatch::states::PeersState;
use overwatch_rs::services::state::{ServiceState, StateOperator};
use tracing::debug;

#[derive(Debug, Clone)]
pub(crate) struct PeersStateOperator;

#[async_trait::async_trait]
impl StateOperator for PeersStateOperator {
    type StateInput = PeersState;

    fn from_settings(_settings: <Self::StateInput as ServiceState>::Settings) -> Self {
        Self
    }

    async fn run(&mut self, _state: Self::StateInput) {
        debug!("PeersStateOperator::run");
    }
}
