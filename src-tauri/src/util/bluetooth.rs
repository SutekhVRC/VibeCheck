use btleplug::api::{Central, Manager as _};
use btleplug::platform::Manager;
use buttplug::client::ButtplugClient;
use buttplug::core::connector::ButtplugInProcessClientConnectorBuilder;
//new_json_ws_client_connector};
use buttplug::server::device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder;
use buttplug::server::device::hardware::communication::lovense_dongle::{LovenseHIDDongleCommunicationManagerBuilder, LovenseSerialDongleCommunicationManagerBuilder};
use buttplug::server::device::ServerDeviceManagerBuilder;
use buttplug::server::ButtplugServerBuilder;
use buttplug::util::device_configuration::load_protocol_configs;
use log::{error as logerr, info, trace, warn};

#[allow(unused)]
pub async fn detect_btle_adapter() -> bool {
    let Ok(manager) = Manager::new().await else {
        logerr!("[-] Failed to create btle Manager.");
        return false;
    };
    let Ok(adapters) = manager.adapters().await else {
        warn!("No btle adapters detected");
        return false;
    };
    if adapters.is_empty() {
        return false;
    }

    let adapter = manager.adapters().await.unwrap();
    let central = adapter.into_iter().next().unwrap();
    info!("[+] BTLE: {}", central.adapter_info().await.unwrap());
    !adapters.is_empty() // TODO is this always true?
}

pub async fn vc_toy_client_server_init(
    client_name: &str,
    allow_raw_messages: bool,
) -> ButtplugClient {

    let dcm = load_protocol_configs(&None, &None, false).unwrap()
    .allow_raw_messages(allow_raw_messages)
    .finish()
    .unwrap();

    let mut device_manager_builder = ServerDeviceManagerBuilder::new(dcm);
    device_manager_builder.comm_manager(BtlePlugCommunicationManagerBuilder::default());
    trace!("Added BtlePlug comm manager");

    device_manager_builder.comm_manager(LovenseHIDDongleCommunicationManagerBuilder::default());
    device_manager_builder.comm_manager(LovenseSerialDongleCommunicationManagerBuilder::default());
    trace!("Added Lovense Dongle HID/Serial managers");

    let server_builder = ButtplugServerBuilder::new(device_manager_builder.finish().unwrap());
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
