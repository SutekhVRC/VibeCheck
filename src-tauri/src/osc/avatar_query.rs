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
 * VRChat's server is found with VRCOSCQuery's discovery (mDNS).
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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use log::{debug, info};
use parking_lot::Mutex;
use vibecheck_core::{Diagnostics, ModuleInstanceId, PortId, Signal, SignalPlane};
use vrcoscquery::node::{OSCQueryNode, OSCQueryValue};
use vrcoscquery::{discovery, Endpoint};

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
    /// VRChat's OSCQuery server, once found.
    endpoint: Arc<Mutex<Option<Endpoint>>>,
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
                let found = discovery::find_vrchat(DISCOVERY_TIMEOUT).await.ok()??;
                info!("Found VRChat's OSCQuery server at {}", found.addr);
                self.log(|| format!("Found VRChat's OSCQuery server at {}", found.addr));
                *self.endpoint.lock() = Some(found.clone());
                found
            }
        };
        match discovery::node(&endpoint, PARAMETERS_PATH, REQUEST_TIMEOUT).await {
            Ok(tree) => Some(leaves(&tree)),
            Err(e) => {
                // VRChat restarted (new port) or closed: look again next time
                debug!("OSCQuery request to {} failed: {}", endpoint.addr, e);
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

/// Every float or bool value in an OSCQuery node tree, by full path. Other
/// types are skipped, as the OSC input skips them. Floats are checked by
/// their type tag: an Int parameter's value would read as a number too.
fn leaves(tree: &OSCQueryNode) -> Vec<(String, Signal)> {
    tree.leaves()
        .into_iter()
        .filter_map(|leaf| {
            let signal = match (leaf.type_tag(), leaf.value()?.first()?) {
                (Some("f"), OSCQueryValue::FLOAT(v)) => Signal::Float(*v as f64),
                (_, OSCQueryValue::BOOLEAN(b)) => Signal::Bool(*b),
                _ => return None,
            };
            Some((leaf.full_path.clone(), signal))
        })
        .collect()
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
        let tree: OSCQueryNode = serde_json::from_str(
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
        let mut values = leaves(&tree);
        values.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(
            values,
            vec![
                (
                    "/avatar/parameters/OGB/Plug/zw_ris_sps/LengthMult2".to_owned(),
                    // VRChat's values are 32-bit floats
                    Signal::Float(17.3739777_f32 as f64)
                ),
                ("/avatar/parameters/Toggle".to_owned(), Signal::Bool(true)),
            ]
        );
    }
}
