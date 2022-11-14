use std::net::Ipv4Addr;
use std::path::Path;
use std::io::Cursor;
use buttplug::client::ButtplugClient;
use buttplug::core::connector::{ButtplugInProcessClientConnectorBuilder};
use buttplug::server::ButtplugServerBuilder;
use buttplug::server::device::hardware::communication::btleplug::BtlePlugCommunicationManagerBuilder;
use buttplug::server::device::hardware::communication::lovense_connect_service::LovenseConnectServiceCommunicationManagerBuilder;
use buttplug::server::device::hardware::communication::lovense_dongle::{
    LovenseHIDDongleCommunicationManagerBuilder,
    LovenseSerialDongleCommunicationManagerBuilder,
};
use buttplug::server::device::hardware::communication::xinput::XInputDeviceCommunicationManagerBuilder;
use directories::BaseDirs;
use image::io::Reader as IReader;
use image::ImageFormat;

// Originally From egui discussions https://github.com/emilk/egui/discussions/1574
// Modified to work with embedded icon data
pub fn load_icon(bytes: Vec<u8>) -> eframe::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let mut reader = IReader::new(Cursor::new(bytes));
        reader.set_format(ImageFormat::Ico);
        let image = reader.decode().unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    eframe::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

pub fn check_valid_port(port: &String) -> bool {
    if let Ok(p) = port.parse::<u16>() {
        // Dont need to check for >= 0 bc of type limits unsigned 16 bit int
        if p < 65535 {
            true
        } else {
            false
        }
    } else {
        false
    }
}

pub fn check_valid_ipv4(ip: &String) -> bool {
    !ip.parse::<Ipv4Addr>().is_err()
}

pub fn path_exists(p: &String) -> bool {
    Path::new(&p).is_dir()
}

pub fn file_exists(p: &String) -> bool {
    Path::new(&p).is_file()
}

pub fn get_user_home_dir() -> String {
    let bd = BaseDirs::new().expect("[-] Could not get user's directories.");
    let bd = bd
        .home_dir()
        .to_str()
        .expect("[-] Failed to get user's home directory.");
    bd.to_string()
}

pub async fn vc_toy_client_server_init(client_name: &str, allow_raw_messages: bool) -> ButtplugClient {
    
    let mut server_builder = ButtplugServerBuilder::default();
    server_builder.name("VibeCheck Toy Server");

    server_builder.comm_manager(LovenseHIDDongleCommunicationManagerBuilder::default());
    server_builder.comm_manager(LovenseSerialDongleCommunicationManagerBuilder::default());

    server_builder.comm_manager(BtlePlugCommunicationManagerBuilder::default());
    server_builder.comm_manager(LovenseConnectServiceCommunicationManagerBuilder::default());
    server_builder.comm_manager(XInputDeviceCommunicationManagerBuilder::default());

    if allow_raw_messages {
      server_builder.allow_raw_messages();
    }
    let server = server_builder.finish().unwrap();
    let connector = ButtplugInProcessClientConnectorBuilder::default().server(server).finish();
    let client = ButtplugClient::new(client_name);
    client.connect(connector).await.unwrap();
    client
}