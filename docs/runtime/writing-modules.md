# Writing modules

How to write a module for VibeCheck: an input that brings data in, a processor that turns values into levels, or an output that drives a device. For how the runtime around them works, see [`vibecheck-core`](vibecheck-core.md).

Modules are ordinary Rust types compiled into the app. There's no plugin loading: a module is added by wiring it in code (see [Getting it into the app](#getting-it-into-the-app)). For anything bigger than a small fix, open an issue first so we can agree on the shape.

## Which kind?

| You want to | Write | Trait | Example |
|---|---|---|---|
| Bring values in from outside (a network protocol, audio, a sensor) | An input | `Input` | OSC from VRChat (`modules/osc.rs`) |
| Turn values into a level, or notice something about them | A processor | `Processor` | Smooth, Rate (`modules/modes.rs`), SPS and TPS (app) |
| Make a device or service do something | An output | `Output` | Toys over Buttplug (`modules/buttplug.rs`) |

Each trait in `src-tauri/crates/vibecheck-core/src/module.rs` has a complete, working example in its docs. They're doctests, so `cargo test` compiles and runs them and they can't go stale:

- `Input`: values sent over a channel from elsewhere in the program;
- `Processor`: a threshold with hysteresis that outputs on/off and publishes an event when it flips;
- `Output`: a recorder that keeps the level sent to each port, plus the panic check.

`cargo doc -p vibecheck-core --open` (from `src-tauri/`) shows them, with the rest of the API.

## Rules every module follows

- **Ports.** Every module lists its ports in `ports()`: name, direction (in or out) and kind (signal or event). Names are unique within the module and stable: saved settings and graphs refer to them. Never put a Buttplug device index or anything else that changes between runs in a name. A port's full address is `module/port`, e.g. `toy:lovense_lush:0/Vibrator_0` (see [Addressing](vibecheck-core.md#addressing)).
- **Signals are latest-value.** A signal port holds one value; writing replaces it, and nothing queues. Readers may skip values that were overwritten in between. If every value matters (a sequence of messages), use events or your own channel.
- **Levels are intensities.** A level is 0.0 to 1.0, where 0 is off for every device. Device quirks such as Flip Input are applied by the output, after the safety layer, so caps like the master intensity mean the same thing everywhere.
- **Everything reaching a device goes through the safety layer.** Outputs only ever receive `ClampedCmd`s, which only `SafetyLayer` can create. There's no way around it, on purpose.
- **Don't block.** `Processor::tick` runs 100 times a second for every toy on one task: no I/O, sleeping or locks held for long. Inputs and outputs are `async`; use async I/O, or `spawn_blocking` for blocking libraries.
- **Report what changed, not every time.** Events fire when a state changes (failing, then working again), not on every attempt, so a disconnected device doesn't flood the event plane.
- **Docs.** `vibecheck-core` has `#![warn(missing_docs)]`: every public item needs a doc comment.

### Events in use

Publish with an existing kind and port name where one fits, so whatever reacts to events understands yours too:

| Kind | Port | Value | Published by |
|---|---|---|---|
| `AvatarChange` | `avatar_change` | `Text`: the avatar id | `OscInput` |
| `Connection` | `connection` | `Bool`: true connected, false disconnected | The app, for VRChat (module `vrchat`) and each toy |
| `Activity` | `activity` | `Bool`: true started, false stopped | `ButtplugOutput`; stopped means every actuator at 0 for a second |
| `OutputError` | `error` | `Text` when it starts failing, `None` when it works again | `ButtplugOutput` |
| `Threshold` | the processor's own | `Bool` or `Float` | Processors that watch a level |
| `Acted` | `acted` | `None` | The safety layer, when it admits an action taken in response to an event |
| `Custom(name)` | your own | anything | For events no kind above fits |

While Diagnostics is open, the app logs `Connection`, `Activity` and `OutputError` events in its log (`runtime/osc_routing.rs`, `describe`), so you can see them while testing.

## Writing a processor

