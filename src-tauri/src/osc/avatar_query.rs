/*
 * Fills in avatar parameters that VRChat never sends over OSC.
 *
 * VRChat only sends a parameter over OSC when its value changes. Some
 * parameters are set once when the avatar loads and never change (an SPS
 * plug's LengthMult2, a socket's DepthRadius). If one sits at its default,
 * VibeCheck never hears it, and it's also missed if VibeCheck starts after
 * the avatar loaded.
 *
 * VRChat's OSCQuery server reports every parameter's current value, so the
 * ones that matter are read from there: when listening starts, and a few
 * times after each avatar change (VRChat may still be loading the avatar).
 * VRChat's server is found over mDNS like any OSCQuery service.
 *
 * Values are written to the signal plane as if they had arrived over OSC,
 * but only where nothing has arrived yet, so a value VRChat sends always
 * wins.
 *
 * While diagnostics are on, every value filled in, and every failed lookup,
 * is logged under the `oscquery` module, so the diagnostics view can tell
 * them apart from values VRChat sent over OSC.
 */

use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use log::{debug, info};
use mdns_sd::{ServiceDaemon, ServiceEvent};
use parking_lot::Mutex;
use serde_json::Value as Json;
use vibecheck_core::{Diagnostics, ModuleInstanceId, PortId, Signal, SignalPlane};

const OSCJSON_SERVICE: &str = "_oscjson._tcp.local.";
const VRCHAT_SERVICE_PREFIX: &str = "VRChat-Client-";
const PARAMETERS_PATH: &str = "/avatar/parameters";

/// The module diagnostics log entries are logged under.
const LOG_MODULE: &str = "oscquery";

/// The label of log entries about the lookup itself, rather than a value.
const LOG_LABEL: &str = "OSCQuery";

/// How long to look for VRChat's OSCQuery server.
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// When to look after an avatar change. Later attempts only fill in what's
/// still missing, so they're cheap, and they cover avatars that take a while
/// to load.
const AFTER_AVATAR_CHANGE: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(4),
    Duration::from_secs(10),
];

/// Which parameters to fill in, by full OSC address.
pub type Wanted = fn(&str) -> bool;

#[derive(Clone)]
pub struct AvatarQuery {
    signals: SignalPlane,
    /// The OSC input's module: values are written as if it received them.
    module: ModuleInstanceId,
    wanted: Wanted,
    /// VRChat's OSCQuery server (`http://host:port`), once found.
    endpoint: Arc<Mutex<Option<String>>>,
    /// Bumped on every avatar change, so a lookup still running for the
    /// previous avatar doesn't write anything.
    avatar: Arc<AtomicU64>,
    diagnostics: Diagnostics,
    log_module: ModuleInstanceId,
    /// Addresses filled in for the current avatar, for the diagnostics view.
    filled: Arc<Mutex<HashSet<String>>>,
}

impl AvatarQuery {
    pub fn new(
        signals: SignalPlane,
        module: ModuleInstanceId,
        wanted: Wanted,
        diagnostics: Diagnostics,
    ) -> Self {
        Self {
            signals,
            module,
            wanted,
            endpoint: Arc::default(),
            avatar: Arc::default(),
            diagnostics,
            log_module: ModuleInstanceId::new(LOG_MODULE),
            filled: Arc::default(),
        }
    }

    /// Whether `addr` was filled in from OSCQuery for the current avatar.
    /// It stays true after VRChat sends the parameter itself; check the
    /// port's write count to tell (a filled-in value is written once).
    pub fn was_filled(&self, addr: &str) -> bool {
        self.filled.lock().contains(addr)
    }

    /// Adds a diagnostics log entry about the lookup.
    fn log(&self, message: impl FnOnce() -> String) {
        self.diagnostics.log(&self.log_module, LOG_LABEL, message);
    }

    /// Looks once, for the avatar that's already loaded.
    pub async fn fill_now(&self) {
        let avatar = self.avatar.load(Ordering::SeqCst);
        self.fill(avatar).await;
    }

    /// Looks a few times over the next seconds. Call on every avatar change,
    /// after the OSC input has cleared the old avatar's values.
    pub async fn fill_after_avatar_change(&self) {
        let avatar = self.avatar.fetch_add(1, Ordering::SeqCst) + 1;
        self.filled.lock().clear();
        let start = Instant::now();
        for at in AFTER_AVATAR_CHANGE {
            tokio::time::sleep(at.saturating_sub(start.elapsed())).await;
            if self.avatar.load(Ordering::SeqCst) != avatar {
                return; // Changed again: the newer lookup takes over
            }
            self.fill(avatar).await;
        }
    }

