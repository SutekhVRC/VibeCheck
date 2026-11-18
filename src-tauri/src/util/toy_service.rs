/*
 * The connection toys are reached through: a Buttplug server.
 *
 * - Local (the default): VibeCheck's own server, in-process, using this
 *   computer's Bluetooth and the Lovense USB dongle (util/bluetooth.rs).
 * - Intiface: Intiface Central (desktop, or the phone app) over websocket.
 *   For computers without Bluetooth: toys pair with the phone, and VibeCheck
 *   drives them through it over the local network.
 *
 * Everything after the connection (scanning, outputs, battery, saved toys)
 * only uses the ButtplugClient in the app state, so it works the same either
 * way.
 *
 * `run` supervises it for the life of the app:
 *   connect -> put the client in the state -> run the client event handler
 *   -> wait until the connection drops or the setting changes.
 * - Dropped (Intiface closed, the phone asleep, Wi-Fi): the client has
 *   already removed every device (DeviceRemoved, which stops and removes the
 *   toys), the status says so, and it tries again every RETRY_EVERY.
 * - Setting changed: the old client is disconnected (its toys leave the same
 *   way) and the new one connects straight away.
 * The status is kept for Settings to show.
 */

use std::{sync::Arc, time::Duration};

use buttplug_client::{connector::ButtplugRemoteClientConnector, ButtplugClient};
use buttplug_transport_websocket_tungstenite::ButtplugWebsocketClientTransport;
use log::{info, warn};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::{mpsc::UnboundedSender, watch};
use ts_rs::TS;

use crate::{
    error_signal_handler::reporter::ErrorReporter,
    toy_handling::runtime::client_event_handler::client_event_handler,
    util::bluetooth,
    vcore::{ipc::call_plane::ToyManagementEvent, state::VibeCheckState},
};

/// How long to wait before connecting again after a failure or a drop.
pub const RETRY_EVERY: Duration = Duration::from_secs(5);

/// How long to wait for the old connection's toys to leave when switching.
const SWITCH_TIMEOUT: Duration = Duration::from_secs(5);

/// The name VibeCheck gives itself to the Buttplug server.
const CLIENT_NAME: &str = "VibeCheck";

/// Where toys are reached (saved in Config.json as `toy_connection`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "mode")]
#[ts(export)]
pub enum ToyConnection {
    /// VibeCheck's own server: this computer's Bluetooth and Lovense dongle.
    #[default]
    Local,
    /// Intiface Central at `address`, e.g. `ws://192.168.1.20:12345`.
    Intiface { address: String },
}

impl ToyConnection {
    /// Checked and tidied: an Intiface address gets `ws://` if it has no
    /// scheme. Errors are for showing to the user.
    pub fn normalized(self) -> Result<Self, String> {
        match self {
            ToyConnection::Local => Ok(ToyConnection::Local),
            ToyConnection::Intiface { address } => {
                let address = address.trim();
                if address.is_empty() {
                    return Err("Enter Intiface's address, e.g. 192.168.1.20:12345".into());
                }
                if address.contains(char::is_whitespace) {
                    return Err("The address can't contain spaces".into());
                }
                let address = if address.starts_with("ws://") || address.starts_with("wss://") {
                    address.to_owned()
                } else if address.contains("://") {
                    return Err("The address must start with ws:// (or wss://)".into());
                } else {
                    format!("ws://{address}")
                };
                Ok(ToyConnection::Intiface { address })
            }
        }
    }
}

/// How the connection is doing, for Settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "state", content = "error")]
#[ts(export)]
pub enum ToyServiceStatus {
    Connecting,
    Connected,
    /// Not connected; it tries again every few seconds.
    Failed(String),
}

/// Connects to the server `connection` names.
async fn connect(connection: &ToyConnection) -> Result<ButtplugClient, String> {
    match connection {
        ToyConnection::Local => bluetooth::vc_toy_client_server_init(CLIENT_NAME)
            .await
            .map_err(|e| format!("Couldn't start VibeCheck's toy service: {e:?}")),
        ToyConnection::Intiface { address } => {
            let transport = ButtplugWebsocketClientTransport::new_insecure_connector(address);
            let connector =
                ButtplugRemoteClientConnector::<ButtplugWebsocketClientTransport>::new(transport);
            let client = ButtplugClient::new(CLIENT_NAME);
            client
                .connect(connector)
                .await
                .map(|()| client)
                .map_err(|e| {
                    format!("Couldn't connect to Intiface at {address}: {e}. Check Intiface Central is running with its server started, and the address.")
                })
        }
    }
}

