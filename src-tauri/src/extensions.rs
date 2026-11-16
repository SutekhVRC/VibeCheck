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
 * - work to start once VibeCheck is running (`on_start`).
 *
 * Tauri only takes one command handler, built at compile time by
 * `generate_handler!`, so an app's commands come with their names: a call
 * for one of those names goes to the app's handler, anything else to
 * VibeCheck's. An app's command with the same name as one of VibeCheck's
 * replaces it.
 */

use tauri::{ipc::Invoke, AppHandle, Builder, Wry};

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