    async fn fill(&self, avatar: u64) {
        let Some(values) = self.fetch().await else {
            self.log(|| "VRChat's OSCQuery server wasn't found or didn't answer".into());
            return;
        };
        if self.avatar.load(Ordering::SeqCst) != avatar {
            return;
        }
        let filled = self.apply(values);
        if filled > 0 {
            info!("Filled in {} avatar parameters from OSCQuery", filled);
        }
    }

    /// Writes the wanted values that haven't arrived over OSC yet. Returns
    /// how many were written.
    fn apply(&self, values: Vec<(String, Signal)>) -> usize {
        let mut filled = 0;
        for (addr, value) in values {
            if (self.wanted)(&addr)
                && self
                    .signals
                    .write_if_absent(&PortId::new(self.module.clone(), addr.as_str()), value)
            {
                debug!("OSCQuery filled in {} = {:?}", addr, value);
                self.diagnostics.log(&self.log_module, addr.as_str(), || {
                    format!("{} from OSCQuery", format_signal(value))
                });
                self.filled.lock().insert(addr);
                filled += 1;
            }
        }
        filled
    }

    /// Every avatar parameter's current value, or `None` if VRChat's
    /// OSCQuery server can't be reached.
    async fn fetch(&self) -> Option<Vec<(String, Signal)>> {
        let known = self.endpoint.lock().clone();
        let endpoint = match known {
            Some(endpoint) => endpoint,
            None => {
                let found = tokio::task::spawn_blocking(find_vrchat).await.ok()??;
                info!("Found VRChat's OSCQuery server at {}", found);
                self.log(|| format!("Found VRChat's OSCQuery server at {found}"));
                *self.endpoint.lock() = Some(found.clone());
                found
            }
        };
        match get_tree(&endpoint).await {
            Ok(tree) => {
                let mut values = Vec::new();
                leaves(&tree, &mut values);
                Some(values)
            }
            Err(e) => {
                // VRChat restarted (new port) or closed: look again next time
                debug!("OSCQuery request to {} failed: {}", endpoint, e);
                *self.endpoint.lock() = None;
                None
            }
        }
    }
}

/// A value the way the diagnostics log shows OSC arguments, e.g.
/// `17.373978 (float)`. VRChat's values are 32-bit, so they're shown as such.
fn format_signal(value: Signal) -> String {
    match value {
        Signal::Float(v) => format!("{} (float)", v as f32),
        Signal::Bool(b) => format!("{b} (bool)"),
    }
}

/// Finds VRChat's OSCQuery server over mDNS. Blocking.
fn find_vrchat() -> Option<String> {
    let mdns = ServiceDaemon::new().ok()?;
    let found = mdns.browse(OSCJSON_SERVICE).ok().and_then(|events| {
        let deadline = Instant::now() + DISCOVERY_TIMEOUT;
        loop {
            match events.recv_deadline(deadline) {
                Ok(ServiceEvent::ServiceResolved(service))
                    if service.get_fullname().starts_with(VRCHAT_SERVICE_PREFIX) =>
                {
                    // Prefer this machine's address if it's advertised
                    let addresses = service.get_addresses();
                    let host = addresses
                        .iter()
                        .find(|a| a.is_loopback())
                        .or_else(|| addresses.iter().find(|a| matches!(a, IpAddr::V4(_))))
                        .or_else(|| addresses.iter().next())?;
                    let host = match host {
                        IpAddr::V6(v6) => format!("[{v6}]"),
                        IpAddr::V4(v4) => v4.to_string(),
                    };
                    break Some(format!("http://{}:{}", host, service.get_port()));
                }
                Ok(_) => continue,
                Err(_) => {
                    debug!("VRChat's OSCQuery server wasn't found over mDNS");
                    break None;
                }
            }
        }
    });
    let _ = mdns.shutdown();
    found
}

async fn get_tree(endpoint: &str) -> Result<Json, String> {
    let text = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())?
        .get(format!("{endpoint}{PARAMETERS_PATH}"))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Collects every float or bool value in an OSCQuery node tree, by full
