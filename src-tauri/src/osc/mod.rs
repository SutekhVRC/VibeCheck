use std::net::{Ipv4Addr, SocketAddrV4};

use std::time::{Duration, Instant};

use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::frontend::frontend_types::{FeOSCNetworking, FeOscMode};
use crate::util::net::{find_available_udp_port, udp_port_available};

pub mod avatar_catalog;
pub mod avatar_query;
pub mod logic;
pub mod oscquery;

/// The default addresses: VRChat sends to 9001 and listens on 9000.
pub(crate) const DEFAULT_BIND: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9001);
const DEFAULT_REMOTE: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9000);

/// How long [`OSCNetworking::claim_saved_port`] waits for a busy saved port
/// to become free (e.g. a previous VibeCheck still closing) before moving.
const PORT_FREE_WAIT: Duration = Duration::from_secs(5);
const PORT_FREE_POLL: Duration = Duration::from_millis(250);

/// How VibeCheck and VRChat find each other.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscMode {
    /// VibeCheck listens on its own port and advertises it over OSCQuery;
    /// VRChat finds it and sends there. Battery levels go to the port VRChat
    /// advertises. No port to clash with other OSC apps.
    OscQuery,
    /// The bind and remote addresses set by hand (9001 and 9000 by default).
    Manual,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OSCNetworking {
    /// Manual mode: where VibeCheck listens.
    pub bind: SocketAddrV4,
    /// Manual mode: where VibeCheck sends (battery levels).
    pub remote: SocketAddrV4,
    /// Missing in configs from before OSCQuery mode; [`settle`](Self::settle)
    /// fills it in.
    #[serde(default)]
    pub mode: Option<OscMode>,
    /// OSCQuery mode: the port VibeCheck listens on, picked once and kept, so
    /// it's the same every run. VRChat keeps sending to a port it has found
    /// (it ignores the mDNS goodbye), so changing it would leave VRChat
    /// sending to a closed port.
    #[serde(default)]
    pub oscquery_port: Option<u16>,
}

impl Default for OSCNetworking {
    fn default() -> Self {
        Self {
            bind: DEFAULT_BIND,
            remote: DEFAULT_REMOTE,
            mode: Some(OscMode::OscQuery),
            oscquery_port: None,
        }
    }
}

impl OSCNetworking {
    pub fn mode(&self) -> OscMode {
        self.mode.unwrap_or(OscMode::OscQuery)
    }

    /// Fills in what older configs lack, and picks the OSCQuery port if
    /// there's none yet. Never changes a saved port: VibeCheck may be holding
    /// it itself (see [`claim_saved_port`](Self::claim_saved_port) for
    /// startup). Returns whether anything changed (so the config gets saved).
    ///
    /// A config from before OSCQuery mode stays on Manual if its bind or
    /// remote address was changed from the defaults (e.g. for a router app);
    /// otherwise it moves to OSCQuery.
    pub fn settle(&mut self) -> bool {
        let mut changed = false;
        if self.mode.is_none() {
            let customised = self.bind != DEFAULT_BIND || self.remote != DEFAULT_REMOTE;
            let mode = if customised {
                OscMode::Manual
            } else {
                OscMode::OscQuery
            };
            info!("Connection mode set to {:?} (from an older config)", mode);
            self.mode = Some(mode);
            changed = true;
        }
        if self.mode() == OscMode::OscQuery && self.oscquery_port.is_none() {
            match find_available_udp_port(Ipv4Addr::UNSPECIFIED) {
                Some(port) => {
                    info!("OSCQuery port picked: {}", port);
                    self.oscquery_port = Some(port);
                    changed = true;
                }
                None => warn!("No free port found for OSCQuery mode"),
            }
        }
        changed
    }

    /// At startup, before VibeCheck binds anything: makes sure the saved
    /// OSCQuery port can be used. If it's busy, waits up to
    /// [`PORT_FREE_WAIT`] for it to free up (e.g. the previous VibeCheck is
    /// still closing), and only then moves to another port. Moving is a last
    /// resort: VRChat keeps sending to a port it has found, until it
    /// restarts. Returns whether the port changed.
    pub fn claim_saved_port(&mut self) -> bool {
        self.claim_saved_port_within(PORT_FREE_WAIT)
    }

    fn claim_saved_port_within(&mut self, wait: Duration) -> bool {
        let Some(saved) = self.oscquery_port else {
            return false;
        };
        if self.mode() != OscMode::OscQuery {
            return false;
        }
        let deadline = Instant::now() + wait;
        loop {
            if udp_port_available(Ipv4Addr::UNSPECIFIED, saved) {
                return false;
            }
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(PORT_FREE_POLL);
        }
        match find_available_udp_port(Ipv4Addr::UNSPECIFIED) {
            Some(port) => {
                warn!(
                    "OSCQuery port {} is taken by another app; using {} instead (restart VRChat if it keeps sending to {})",
                    saved, port, saved
                );
                self.oscquery_port = Some(port);
                true
            }
            None => {
                warn!("OSCQuery port {} is taken and no other port is free", saved);
                false
            }
        }
    }

