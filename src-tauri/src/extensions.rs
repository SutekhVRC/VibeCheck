/*
 * What an app built on this crate can add to VibeCheck.
 *
 * The VibeCheck app (main.rs) runs the library with no extensions. Another
 * app can depend on this crate, call `vibecheck_lib::run` with its own Tauri
 * context (its own tauri.conf.json and frontend) and add:
 *
 * - Tauri commands, next to VibeCheck's own (`commands`);
 * - anything else on the Tauri builder: plugins, managed state
 *   (`builder`);
 * - work to start once VibeCheck is running (`on_start`);
 * - its own tray icon (`tray_icon`), as the app icon comes from its own
 *   tauri.conf.json;
 * - its own graphs and outputs in every listening session (`session`):
 *   each time VibeCheck turns on, `SessionExtension::start` gets the
 *   session's signal and event planes, the safety layer and a way to add
 *   graphs to the session's graph runner; `stop` runs just before the
 *   session ends (VibeCheck turned off or reset), while its tasks still run.
 *
 * Tauri only takes one command handler, built at compile time by
 * `generate_handler!`, so an app's commands come with their names: a call
 * for one of those names goes to the app's handler, anything else to
 * VibeCheck's. An app's command with the same name as one of VibeCheck's
 * replaces it.
 */

use std::net::SocketAddrV4;

use tauri::{image::Image, ipc::Invoke, AppHandle, Builder, Manager, Wry};
use tokio::{runtime::Handle, sync::mpsc::UnboundedSender};
use vibecheck_core::{
    graph::Graph, CommandSlots, Diagnostics, EventPlane, ModuleInstanceId, SafetyLayer, SignalPlane,
};

use crate::{
    toy_handling::runtime::graph_runner::{GraphControl, GraphKey, GraphView},
    vcore::state::VCStateMutex,
};

pub(crate) type Handler = Box<dyn Fn(Invoke<Wry>) -> bool + Send + Sync>;
type BuilderHook = Box<dyn FnOnce(Builder<Wry>) -> Builder<Wry>>;
type StartHook = Box<dyn FnOnce(&AppHandle) + Send>;

/// What an app adds to VibeCheck; see the top of this file. The default adds
/// nothing.
#[derive(Default)]
pub struct Extensions {
    pub(crate) commands: Vec<(Vec<&'static str>, Handler)>,
    pub(crate) builder: Vec<BuilderHook>,
    pub(crate) on_start: Vec<StartHook>,
    pub(crate) sessions: Vec<Box<dyn SessionExtension>>,
    pub(crate) tray_icon: Option<Image<'static>>,
}

impl Extensions {
    /// Adds Tauri commands. `names` are the commands `handler` handles; make
    /// it with `tauri::generate_handler!` from the same list.
    pub fn commands(
        mut self,
        names: &[&'static str],
        handler: impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static,
    ) -> Self {
        self.commands.push((names.to_vec(), Box::new(handler)));
        self
    }

    /// Changes the Tauri builder before the app is built, e.g. to add a
    /// plugin or managed state. Don't set the invoke handler here: use
    /// `commands`.
    pub fn builder(mut self, hook: impl FnOnce(Builder<Wry>) -> Builder<Wry> + 'static) -> Self {
        self.builder.push(Box::new(hook));
        self
    }

    /// Runs `start` once VibeCheck's own services have started, before the
    /// event loop.
    pub fn on_start(mut self, start: impl FnOnce(&AppHandle) + Send + 'static) -> Self {
        self.on_start.push(Box::new(start));
        self
    }

    /// Uses `icon` in the tray instead of VibeCheck's, e.g.
    /// `tauri::include_image!("icons/tray.png")`. 64 px reads well on both
    /// Windows and Linux trays.
    pub fn tray_icon(mut self, icon: Image<'static>) -> Self {
        self.tray_icon = Some(icon);
        self
    }

    /// Adds graphs and outputs to every listening session (see the top of
    /// this file).
    pub fn session(mut self, extension: impl SessionExtension + 'static) -> Self {
        self.sessions.push(Box::new(extension));
        self
    }
}

/// An app's part in each listening session.
pub trait SessionExtension: Send {
    /// VibeCheck was turned on: a new session started. Add graphs with
    /// [`SessionContext::set_graph`] and spawn output tasks on
    /// [`SessionContext::runtime`]; they're stopped with the session.
    fn start(&mut self, ctx: &SessionContext);

