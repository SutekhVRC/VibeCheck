/*
 * Turns on newer SPS avatars' haptics while VibeCheck needs them.
 *
 * Newer SPS avatars send no haptics parameters (the `OGB/...` contacts)
 * until an app sets their enable switch to true: `OGB/Enabled` on SPS 3,
 * `OGB_ENABLED` on newer legacy SPS. It starts false whenever the avatar
 * loads. Without it, SPS input processors get nothing from these avatars.
 *
 * While the session runs and some graph watches SPS parameters (a toy
 * feature with System SPS, or an app's SPS node), this sends `true` to each
 * switch the current avatar has: at once when that starts or the avatar
 * changes, then every RESEND_EVERY. It never sends false (another app may
 * be using the switch, and the avatar starts at false on its next load).
 * Which switches the avatar has comes from the avatar catalog (OSCQuery);
 * without it nothing is sent. Sent like battery levels: from a socket on
 * port 0 to where VibeCheck sends to VRChat.
 */

use std::{
    net::Ipv4Addr,
    sync::Arc,
    time::{Duration, Instant},
};

use log::{info, warn};
use rosc::{encoder, OscMessage, OscPacket, OscType};
use tauri::{AppHandle, Manager};
use tokio::{net::UdpSocket, sync::watch, time::sleep};
use vibecheck_core::{ModuleInstanceId, Watch};

use crate::{
    osc::avatar_catalog::{Avatar, AvatarCatalog},
    toy_handling::runtime::toy_graph::OSC_MODULE,
    vcore::state::VCStateMutex,
};

/// The enable switches, as full OSC addresses.
pub const SWITCHES: [&str; 2] = [
    "/avatar/parameters/OGB/Enabled",
    "/avatar/parameters/OGB_ENABLED",
];

/// How often the switches are sent again while needed.
pub const RESEND_EVERY: Duration = Duration::from_secs(5);

/// How often it checks whether it's needed and whether the avatar changed.
const CHECK_EVERY: Duration = Duration::from_secs(1);

/// SPS parameters, as the SPS processor watches them.
const SPS_PREFIXES: [&str; 2] = ["/avatar/parameters/OGB/", "/avatar/parameters/VFH/"];

/// Whether any of `watches` reads SPS parameters.
pub fn wants_sps(watches: &[Watch]) -> bool {
    let osc = ModuleInstanceId::new(OSC_MODULE);
    watches
        .iter()
        .any(|w| w.module == osc && SPS_PREFIXES.iter().any(|p| w.prefix.starts_with(p)))
}

/// The switches `avatar` has.
pub fn switches(avatar: &Avatar) -> Vec<&'static str> {
    SWITCHES
        .into_iter()
        .filter(|s| avatar.param(s).is_some())
        .collect()
}

/// The switch as it's known, without `/avatar/parameters/`.
fn short(switch: &str) -> &str {
    switch.strip_prefix("/avatar/parameters/").unwrap_or(switch)
}

/// Runs for the session (see the top of this file). `watched` is every
/// watch of every running graph (the graph runner keeps it current).
pub async fn sps_switch(
    watched: watch::Receiver<Vec<Watch>>,
    catalog: AvatarCatalog,
    app_handle: AppHandle,
) {
    let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await {
        Ok(socket) => socket,
        Err(e) => {
            warn!("SPS: couldn't open a socket to turn avatars' haptics on: {e}");
            return;
        }
    };
    let mut last_avatar: Option<Arc<Avatar>> = None;
    let mut last_sent: Option<Instant> = None;
    let mut logged: Vec<&'static str> = Vec::new();

    loop {
        let avatar = catalog.avatar().filter(|_| wants_sps(&watched.borrow()));
        let to_send = avatar.as_deref().map(switches).unwrap_or_default();
        if to_send.is_empty() {
            last_avatar = None;
            last_sent = None;
            logged.clear();
        } else {
            let changed = !matches!(
                (&last_avatar, &avatar),
                (Some(a), Some(b)) if Arc::ptr_eq(a, b)
            );
            let due = last_sent.is_none_or(|at| at.elapsed() >= RESEND_EVERY);
            if changed || due {
                let remote = app_handle.state::<VCStateMutex>().0.lock().osc_remote();
                for switch in &to_send {
                    let packet = OscPacket::Message(OscMessage {
                        addr: (*switch).to_owned(),
                        args: vec![OscType::Bool(true)],
                    });
                    match encoder::encode(&packet) {
                        Ok(buf) => {
                            if let Err(e) = socket.send_to(&buf, remote).await {
                                warn!("SPS: couldn't send {} to VRChat: {e}", short(switch));
                            }
                        }
                        Err(e) => warn!("SPS: couldn't encode {}: {e}", short(switch)),
                    }
                }
                if logged != to_send {
                    let names: Vec<&str> = to_send.iter().map(|s| short(s)).collect();
                    info!(
                        "SPS: turned on this avatar's haptics ({})",
                        names.join(", ")
                    );
                    logged = to_send.clone();
                }
                last_sent = Some(Instant::now());
            }
            last_avatar = avatar;
        }
        sleep(CHECK_EVERY).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn watch(prefix: &str) -> Watch {
        Watch::new(ModuleInstanceId::new(OSC_MODULE), prefix)
    }

    fn fixture(name: &str) -> Avatar {
        let path = format!("{}/src/osc/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        let tree: vrcoscquery::node::OSCQueryNode =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        Avatar::from_tree(&tree)
    }

    #[test]
    fn only_the_switches_the_avatar_has() {
        // A captured SPS 3 avatar has OGB/Enabled; the older legacy SPS
        // capture has no switch (its parameters flow without one)
        assert_eq!(
            switches(&fixture("sps3_with_legacy.json")),
            vec!["/avatar/parameters/OGB/Enabled"]
        );
        assert!(switches(&fixture("legacy_sps.json")).is_empty());
        assert!(switches(&fixture("no_system.json")).is_empty());
    }

    #[test]
    fn the_switches_never_become_sps_zones() {
        use crate::toy_handling::input_processor::penetration_systems::sps;
        for switch in SWITCHES {
            assert!(sps::zones([short(switch)]).is_empty(), "{switch}");
        }
    }

    #[test]
    fn needed_only_while_something_watches_sps() {
        assert!(!wants_sps(&[]));
        assert!(!wants_sps(&[watch("/avatar/parameters/TPS_Internal/")]));
        assert!(wants_sps(&[watch("/avatar/parameters/OGB/")]));
        assert!(wants_sps(&[watch("/avatar/parameters/VFH/Zone/")]));
        // Another module's ports with the same name don't count
        assert!(!wants_sps(&[Watch::new(
            ModuleInstanceId::new("other"),
            "/avatar/parameters/OGB/"
        )]));
    }
}