    /// Where VibeCheck listens for OSC.
    ///
    /// In OSCQuery mode that's every interface: VRChat ignores the address
    /// VibeCheck advertises and sends to the machine's LAN address instead,
    /// so listening only on 127.0.0.1 would get nothing.
    pub fn listen_addr(&self) -> SocketAddrV4 {
        match self.mode() {
            OscMode::Manual => self.bind,
            OscMode::OscQuery => SocketAddrV4::new(
                Ipv4Addr::UNSPECIFIED,
                self.oscquery_port.unwrap_or(DEFAULT_BIND.port()),
            ),
        }
    }

    /// Where VibeCheck sends OSC to VRChat. In OSCQuery mode, `vrchat` is the
    /// address VRChat advertises (if it's been found); otherwise the default.
    pub fn remote_addr(&self, vrchat: Option<SocketAddrV4>) -> SocketAddrV4 {
        match self.mode() {
            OscMode::Manual => self.remote,
            OscMode::OscQuery => vrchat.unwrap_or(DEFAULT_REMOTE),
        }
    }

    pub fn to_fe(&self) -> FeOSCNetworking {
        FeOSCNetworking {
            bind: self.bind.to_string(),
            remote: self.remote.to_string(),
            mode: match self.mode() {
                OscMode::OscQuery => FeOscMode::OscQuery,
                OscMode::Manual => FeOscMode::Manual,
            },
            oscquery_port: self.oscquery_port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn old_config(bind: &str, remote: &str) -> OSCNetworking {
        // Shaped like a config from before OSCQuery mode
        serde_json::from_str(&format!(
            r#"{{"bind":"{bind}","remote":"{remote}","osc_query_enabled":true}}"#
        ))
        .unwrap()
    }

    #[test]
    fn older_configs_on_the_defaults_move_to_oscquery() {
        let mut net = old_config("127.0.0.1:9001", "127.0.0.1:9000");
        assert_eq!(net.mode, None);
        assert!(net.settle());
        assert_eq!(net.mode(), OscMode::OscQuery);
        let port = net.oscquery_port.expect("a port was picked");
        assert_eq!(
            net.listen_addr(),
            SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)
        );
        // Settled now: nothing changes the next time
        assert!(!net.settle());
        assert_eq!(net.oscquery_port, Some(port));
    }

    #[test]
    fn older_configs_with_custom_addresses_stay_manual() {
        for (bind, remote) in [
            ("127.0.0.1:9002", "127.0.0.1:9000"),
            ("127.0.0.1:9001", "192.168.1.5:9000"),
        ] {
            let mut net = old_config(bind, remote);
            assert!(net.settle());
            assert_eq!(net.mode(), OscMode::Manual);
            assert_eq!(net.oscquery_port, None);
            assert_eq!(net.listen_addr().to_string(), bind);
            assert_eq!(net.remote_addr(None).to_string(), remote);
        }
    }

    #[test]
    fn settle_never_moves_a_saved_port_even_while_its_held() {
        // VibeCheck's own listener holds the port whenever settings are saved
        let held = std::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let port = held.local_addr().unwrap().port();
        let mut net = OSCNetworking {
            oscquery_port: Some(port),
            ..Default::default()
        };
        assert!(!net.settle());
        assert_eq!(net.oscquery_port, Some(port));
    }

    #[test]
    fn startup_waits_for_a_busy_saved_port_before_moving() {
        let held = std::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
        let port = held.local_addr().unwrap().port();
        let mut net = OSCNetworking {
            oscquery_port: Some(port),
            ..Default::default()
        };

        // Freed while waiting (the previous VibeCheck finished closing): kept
        let release = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            drop(held);
        });
        assert!(!net.claim_saved_port_within(Duration::from_secs(3)));
        assert_eq!(net.oscquery_port, Some(port));
        release.join().unwrap();

        // Still taken after the wait (another app): moved
        let other_app = std::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port)).unwrap();
        assert!(net.claim_saved_port_within(Duration::from_millis(300)));
        assert_ne!(net.oscquery_port, Some(port));
        drop(other_app);
    }

    #[test]
    fn oscquery_mode_sends_to_vrchats_port_once_found() {
        let net = OSCNetworking::default();
        assert_eq!(net.remote_addr(None), DEFAULT_REMOTE);
        let vrchat = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9123);
        assert_eq!(net.remote_addr(Some(vrchat)), vrchat);
    }
}
