# `vibecheck-core`

A library crate at `src-tauri/crates/vibecheck-core` that holds the module runtime. The app (`src-tauri`) depends on it by path.

The Cargo workspace is rooted at `src-tauri/Cargo.toml`:

| Member | Path | What it is |
|---|---|---|
| `vibecheck` | `src-tauri` | The Tauri app |
| `vibecheck-core` | `src-tauri/crates/vibecheck-core` | This crate |
| `osc-tool` | `src-tauri/crates/osc-tool` | Debug CLI to listen for, send and sweep OSC ([README](../../src-tauri/crates/osc-tool/README.md)). Not part of the app or installers |

The goal is a pipeline of modules:

```
Inputs  ->  Processors  ->  Safety layer  ->  Outputs
```

The app is being moved onto it piece by piece, without changing behaviour. See [Migration status](#migration-status) for what is in use today.

## Layout

| File | What's in it | Used by the app |
|---|---|---|
| `address.rs` | `ModuleInstanceId`, `PortId` | Yes |
| `safety.rs` | `SafetyLayer`, `SafetyCaps`, `ClampedCmd` | Yes |
| `slots.rs` | `CommandSlots` | Yes |
| `modules/buttplug.rs` | `ButtplugOutput`, `Actuator` | Yes |
| `modules/osc.rs` | `OscInput` | Yes |
| `module.rs` | `Input`, `Processor`, `Output` traits, `PortSpec`, `InputCtx` | `Input` and `Output` |
| `signal.rs` | `Signal`, `SignalPlane` | Written by `OscInput`; nothing reads it yet |
| `event.rs` | `Event`, `EventPlane`, `EventReceiver` | Yes: avatar change |
| `error.rs` | `Error`, `Result` | Yes |

## Addressing

Every port has a stable, typed address: a `ModuleInstanceId` plus a port name, written `module/port`.

```
toy:lovense_edge:0/Vibrator_1
```

Addresses never contain Buttplug device indexes, which change between sessions and reconnects. See [toy-command-path.md](toy-command-path.md#names) for how toy and motor names are built.

## Two data planes

| | Signal plane | Event plane |
|---|---|---|
| Carries | Continuous values: levels, intensities | Discrete happenings: avatar change, connect, panic |
| Delivery | Latest value wins; nothing queues | Queued and ordered |
| Type | `SignalPlane` (`signal.rs`) | `EventPlane` / `EventReceiver` (`event.rs`) |
| Rate | High (hundreds per second) | Low |

**Signal plane.** `SignalPlane` holds one slot per `PortId`. `write` overwrites it, `read` returns the newest value. Clones share the same slots. `Signal` is `Float(f64)` or `Bool(bool)`.

**Event plane.** `EventPlane::new()` returns a sender (cheap to clone, one per module) and the single `EventReceiver`. Every `Event` records:

- `cause`: the id of the event that led to it, if any;
- `depth`: how long the chain is so far. Root events are 0, and `publish_caused_by` sets the depth to the cause's depth + 1.

The safety layer uses `depth` to stop feedback loops.

## Module traits

| Trait | Does | Shape |
|---|---|---|
| `Input` | Produces signals and events | `async fn run(&mut self, ctx: InputCtx)` runs for the module's lifetime |
| `Processor` | Transforms signals once per graph tick, may emit events | `fn tick(&mut self, ins, dt, out, events)` |
| `Output` | Acts on a device or service | `async fn apply(&mut self, cmds: &[ClampedCmd])` |

Modules are ordinary Rust types compiled into the app. `async-trait` keeps `Input` and `Output` usable as trait objects.

`InputCtx` gives an input its own module id and lets it write signals and publish events on its own ports.

## Safety layer

Every command that reaches an output is a `ClampedCmd`. Its fields are private and it has no public constructor, so outside the crate the only way to get one is from `SafetyLayer`. An output can't receive, or invent, a command that skipped the user's caps.

`SafetyLayer::clamp(target, level, duration_ms)`:

- clamps the level to `0.0..=max_level`;
- turns a non-finite level (NaN, infinity) into 0;
- caps the duration at `max_duration_ms`.

`SafetyLayer::clamp_caused_by(cause, ...)` does the same, but returns `None` if `cause.depth` has reached `max_event_depth`.

| `SafetyCaps` | Default |
|---|---|
| `max_level` | 1.0 |
| `max_duration_ms` | 10 000 |
| `max_event_depth` | 3 |

Not built yet: arming, rate limits and cooldowns, panic stop, and refusing an event a rule caused for itself.

## Command slots

`CommandSlots` sits in front of an output. Producers write `ClampedCmd`s whenever they like; the output calls `take_changed()` at its own rate and gets every command written since the last call, **at most one per target port**, ordered by target. Writing the same value again counts as a change.

This is what lets several motors on one toy be driven in the same tick.

## Built-in modules

### `OscInput`

An `Input` that listens for OSC over UDP.

- `OscInput::bind(addr)` binds the socket, so a busy port is reported before anything runs.
- `with_tap(tx)` also sends every message except avatar changes to `tx`, in arrival order. `run` returns once the tap's receiver is dropped.
- `/avatar/change` (any address starting with it) is published as an `AvatarChange` event on the `avatar_change` port, with the avatar id as `Value::Text` when present.
- Any other message whose last argument is a float or bool writes it to the signal plane, on a port named after the full OSC address, e.g. `osc//avatar/parameters/Boop`.
- Bundles are unpacked. Packets that aren't valid OSC, and per-packet receive errors, are skipped.

`rosc` is re-exported as `modules::osc::rosc`, so the app uses the same `OscMessage` type.

### `ButtplugOutput`

An `Output` for one Buttplug device. Each port maps to an `Actuator`:

| `Actuator` | Buttplug call | Notes |
|---|---|---|
| `Scalar { index, actuator }` | `scalar` | Vibrate, constrict, oscillate, inflate, position, scalar rotate |
| `Rotate { index }` | `rotate` | Always clockwise |
| `Linear { index }` | `linear` | The command's `duration_ms` is the movement time |

`apply` sends every command, even if one fails, and returns the first error. A command for a port the output doesn't have is an `Error::UnknownPort`.

## Migration status

| Step | State |
|---|---|
| Workspace split, core types and traits | Done |
| Toy commands through `ButtplugOutput` with one slot per motor | Done ([toy-command-path.md](toy-command-path.md)) |
| OSC listener as an `Input` module, routing each message once, avatar change as an event | Done ([toy-command-path.md](toy-command-path.md#routing)) |
| SPS/TPS, Smooth, Rate, Constant and level tweaks as `Processor` modules | Not started |
| Each toy's graph generated from its existing config | Not started |

## Tests

```
cd src-tauri
cargo test -p vibecheck-core
```

Covers address formatting, latest-value-wins on the signal plane, event order and causality, clamping and the depth limit, that motors on one toy don't overwrite each other in `CommandSlots`, and `OscInput` over real UDP: tapping in order, bundles, avatar change events, signal writes, stopping when the tap closes, and reporting a busy port.