    /// The session is about to end (VibeCheck turned off or reset). Its
    /// runtime still runs; stop devices here. A panic stop doesn't end the
    /// session: outputs check [`SafetyLayer::is_panicked`] themselves.
    fn stop(&mut self) {}
}

/// What a [`SessionExtension`] gets from a listening session. Cheap to
/// clone, and usable from the session's tasks.
#[derive(Clone)]
pub struct SessionContext {
    pub(crate) app: AppHandle,
    pub(crate) runtime: Handle,
    pub(crate) signals: SignalPlane,
    pub(crate) events: EventPlane,
    pub(crate) safety: SafetyLayer,
    pub(crate) diagnostics: Diagnostics,
    pub(crate) graphs: UnboundedSender<GraphControl>,
}

impl SessionContext {
    /// The app.
    pub fn app(&self) -> &AppHandle {
        &self.app
    }

    /// The session's tokio runtime. Tasks spawned on it end with the session.
    pub fn runtime(&self) -> &Handle {
        &self.runtime
    }

    /// Avatar parameters and every other signal (module `osc`, port = OSC
    /// address).
    pub fn signals(&self) -> &SignalPlane {
        &self.signals
    }

    /// The session's events (avatar change, connections, activity, ...).
    pub fn events(&self) -> &EventPlane {
        &self.events
    }

    /// The app-wide safety layer: caps, panic stop, arming, rate limits.
    pub fn safety(&self) -> &SafetyLayer {
        &self.safety
    }

    /// The diagnostics switch and log.
    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    /// Where VibeCheck sends OSC to VRChat right now: the address set in
    /// Settings, or in Automatic mode the one VRChat advertises (it can
    /// change while the session runs, so ask again rather than keeping it).
    /// Send from your own socket bound to port 0; never bind or advertise
    /// VRChat's ports.
    pub fn osc_remote(&self) -> SocketAddrV4 {
        self.app.state::<VCStateMutex>().0.lock().osc_remote()
    }

    /// Runs `graph` in the session's graph runner (100 Hz) under `name`,
    /// replacing any graph this app set under that name. Its commands go to
    /// `slots`, for the app's output to send. Like toy graphs, it's reset
    /// on avatar change.
    pub fn set_graph(&self, name: impl Into<String>, graph: Graph, slots: CommandSlots) {
        let _ = self.graphs.send(GraphControl::Set {
            key: GraphKey::App(name.into()),
            graph,
            slots,
        });
    }

    /// Stops running the graph set under `name`.
    pub fn remove_graph(&self, name: impl Into<String>) {
        let _ = self
            .graphs
            .send(GraphControl::Remove(GraphKey::App(name.into())));
    }

    /// Makes `slots` the place for commands to `module`'s ports, from any
    /// graph in the session (toys are registered already). A graph can then
    /// drive this output and others at once; see graph_runner.rs.
    pub fn set_output(&self, module: ModuleInstanceId, slots: CommandSlots) {
        let _ = self.graphs.send(GraphControl::SetOutput { module, slots });
    }

    /// Stops routing commands for `module` (see [`set_output`](Self::set_output)).
    pub fn remove_output(&self, module: ModuleInstanceId) {
        let _ = self.graphs.send(GraphControl::RemoveOutput(module));
    }

    /// Every graph the session runs now (each toy's and every app's), with
    /// its shape and current values. Empty if the session is ending.
    pub async fn graphs(&self) -> Vec<GraphView> {
        let (reply, answer) = tokio::sync::oneshot::channel();
        if self.graphs.send(GraphControl::Describe(reply)).is_err() {
            return Vec::new();
        }
        answer.await.unwrap_or_default()
    }
}

/// The handler for a command name, if an app added one.
pub(crate) fn handler_for<'a>(
    commands: &'a [(Vec<&'static str>, Handler)],
    command: &str,
) -> Option<&'a Handler> {
    commands
        .iter()
        .find(|(names, _)| names.contains(&command))
        .map(|(_, handler)| handler)
}