A processor runs as a node in a [graph](vibecheck-core.md#graphs). Each tick it gets the latest value of each signal input and may write its outputs.

1. **`ports()`**: its signal inputs and outputs, in order, plus any event ports. `tick`'s `ins` and `out` follow the same order.
2. **`tick(ins, dt, out, events)`**:
   - `dt` is the real time since the last tick, in seconds (capped at 0.1). Use it for anything time-based instead of counting ticks, so behaviour doesn't change with the tick rate.
   - Each `out` slot starts as `None`. Set it to write a value; leave it `None` to keep the last one (e.g. while still collecting values).
   - Push an `EventOut` naming one of your event ports to publish an event. The graph publishes it with your node as the source.
   - A node only runs once every wired input has a value.
3. **`reset()`**: forget everything learned from past input, keep settings. Called on avatar change, so nothing from the old avatar carries over.
4. **Inputs that aren't known in advance** (e.g. every SPS parameter an avatar sends): declare `watches()` with a module and a port-name prefix, and handle the values in `watched()`. See [Watching ports](vibecheck-core.md#watching-ports).

**Testing.** For a single-input, single-output processor, `process_one(&mut p, input, dt)` runs one tick and returns the output. For events, several inputs or watches, build a small graph as the `Processor` example does, and read events with `EventReceiver::try_recv`. Pin the behaviour users feel (when it starts, how fast it winds down, what jitter does), not the internals.

## Writing an input

An input runs as its own task: `run(ctx)` loops for the module's life and returns when it's done (or on an error that ends it).

- **`InputCtx`** is how it talks to the runtime: `write_signal(port, value)`, `publish(port, kind, value)`, `clear_signals()` (forget everything it wrote, e.g. when a new avatar replaces the old one), and `log(label, value)` for the Diagnostics log. `log` only formats while Diagnostics is on, so it costs nothing otherwise.
- **Fail early.** Open sockets or devices in a constructor (like `OscInput::bind`), so the app can tell the user about a busy port before anything starts.
- **Keep order where it matters.** Anything that must happen in message order (like clearing old values before the new avatar's arrive) belongs in the input, not in something reacting to an event later.
- **Diagnostics** should show what never reaches the signal plane (messages nothing uses, types it doesn't keep), so a user can see what's arriving.

**Testing.** Make an `InputCtx` over a fresh `SignalPlane` and `EventPlane`, run the input, then read the plane and the receiver, as the `Input` example does. `OscInput`'s tests send real UDP to a port bound on `127.0.0.1:0`.

## Writing an output

An output gets batches of `ClampedCmd`s in `apply` and makes the device do them.

- **Commands:** `target()` (which of your ports), `level()` (an intensity, already capped) and `duration_ms()` (for timed actions, such as a stroker's movement time). Apply device quirks (like Flip Input) here, after the safety layer.
- **Send every command,** even if one fails, and return the first error. An unknown port is `Error::UnknownPort`; a device that can't be reached is `Error::OutputUnavailable`.
- **Rate.** Outputs normally sit behind `CommandSlots`: whatever produces commands writes them at any rate, and the output's task drains `take_changed()` at the device's own rate, getting at most one command per port.
- **Panic and disarm.** Whatever drains the slots calls `SafetyLayer::admits(&cmd)` right before sending and drops what it refuses. That's what stops a command queued before a panic from going out.
- **Events.** Publish `OutputError` when sending starts failing and when it works again, and `Activity` if starting and stopping means something for the device. If you report the result of a command, publish it caused by `cmd.cause()` (`EventPlane::publish_caused_by_id`), so the safety layer can see feedback loops.
- **Devices that need extra care** get a policy: `SafetyLayer::set_policy(module, TargetPolicy { requires_arming, min_interval })`. With `requires_arming`, nothing is sent until the user arms it, and it's disarmed automatically on panic, on failure, when it disconnects and when VRChat goes away. `min_interval` limits how often each port may act. See [Arming and rate limits](vibecheck-core.md#arming-and-rate-limits).

**Testing.** Get commands from a `SafetyLayer` (`clamp`) and call `apply`, as the `Output` example does. Test with a fake device where you can; `ButtplugOutput` tests its pure parts (layouts, flip, error and activity state) without hardware.

## Getting it into the app

There's no module registry yet, so each kind is wired where the app builds its pipeline. All of it is in `src-tauri/src/toy_handling/`.

- **Processors used by toys** are added to each toy's graph in `runtime/toy_graph.rs` (`build_toy_graph`), from the toy's saved settings. A new processing mode goes in `modules/modes.rs` (`Mode` and `modes::build`) plus the app's `ProcessingMode`. A new input processor implements the app's `InputProcessor` trait (`input_processor/mod.rs`, which explains the steps) and gets a variant in `PenetrationSystemType`.
- **Inputs** are started per listening session in `runtime/toy_management_handler.rs`, next to `start_osc`, each with its own `InputCtx` and module id, so they share the session's signal and event planes.
- **Outputs**: toys get their `ButtplugOutput` in `runtime/toy_output.rs`, drained by the emitter task in `runtime/toy_emitter_thread.rs`. Another kind of output needs its own task like it.
- **Settings** the user can change live in the app's config types, which are shared with the frontend through `ts-rs` (`cargo test` regenerates `src-tauri/bindings`), and in the React UI under `src/`.

## Before opening a pull request

From `src-tauri/`:

```
cargo test --workspace
cargo fmt --all --check
cargo clippy -p vibecheck-core --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc -p vibecheck-core --no-deps
```

`vibecheck-core` is clean under clippy and rustdoc; keep it that way. The app crate still has older clippy warnings: don't add new ones.

Then update [`vibecheck-core.md`](vibecheck-core.md) (and [toy-command-path.md](toy-command-path.md) if the path a level takes changed), and bump the version in `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`.
