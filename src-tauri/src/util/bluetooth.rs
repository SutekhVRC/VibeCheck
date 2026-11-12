use btleplug::api::{Central, Manager as _};
use btleplug::platform::Manager;
use buttplug_client::ButtplugClient;
use buttplug_client_in_process::ButtplugInProcessClientConnectorBuilder;
use buttplug_server::device::ServerDeviceManagerBuilder;
use buttplug_server::ButtplugServerBuilder;
use buttplug_server_device_config::load_protocol_configs;
use buttplug_server_hwmgr_btleplug::BtlePlugCommunicationManagerBuilder;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use buttplug_server_hwmgr_lovense_dongle::LovenseHIDDongleCommunicationManagerBuilder;
use log::{error as logerr, info, trace, warn};

use crate::util::errors::UtilError;

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

/// Starts VibeCheck's own Buttplug server in-process (Bluetooth LE, plus the
/// Lovense USB dongle on Windows and Linux) and connects a client to it.
pub async fn vc_toy_client_server_init(client_name: &str) -> Result<ButtplugClient, UtilError> {
    // Buttplug's built-in device list, no user config
    let dcm = match load_protocol_configs(&None, &None, false).and_then(|mut b| b.finish()) {
        Ok(dcm) => dcm,
        Err(e) => {
            logerr!("Couldn't load Buttplug's device configuration: {}", e);
            return Err(UtilError::BPIOInit);
        }
    };

    let mut device_manager_builder = ServerDeviceManagerBuilder::new(dcm);
    device_manager_builder.comm_manager(BtlePlugCommunicationManagerBuilder::default());
    trace!("Added BtlePlug comm manager");

    // Buttplug 12 dropped the serial (COM port) Lovense dongle manager; the
    // HID dongle is still supported
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        device_manager_builder.comm_manager(LovenseHIDDongleCommunicationManagerBuilder::default());
        trace!("Added Lovense HID dongle manager");
    }

    let sdm = match device_manager_builder.finish() {
        Ok(sdm) => sdm,
        Err(_) => return Err(UtilError::BPIOInit),
    };

    let server_builder = ButtplugServerBuilder::new(sdm);
    let server = match server_builder.finish() {
        Ok(server) => server,
        Err(_) => return Err(UtilError::BPIOInit),
    };

    /*
     * Possibly add support to mutate the VibeCheck internal state to use websocket connector for Intiface Central / other websocket server implementations.
     * Adding support for this would involve shutting down all handlers and completely recreating the internal VibeCheck state with a new ButtplugClient
     * Making VibeCheckState initialization modular would probably be a good idea.
     */

    let connector = ButtplugInProcessClientConnectorBuilder::default()
        .server(server)
        .finish();

    let client = ButtplugClient::new(client_name);
    if (client.connect(connector).await).is_err() {
        return Err(UtilError::BPIOInit);
    }
    Ok(client)
}
