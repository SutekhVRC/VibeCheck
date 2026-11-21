# Errors

How problems reach the user. There are two paths, depending on whether the
frontend asked for the work.

## Command errors

A Tauri command that fails returns `Err`. VibeCheck's command errors
(`VCFeError`, and `VibeCheckFSError` for `clear_osc_config`, in
`src-tauri/src/vcore/errors.rs`) serialize as:

```json
{ "code": "ToyOnline", "message": "The toy is connected. Disconnect it first." }
```

- `message` is the error's `Display`, written for the user with `thiserror`'s
  `#[error("...")]`. Write it as a sentence that says what happened and, when
  there is one, what to do.
- `code` is the innermost variant's name (`strum::IntoStaticStr`). A wrapping
  variant such as `VCFeError::ToyManager(ToyHandlingError::ToyOnline)` gives
  the inner `ToyOnline`. The frontend checks it only for cases it handles
  itself, such as `VRChatOscNotFound` in Settings.

On the frontend, `errorText(e)` in `src/lib/errors.ts` turns any caught error
into text (a command error's message, an `Error`'s message, or a string), and
`errorCode(e)` reads the code. Toasts use `errorText`, never
`JSON.stringify`.

A command's error is shown by the code that called it, so it isn't also
reported (below), or it would show twice.

## Background problems: the reporter

Work nobody is waiting on (toy connections, the battery refresh, the OSC API,
config loading at startup) reports through an `ErrorReporter`
(`src-tauri/src/error_signal_handler/reporter.rs`):

```rust
reporter.warning("toys", "battery_socket", format!("Couldn't send battery levels to VRChat at {remote}: {e}"));
```

- `module` says where it happened. It's also the module shown in the
  [diagnostics](vibecheck-core.md) log.
- `code` names the problem. The same module and code are shown at most once
  per 30 seconds, so a failure inside a loop can't flood the screen; put
  details that change in the message, not the code.
- `error` is for something that stopped working; `warning` for something
  VibeCheck worked around (defaults, a retry).

Every notice is logged when reported. The notice handler then adds it to the
diagnostics log (while the Diagnostics page is open) and emits `fe_error`,
which the frontend shows as a toast.

`VibeCheckState::reporter` exists from the moment the state is created, so
startup code can report before the window opens: notices wait until
`notice_handler_start` runs. Tasks get a clone of it.

## Panics

`error_signal_handler/panic_hook.rs` installs a panic hook right after the
state is created. A panic in a background task only ends that task, so the
hook reports it like any other problem (code `panic`), with the message and
where it happened, then runs the default hook.

## The log file

`util/log_file.rs` sends the `log` crate's output to `VibeCheck.log` in the
app's log folder (`~/.local/share/com.vibecheck/logs` on Linux,
`%LOCALAPPDATA%\com.vibecheck\logs` on Windows). Settings > Troubleshooting
has an **Open log folder** button for bug reports.

- **Levels:** VibeCheck's own crates at info, everything else (Buttplug,
  mDNS, Tauri) at warning. Debug builds log everything at debug and also
  print to stderr. Don't log at info or above for every OSC message or tick:
  use `debug!`/`trace!` for anything per packet.
- **Performance:** logging formats the line and queues it; a background
  thread writes and flushes. A full queue drops lines (and notes how many)
  instead of blocking. The panic hook flushes before reporting.
- **Size:** `VibeCheck.log` is this session, `VibeCheck.old.log` the one
  before; past 5 MB the current file becomes the old one.
- **A second start:** the running app holds a lock on `VibeCheck.lock` in
  the log folder. Starting VibeCheck again while it runs (which hands over
  to the running one and quits) can't take the lock, so it leaves the log
  files alone; without it, the running app's log was moved aside and a
  third start overwrote it. The OS lets go of the lock however the app
  ends, crashes included.
- **Updates:** the log folder is in the same folder as the webview's data,
  which is cleared when VibeCheck starts as a new version
  (`util/webview_data.rs`). The log folder is on its `KEEP` list, so the
  first run after an update keeps its own log and the one before it.
- **Privacy:** every line is redacted before it's written: the home folder
  (to `~`), the computer and account names, VRChat `usr_`/`avtr_`/`wrld_`
  ids, Bluetooth addresses, and IP addresses other than loopback and
  `0.0.0.0`. Times are UTC. On Linux the file is readable by the user only.
  Redaction is a safety net: never log credentials, tokens or keys, from
  any device or service, at any level.

## unwrap and expect

Use `unwrap` only in tests. In app code:

- A failure that can happen (files, sockets, devices, channels to tasks that
  may have stopped, anything from the frontend) is handled: returned from a
  command, or reported.
- Something that is always set by that point (the toy manager after startup,
  a connected toy's config, the app handle) uses a helper such as
  `VibeCheckState::toy_manager()` or `VCToy::config_mut()`, or an `expect`
  whose message says why it can't fail.

## Settings files that can't be read

`Config.json` and toy configs that aren't valid are renamed to `*.json.bad`
before defaults are written over them, and the user is told. A config file
that can't be opened at all is left alone, and VibeCheck runs on defaults.
See `load_config_file` in `vcore/config/app.rs` and `load_toy_config` in
`toy_handling/toyops.rs`.
