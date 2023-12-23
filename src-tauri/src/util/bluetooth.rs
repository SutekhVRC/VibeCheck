use btleplug::api::{Central, Manager as _};
use btleplug::platform::Manager;
use buttplug::client::ButtplugClient;
use buttplug::core::connector::ButtplugInProcessClientConnectorBuilder; //new_json_ws_client_connector};
use buttplug::server::device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder;
use buttplug::server::device::hardware::communication::lovense_connect_service::LovenseConnectServiceCommunicationManagerBuilder;
use buttplug::server::ButtplugServerBuilder;
//use buttplug::server::device::hardware::communication::websocket_server::websocket_server_comm_manager::WebsocketServerDeviceCommunicationManagerBuilder;
use log::{error as logerr, info, trace, warn};

#[allow(unused)]
pub async fn detect_btle_adapter() -> bool {
    if let Ok(manager) = Manager::new().await {
        if let Ok(adapters) = manager.adapters().await {
            if adapters.is_empty() {
                return false;
            }
            let adapter = manager.adapters().await.unwrap();
            let central = adapter.into_iter().nth(0).unwrap();
            info!("[+] BTLE: {}", central.adapter_info().await.unwrap());
            return !adapters.is_empty();
        } else {
            warn!("No btle adapters detected");
            return false;
        }
    } else {
        logerr!("[-] Failed to create btle Manager.");
        return false;
    }
}

pub async fn vc_toy_client_server_init(
    client_name: &str,
    allow_raw_messages: bool,
) -> ButtplugClient {
    let mut server_builder = ButtplugServerBuilder::default();
    server_builder.comm_manager(BtlePlugCommunicationManagerBuilder::default());
    trace!("Added BtlePlug comm manager");
    server_builder.comm_manager(LovenseConnectServiceCommunicationManagerBuilder::default());
    trace!("Added Lovense Connect comm manager");
    //new_json_ws_client_connector("ws://192.168.123.103:12345/buttplug")
    //server_builder.comm_manager(WebsocketServerDeviceCommunicationManagerBuilder::default());

    if allow_raw_messages {
        server_builder.allow_raw_messages();
    }
    let server = server_builder.finish().unwrap();

    /*
     * Possibly add support to mutate the VibeCheck internal state to use websocket connector for Intiface Central / other websocket server implementations.
     * Adding support for this would involve shutting down all handlers and completely recreating the internal VibeCheck state with a new ButtplugClient
     * Making VibeCheckState initialization modular would probably be a good idea.
     */

    let connector = ButtplugInProcessClientConnectorBuilder::default()
        .server(server)
        .finish();

    let client = ButtplugClient::new(client_name);
    client.connect(connector).await.unwrap();
    client
}