/// path. Other types are skipped, as the OSC input skips them.
fn leaves(node: &Json, out: &mut Vec<(String, Signal)>) {
    if let Some(contents) = node.get("CONTENTS").and_then(Json::as_object) {
        for child in contents.values() {
            leaves(child, out);
        }
        return;
    }
    let path = node.get("FULL_PATH").and_then(Json::as_str);
    let value = node
        .get("VALUE")
        .and_then(Json::as_array)
        .and_then(|v| v.first());
    let (Some(path), Some(value)) = (path, value) else {
        return;
    };
    let signal = match (node.get("TYPE").and_then(Json::as_str), value) {
        (Some("f"), Json::Number(n)) => n.as_f64().map(Signal::Float),
        (_, Json::Bool(b)) => Some(Signal::Bool(*b)),
        _ => None,
    };
    if let Some(signal) = signal {
        out.push((path.to_owned(), signal));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logs_each_value_it_fills_in_but_not_ones_that_already_arrived() {
        let signals = SignalPlane::new();
        let diagnostics = Diagnostics::default();
        diagnostics.set_enabled(true);
        let query = AvatarQuery::new(
            signals.clone(),
            ModuleInstanceId::new("osc"),
            |addr| addr.ends_with("LengthMult2") || addr.ends_with("DepthRadius"),
            diagnostics.clone(),
        );
        let plug = "/avatar/parameters/OGB/Plug/Test/LengthMult2";
        let socket = "/avatar/parameters/OGB/Socket/Test/DepthRadius";
        // VRChat already sent the socket's value over OSC
        signals.write(
            &PortId::new(ModuleInstanceId::new("osc"), socket),
            Signal::Float(0.5),
        );

        let filled = query.apply(vec![
            (plug.to_owned(), Signal::Float(17.3739777)),
            (socket.to_owned(), Signal::Float(1.0)),
            ("/avatar/parameters/Other".to_owned(), Signal::Bool(true)),
        ]);
        assert_eq!(filled, 1);
        assert!(query.was_filled(plug));
        assert!(!query.was_filled(socket));

        let logged: Vec<(String, String, String)> = diagnostics
            .entries_since(0)
            .0
            .into_iter()
            .map(|e| (e.module.to_string(), e.label, e.value))
            .collect();
        assert_eq!(
            logged,
            vec![(
                "oscquery".to_owned(),
                plug.to_owned(),
                "17.373978 (float) from OSCQuery".to_owned()
            )]
        );
    }

    #[test]
    fn leaves_reads_floats_and_bools_from_a_node_tree() {
        // Shaped like VRChat's reply, with values from a real avatar
        let tree: Json = serde_json::from_str(
            r#"{
              "FULL_PATH": "/avatar/parameters", "ACCESS": 0,
              "CONTENTS": {
                "OGB": { "FULL_PATH": "/avatar/parameters/OGB", "ACCESS": 0, "CONTENTS": {
                  "Plug": { "FULL_PATH": "/avatar/parameters/OGB/Plug", "ACCESS": 0, "CONTENTS": {
                    "zw_ris_sps": { "FULL_PATH": "/avatar/parameters/OGB/Plug/zw_ris_sps", "ACCESS": 0, "CONTENTS": {
                      "LengthMult2": { "FULL_PATH": "/avatar/parameters/OGB/Plug/zw_ris_sps/LengthMult2",
                                       "ACCESS": 3, "TYPE": "f", "VALUE": [17.3739777] }
                    }}
                  }}
                }},
                "Toggle": { "FULL_PATH": "/avatar/parameters/Toggle", "ACCESS": 3, "TYPE": "T", "VALUE": [true] },
                "Outfit": { "FULL_PATH": "/avatar/parameters/Outfit", "ACCESS": 3, "TYPE": "i", "VALUE": [2] },
                "Empty": { "FULL_PATH": "/avatar/parameters/Empty", "ACCESS": 3, "TYPE": "f" }
              }
            }"#,
        )
        .unwrap();
        let mut values = Vec::new();
        leaves(&tree, &mut values);
        values.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            values,
            vec![
                (
                    "/avatar/parameters/OGB/Plug/zw_ris_sps/LengthMult2".to_owned(),
                    Signal::Float(17.3739777)
                ),
                ("/avatar/parameters/Toggle".to_owned(), Signal::Bool(true)),
            ]
        );
    }
}
