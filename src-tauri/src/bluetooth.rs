use btleplug::api::{Central, Manager as _};
use btleplug::platform::Manager;
use buttplug::client::ButtplugClient;
use buttplug::core::connector::ButtplugInProcessClientConnectorBuilder;
use buttplug::server::ButtplugServerBuilder;
use buttplug::server::device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder;
use buttplug::server::device::hardware::communication::lovense_connect_service::LovenseConnectServiceCommunicationManagerBuilder;
use log::{error as logerr, info, warn, trace};

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

pub async fn vc_toy_client_server_init(client_name: &str, btle_enabled: &mut bool, allow_raw_messages: bool) -> ButtplugClient {
    
    let mut server_builder = ButtplugServerBuilder::default();

    server_builder.comm_manager(BtlePlugCommunicationManagerBuilder::default());
    *btle_enabled = true;
    trace!("Added BtlePlug comm manager");
    //info!("Bluetooth LE interface detected.. Enabling btle.");
    
    server_builder.comm_manager(LovenseConnectServiceCommunicationManagerBuilder::default());
    trace!("Added Lovense Connect comm manager");
    if allow_raw_messages {
      server_builder.allow_raw_messages();
    }
    let server = server_builder.finish().unwrap();
    let connector = ButtplugInProcessClientConnectorBuilder::default()
    // Here pass Arc<Mutex<ButtplugServer>>
      .server(server)
      .finish();

    let client = ButtplugClient::new(client_name);
    client.connect(connector).await.unwrap();
    // Here return client and Arc<Mutex<ButtplugServer>>
    client
}