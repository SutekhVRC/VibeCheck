/*
 * OSCQuery mode: advertises VibeCheck so VRChat sends it OSC, and finds out
 * where VRChat listens.
 *
 * While the app runs in OSCQuery mode (not only while VibeCheck is on), a
 * task:
 * - advertises VibeCheck's OSC port over OSCQuery (mDNS + HTTP), so VRChat
 *   finds it and sends avatar parameters there;
 * - reads the port VRChat receives OSC on every few seconds, for sending
 *   battery levels (VRChat's OSCQuery service is the one the avatar catalog
 *   found, see avatar_catalog.rs).
 *
 * The UDP port itself is held by VibeCheck's OSC listeners (the session's
 * OSC input while on, the off-state listener while off), on every interface:
 * VRChat ignores the advertised OSC address and sends to the machine's LAN
 * address. VRChat also ignores the mDNS goodbye and keeps sending to a port
 * it has found, so the port is kept for the whole run and saved for the next
 * (OSCNetworking::oscquery_port).
 *
 * VRChat also keeps sending to its default port, 127.0.0.1:9001, whatever it
 * finds over OSCQuery. With nothing listening there, those sends fail, and
 * through Proton enough failures make VRChat stop sending OSC for the session
 * ("OSC sending failed too often"). So the task also holds 9001 while it's
 * free, without reading it: the OS drops what arrives once the buffer is
 * full, which VRChat never notices. The same data arrives on VibeCheck's own
 * port. If another app has 9001, it's the one listening; the task tries
 * again every few seconds in case it lets go.
 *
 * Changes are logged to the diagnostics log under `oscquery`.
 */

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::Arc;
use std::time::Duration;

use log::{info, warn};
use parking_lot::Mutex;
use tokio::task::JoinHandle;
use vibecheck_core::{Diagnostics, ModuleInstanceId};
use vrcoscquery::{discovery, OscQueryService};

use super::avatar_catalog::AvatarCatalog;
use super::DEFAULT_BIND;

/// The name VibeCheck advertises.
const SERVICE_NAME: &str = "VibeCheck";

/// How long reading VRChat's HOST_INFO may take.
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(3);

/// How often to read where VRChat receives OSC (and notice it closing or
/// restarting).
const LOOKUP_EVERY: Duration = Duration::from_secs(10);

/// The diagnostics log module (shared with the avatar parameter lookup).
const LOG_MODULE: &str = "oscquery";
const LOG_LABEL: &str = "OSCQuery";

/// What OSCQuery mode is doing, for the Settings page.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OscQueryStatus {
    /// The OSC port being advertised, once advertising started.
    pub advertising: Option<u16>,
    /// Where VRChat receives OSC, while VRChat is found.
    pub vrchat: Option<SocketAddrV4>,
    /// Why advertising failed, if it did.
    pub error: Option<String>,
    /// Whether VibeCheck is holding VRChat's default port (9001), so VRChat's
    /// sends there don't fail.
    pub holding_default_port: bool,
}

/// Runs OSCQuery mode's task. Cheap to clone; clones share the task.
#[derive(Clone, Default)]
pub struct OscQuery {
    status: Arc<Mutex<OscQueryStatus>>,
    task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl OscQuery {
    /// Starts advertising `osc_port` (replacing anything running) on
    /// `runtime`. VRChat is looked up through `catalog`.
    pub fn start(
        &self,
        runtime: &tokio::runtime::Handle,
        osc_port: u16,
        diagnostics: Diagnostics,
        catalog: AvatarCatalog,
    ) {
        self.stop();
        let status = self.status.clone();
        let task = runtime.spawn(run(osc_port, status, diagnostics, catalog));
        *self.task.lock() = Some(task);
    }

    /// Stops advertising. The task's service is dropped, which sends the
    /// mDNS goodbye (VRChat ignores it, so the port stays held by the OSC
    /// listener anyway).
    pub fn stop(&self) {
        if let Some(task) = self.task.lock().take() {
            task.abort();
        }
        *self.status.lock() = OscQueryStatus::default();
    }

    pub fn status(&self) -> OscQueryStatus {
        self.status.lock().clone()
    }

    /// Where VRChat receives OSC, while it's found.
    pub fn vrchat(&self) -> Option<SocketAddrV4> {
        self.status.lock().vrchat
    }
}

async fn run(
    osc_port: u16,
    status: Arc<Mutex<OscQueryStatus>>,
    diagnostics: Diagnostics,
    catalog: AvatarCatalog,
) {
    let module = ModuleInstanceId::new(LOG_MODULE);
    let log = |message: String| diagnostics.log(&module, LOG_LABEL, move || message);

    let osc_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, osc_port);
    let _service = match OscQueryService::start(SERVICE_NAME, osc_addr).await {
        Ok(service) => {
            info!("OSCQuery: advertising VibeCheck on port {}", osc_port);
            log(format!(
                "Advertising VibeCheck: VRChat will send OSC to port {osc_port}"
            ));
            status.lock().advertising = Some(osc_port);
            service
        }
        Err(e) => {
            warn!("OSCQuery: couldn't advertise VibeCheck: {}", e);
            log(format!("Couldn't advertise VibeCheck: {e}"));
            status.lock().error = Some(e.to_string());
            return;
        }
    };

    // VRChat's default port, held (not read) while it's free
    let mut default_port: Option<UdpSocket> = None;

    loop {
        if default_port.is_none() {
            if let Ok(socket) = UdpSocket::bind(DEFAULT_BIND) {
                info!(
                    "OSCQuery: holding {} for VRChat's default sends",
                    DEFAULT_BIND
                );
                log(format!(
                    "Holding {DEFAULT_BIND} (VRChat's default port), so VRChat's sends there don't fail"
                ));
                default_port = Some(socket);
                status.lock().holding_default_port = true;
            }
        }

        let found = find_vrchat_osc(&catalog).await;
        let previous = std::mem::replace(&mut status.lock().vrchat, found);
        if found != previous {
            match found {
                Some(addr) => {
                    info!("OSCQuery: VRChat found, receiving OSC on {}", addr);
                    log(format!("VRChat found: it receives OSC on {addr}"));
                }
                None => {
                    info!("OSCQuery: VRChat not found");
                    log("VRChat not found (not running, or OSC is off)".to_owned());
                }
            }
        }
        tokio::time::sleep(LOOKUP_EVERY).await;
    }
}

/// Where VRChat receives OSC, from its OSCQuery `HOST_INFO`. VRChat's
/// server is the one the avatar catalog found, so it isn't looked up twice.
async fn find_vrchat_osc(catalog: &AvatarCatalog) -> Option<SocketAddrV4> {
    let vrchat = catalog.vrchat()?;
    let info = discovery::host_info(&vrchat, LOOKUP_TIMEOUT).await.ok()?;
    // VRChat advertises 127.0.0.1 for itself; fall back to the address it
    // was found on if OSC_IP isn't a usable IPv4 address
    let ip = info
        .osc_ip
        .parse::<Ipv4Addr>()
        .ok()
        .filter(|ip| !ip.is_unspecified())
        .or(match vrchat.addr {
            SocketAddr::V4(addr) => Some(*addr.ip()),
            SocketAddr::V6(_) => None,
        })?;
    Some(SocketAddrV4::new(ip, info.osc_port))
}
