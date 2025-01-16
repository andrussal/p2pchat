use crate::messages::ProtocolMessage;
use crate::overwatch::operators::PeersStateOperator;
use crate::overwatch::states::PeersState;
use crate::server::Server;
use overwatch_rs::services::handle::ServiceStateHandle;
use overwatch_rs::services::{ServiceCore, ServiceData, ServiceId};
use overwatch_rs::DynError;

#[derive(Debug, Clone)]
pub(crate) struct ServerSettings {
    pub(crate) address: String,

    pub(crate) known_peer: Option<String>,
}

pub(crate) struct ServerService {
    service_state_handle: ServiceStateHandle<Self>,
    initial_state: <Self as ServiceData>::State,
    server: Server,
}

impl ServiceData for ServerService {
    const SERVICE_ID: ServiceId = "peers";
    type Settings = ServerSettings;
    type State = PeersState;
    type StateOperator = PeersStateOperator;
    type Message = ProtocolMessage;
}

#[async_trait::async_trait]
impl ServiceCore for ServerService {
    fn init(
        service_state: ServiceStateHandle<Self>,
        initial_state: Self::State,
    ) -> Result<Self, DynError> {
        let settings = service_state.settings_reader.get_updated_settings();
        let peers = Server::new(
            &settings.address,
            service_state.overwatch_handle.runtime().clone(),
        )?;
        Ok(Self {
            service_state_handle: service_state,
            initial_state,
            server: peers,
        })
    }

    async fn run(mut self) -> Result<(), DynError> {
        let Self {
            service_state_handle,
            initial_state: _initial_state,
            mut server,
        } = self;

        let ServerSettings {
            address,
            known_peer,
        } = service_state_handle.settings_reader.get_updated_settings();

        if let Some(known_peer) = &known_peer {
            let known_peer = known_peer.parse()?;
            server.bootstrap_from_known_peer(known_peer).await?;
        }

        server.listen(&address).await?;

        Ok(())
    }
}