/// Everything `run` needs from the app.
pub struct ToyServiceContext {
    pub state: Arc<Mutex<VibeCheckState>>,
    pub status: Arc<Mutex<ToyServiceStatus>>,
    pub identifier: String,
    pub app_handle: AppHandle,
    pub tme_send: UnboundedSender<ToyManagementEvent>,
    pub reporter: ErrorReporter,
}

/// Supervises the connection for the life of the app (see the top of this
/// file). `connection` carries the setting; send on it to switch.
pub async fn run(ctx: ToyServiceContext, mut connection: watch::Receiver<ToyConnection>) {
    loop {
        let current = connection.borrow_and_update().clone();
        *ctx.status.lock() = ToyServiceStatus::Connecting;
        info!("Toy service: connecting ({current:?})");

        let client = match connect(&current).await {
            Ok(client) => client,
            Err(e) => {
                warn!("Toy service: {e}");
                *ctx.status.lock() = ToyServiceStatus::Failed(e);
                wait_or_change(&mut connection).await;
                continue;
            }
        };

        let events = client.event_stream();
        ctx.state.lock().bp_client = Some(client);
        *ctx.status.lock() = ToyServiceStatus::Connected;
        info!("Toy service: connected");

        let mut handler = tokio::spawn(client_event_handler(
            events,
            ctx.state.clone(),
            ctx.identifier.clone(),
            ctx.app_handle.clone(),
            ctx.tme_send.clone(),
            ctx.reporter.clone(),
        ));

        tokio::select! {
            // The connection dropped: its devices are already removed
            _ = &mut handler => {
                ctx.state.lock().bp_client = None;
                let lost = match current {
                    ToyConnection::Local => "VibeCheck's toy service stopped".to_owned(),
                    ToyConnection::Intiface { address } => format!(
                        "Lost the connection to Intiface at {address}. Trying again…"
                    ),
                };
                warn!("Toy service: {lost}");
                *ctx.status.lock() = ToyServiceStatus::Failed(lost);
                wait_or_change(&mut connection).await;
            }
            // The setting changed: let the old connection's toys leave
            changed = connection.changed() => {
                let old = ctx.state.lock().bp_client.take();
                if let Some(old) = old {
                    if let Err(e) = old.disconnect().await {
                        warn!("Toy service: disconnecting: {e}");
                    }
                }
                if tokio::time::timeout(SWITCH_TIMEOUT, &mut handler).await.is_err() {
                    handler.abort();
                }
                if changed.is_err() {
                    return; // The app is closing
                }
            }
        }
    }
}

/// Waits RETRY_EVERY, or less if the setting changes.
async fn wait_or_change(connection: &mut watch::Receiver<ToyConnection>) {
    tokio::select! {
        _ = tokio::time::sleep(RETRY_EVERY) => {}
        _ = connection.changed() => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intiface(address: &str) -> ToyConnection {
        ToyConnection::Intiface {
            address: address.into(),
        }
    }

    #[test]
    fn intiface_addresses_are_tidied() {
        assert_eq!(
            intiface(" 192.168.1.20:12345 ").normalized(),
            Ok(intiface("ws://192.168.1.20:12345"))
        );
        assert_eq!(
            intiface("wss://phone.local:12345").normalized(),
            Ok(intiface("wss://phone.local:12345"))
        );
        assert!(intiface("").normalized().is_err());
        assert!(intiface("http://192.168.1.20:12345").normalized().is_err());
        assert!(intiface("192.168.1.20 12345").normalized().is_err());
        assert_eq!(ToyConnection::Local.normalized(), Ok(ToyConnection::Local));
    }

    #[test]
    fn the_setting_reads_as_saved() {
        let local: ToyConnection = serde_json::from_str(r#"{ "mode": "Local" }"#).unwrap();
        assert_eq!(local, ToyConnection::Local);
        let remote: ToyConnection =
            serde_json::from_str(r#"{ "mode": "Intiface", "address": "ws://1.2.3.4:12345" }"#)
                .unwrap();
        assert_eq!(remote, intiface("ws://1.2.3.4:12345"));
    }
}
