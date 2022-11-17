use btleplug::api::{Central, Manager as _};
use btleplug::platform::Manager;
use buttplug::client::ButtplugClient;
use buttplug::core::connector::ButtplugInProcessClientConnectorBuilder;
use buttplug::server::ButtplugServerBuilder;
use buttplug::server::device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder;
use buttplug::server::device::hardware::communication::lovense_connect_service::LovenseConnectServiceCommunicationManagerBuilder;



pub async fn detect_btle_adapter() -> bool {
    if let Ok(manager) = Manager::new().await {
        if let Ok(adapters) = manager.adapters().await {
            if adapters.is_empty() {
                return false;
            }
            let adapter = manager.adapters().await.unwrap();
            let central = adapter.into_iter().nth(0).unwrap();
            println!("[+] BTLE: {}", central.adapter_info().await.unwrap());
            return !adapters.is_empty();

        } else {
            return false;
        }
    } else {
        println!("[-] Failed to create btle Manager.");
        return false;
    }
}

pub async fn vc_toy_client_server_init(client_name: &str, btle_enabled: &mut bool, allow_raw_messages: bool) -> ButtplugClient {
    
    let mut server_builder = ButtplugServerBuilder::default();
    if detect_btle_adapter().await {
        server_builder.comm_manager(BtlePlugCommunicationManagerBuilder::default());
        *btle_enabled = true;
    } else {
        println!("[!] No Bluetooth LE interfaces detected.. Disabling btle.");
    }
    server_builder.comm_manager(LovenseConnectServiceCommunicationManagerBuilder::default());

    if allow_raw_messages {
      server_builder.allow_raw_messages();
    }
    let server = server_builder.finish().unwrap();
    let connector = ButtplugInProcessClientConnectorBuilder::default()
      .server(server)
      .finish();
    let client = ButtplugClient::new(client_name);
    client.connect(connector).await.unwrap();
    client
}