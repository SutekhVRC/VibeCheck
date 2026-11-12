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

The app runs on it: OSC comes in through `OscInput`, each toy's processing is a `Graph` generated from its config and ticked at 100 Hz, and commands leave through the safety layer and `ButtplugOutput`. See [Migration status](#migration-status) for what's left.

The app only uses graphs generated from toy settings; there's no way to build graphs by hand yet.

## Layout

| File | What's in it | Used by the app |
|---|---|---|
| `address.rs` | `ModuleInstanceId`, `PortId` | Yes |
| `safety.rs` | `SafetyLayer`, `SafetyCaps`, `ClampedCmd` | Yes |
| `graph.rs` | `GraphBuilder`, `Graph` | Yes: one graph per toy |
| `slots.rs` | `CommandSlots` | Yes |
| `modules/buttplug.rs` | `ButtplugOutput`, `Actuator` | Yes |
| `modules/osc.rs` | `OscInput` | Yes |
| `modules/modes.rs` | `Mode`, `ModeSettings`, `RawMode`, `SmoothMode`, `RateMode`, `ConstantMode` | Yes |
| `modules/levels.rs` | `OutputLevels` | Yes |
| `module.rs` | `Input`, `Processor`, `Output` traits, `PortSpec`, `InputCtx`, `process_one` | Yes |
| `signal.rs` | `Signal`, `SignalPlane` | Yes: written by `OscInput` and SPS/TPS, read by graphs |
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

The plane also **remembers recent events** (the last 4096), so a chain can be traced back past its direct cause:

- `chain(event_id)` returns the sources of the event and every event before it, newest first, as far back as it remembers;
- `chain_includes(event_id, module)` says whether any of them came from `module`;
- `publish_caused_by_id(cause_id, ...)` publishes an event caused by one you only have the id of (e.g. from a command's `cause()`), working out its depth from the history.

The safety layer uses the depth and the chain to stop feedback loops (see [Feedback loops](#feedback-loops)).

## Module traits

| Trait | Does | Shape |
|---|---|---|
| `Input` | Produces signals and events | `async fn run(&mut self, ctx: InputCtx)` runs for the module's lifetime |
| `Processor` | Transforms signals once per graph tick, may emit events | `fn tick(&mut self, ins: &[Signal], dt, out: &mut [Option<Signal>], events: &mut Vec<EventOut>)` |
| `Output` | Acts on a device or service | `async fn apply(&mut self, cmds: &[ClampedCmd])` |

Modules are ordinary Rust types compiled into the app. `async-trait` keeps `Input` and `Output` usable as trait objects.

`InputCtx` gives an input its own module id and lets it write signals and publish events on its own ports.

A processor's `out` slots start as `None`. Setting one to `Some` writes a new value to that port; leaving it `None` leaves the port unchanged, e.g. while Smooth is still collecting values.

To emit an event, a processor pushes an `EventOut { port, kind, value }` naming one of its own event ports. It can't build a full `Event` itself, because ids and timestamps come from the event plane; the graph publishes it with the node's port as the source.

`process_one(processor, input, dt)` runs a single-input, single-output processor once and returns its output, for quick use outside a graph (and in tests). Events it emits are discarded.

## Graphs

`graph.rs` wires processors together by port and runs them on a tick.

```rust
let mut b = GraphBuilder::new();
b.add_processor(id("mode"), modes::build(Mode::Raw, settings))?;
b.add_processor(id("levels"), Box::new(levels))?;
b.connect(PortId::new(osc, "/avatar/parameters/Boop"), PortId::new(id("mode"), "in"))?;
b.connect(PortId::new(id("mode"), "out"), PortId::new(id("levels"), "in"))?;
b.add_sink(PortId::new(id("levels"), "out"), motor_port, None)?;
let mut graph = b.build()?;

// every tick:
for cmd in graph.tick(&signals, dt, &safety, &mut events) {
    slots.write(cmd);
}
```

**Building** (`GraphBuilder`):

- `add_processor(id, processor)` adds a node; its ports are the processor's signal ports.
- `connect(from, to)` wires a source into a node input. If `from` belongs to a node, it's that node's output; otherwise it's read from the signal plane (e.g. an OSC parameter or SPS/TPS output). Each input takes exactly one source.
- `add_sink(from, target, duration_ms)` sends a node output to a target port, such as a motor.
- `build()` rejects unknown nodes or ports, inputs wired twice or not at all, sinks fed from something other than a node, and cycles. It orders nodes so each runs after the nodes it reads from.

**Ticking** (`Graph::tick`):

1. Nodes run in dependency order, each reading the latest value of its inputs. A node only runs once **all** its inputs have a value, so nothing happens before the first input arrives.
2. A node output keeps its last value until the node writes a new one (`None` means unchanged).
3. Each sink whose value **changed** since it last fired goes through `SafetyLayer::clamp` and is returned as a `ClampedCmd`, unless the safety layer refuses it. Unchanged sinks send nothing, so idle graphs don't flood outputs. A refused value still counts as the sink's last value, so it isn't retried or replayed later (after a resume or arming): the sink waits for its next change.

Events processors emit during a tick are published on the `EventPlane` passed to `tick`, with the node's port as the source (e.g. `proc:avg/crossed`).

`Graph::tick` doesn't sleep or spawn anything; the caller decides the rate. The app ticks at 100 Hz (`graph_runner.rs`) and passes the real time since the last tick as `dt`, capped at 0.1 s.

## Safety layer

Every command that reaches an output is a `ClampedCmd`. Its fields are private and it has no public constructor, so outside the crate the only way to get one is from `SafetyLayer`. An output can't receive, or invent, a command that skipped the user's caps.

Levels in the safety layer are **intensities**: 0 is off and 1 is full power for every toy. Per-actuator Flip Input is applied after it, by the output, so the caps mean the same thing for toys whose motors run backwards.

A `SafetyLayer` is cheap to clone, and **clones share all their state** (caps, panic, arming, rate-limit timers). The app keeps one in its state and hands clones to every session and the Test button, so changing it (master intensity, panic, arming) affects toys that are already running.

### Levels

`SafetyLayer::limit(level)`:

1. 0 while panicked;
2. otherwise, a non-finite level (NaN, infinity) becomes 0 and the rest is clamped to `0.0..=1.0`;
3. scaled by the **master intensity** (`intensity`), keeping the full range: at 0.6, 1.0 becomes 0.6 and 0.5 becomes 0.3;
4. capped at `max_level`.

The intensity scales rather than clips because a ceiling flattens everything above it (70% and 100% would feel the same), while scaling keeps the feel of the whole range, just gentler. Per-motor hard ceilings are what Range is for.

### Commands

`SafetyLayer::clamp(target, level, duration_ms) -> Option<ClampedCmd>` applies `limit`, caps the duration at `max_duration_ms`, and **refuses** (`None`):

- while panicked;
- if the target's module has a `TargetPolicy` with `requires_arming` and isn't armed;
- if the target's `min_interval` hasn't passed since its last command (per port).

`clamp_caused_by(cause, ...)` also refuses if `cause.depth` has reached `max_event_depth`. Modules acting because of an event should use `clamp_in_response` instead, which also stops self-triggering (below).

Every `ClampedCmd` remembers the panic count (epoch) it was made in. **Outputs must call `admits(&cmd)` right before sending**: it's false if a panic has happened since, or if the target has been disarmed. That's how commands already queued when you panic (or disarm) get dropped instead of sent.

### Feedback loops

A module that acts because of an event (a rule, or a processor reacting to events) should ask with:

```rust
safety.clamp_in_response(&events, &trigger, &requester, target, level, duration_ms)
```

It's refused (`None`) if:

1. **the chain is too long**: `trigger.depth` has reached `max_event_depth` (3 by default), however many different modules are involved;
2. **the requester is already in the chain**: `requester` published `trigger` or any event before it. That means it would be reacting to the result of its own earlier action, which is how a feedback loop starts, so it's stopped the first time round rather than after a few laps;
3. or anything `clamp` refuses (panic, not armed, rate limit).

When it's allowed, the safety layer publishes an **`Acted`** event from the requester (on its `acted` port, caused by `trigger`), and the command carries that event as its `cause()`. That's the trace that makes loops visible later on.

**Outputs that report what they did should publish caused by `cmd.cause()`** (`publish_caused_by_id`). The chain then leads back through whoever asked:

```
toy pattern ─▶ Acted (rule A) ─▶ output reports it ─▶ rule A reacts ✗ refused
                                                   └▶ rule B reacts ✓ allowed (depth permitting)
```

Nothing in the free app acts in response to events yet, so the guard is ready for modules that will.

### Panic stop

- `panic()`: refuse every new command, make every existing command stale, and disarm everything.
- `resume()`: lift it. Nothing is re-armed and nothing is replayed.
- `is_panicked()`.

Stopping the devices themselves is the caller's job (the app calls Buttplug's stop-all).

### Arming and rate limits

`set_policy(module, TargetPolicy { requires_arming, min_interval })` marks an output module as risky and/or rate-limited. Modules without a policy (toys) only get the caps and panic stop.

- `arm(module)` (refused while panicked), `disarm(module)`, `disarm_all()`, `is_armed(module)`.
- Arming lasts until disarmed or a panic. Nothing is armed by default.

Nothing in the free app has a policy yet; this is ready for outputs that need it.

| `SafetyCaps` | Default |
|---|---|
| `intensity` | 1.0 (the app sets it from the saved master intensity) |
| `max_level` | 1.0 |
| `max_duration_ms` | 10 000 |
| `max_event_depth` | 3 |

Not here: checking imported rules and per-rule cooldowns, which belong with rules themselves.

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

### Processing modes (`modules::modes`)

Single-input, single-output processors (`in` → `out`), meant to run on a fixed tick. `modes::build(mode, settings)` makes one from a `Mode` and `ModeSettings`.

| Mode | Float in | Bool in |
|---|---|---|
| `RawMode` | passed through | 1.0 or 0.0 |
| `SmoothMode` | time-weighted mean over the last `smooth_window` of ticks, rounded to hundredths. 0 (1 with flip) passes straight through and restarts the average | ignored (no output) |
| `RateMode` | adds the movement since the previous tick to an internal level (capped at 1), then decays it by `rate_decay` per 150 ms of `dt`, every tick. At rest (<= 0, or >= 1 with flip) it resets and outputs 0 | ignored |
| `ConstantMode` | `constant_level` if >= 0.5, else 0 | `constant_level` if true, else 0 |

Raw and Constant behave exactly as the app's original modes. Smooth and Rate were redefined in time when the graph moved to a fixed tick: they used to work per OSC message (average the last N messages; decay only when a message arrived), which made them depend on VRChat's send rate and let Rate stick at its last level. The tests pin each mode's behaviour.

### `OutputLevels` (`modules::levels`)

Idle Speed and Range, as a processor and as `apply(level)`:

1. exactly 0 becomes `idle`;
2. anything else is clamped to `min..=max` (a reversed range is tolerated instead of panicking).

Flip Input isn't applied here, so levels stay intensities through the safety layer. `levels::flip(level)` (`1 - level`, rounded to hundredths) is what outputs use to apply it.

### `ButtplugOutput`

An `Output` for one Buttplug device. Each port maps to an `ActuatorSpec`: an `Actuator` plus `flip`. Flipped actuators get `1 - level` (`device_level`), applied here after the safety layer. `set_actuators` replaces the mapping, e.g. when Flip Input changes.

| `Actuator` | Buttplug call | Notes |
|---|---|---|
| `Scalar { index, actuator }` | `scalar` | Vibrate, constrict, oscillate, inflate, position, scalar rotate |
| `Rotate { index }` | `rotate` | Always clockwise |
| `Linear { index }` | `linear` | The command's `duration_ms` is the movement time |

`apply` sends every command, even if one fails, and returns the first error. A command for a port the output doesn't have is an `Error::UnknownPort`.

With `with_events(module, events)`, it publishes `EventKind::OutputError` on its `error` port when sending **starts** failing (value: the error as text) and when it **works again** (value: `None`). It doesn't fire on every failed send, so a disconnected toy at 20 Hz doesn't flood the event plane.

## Migration status

| Step | State |
|---|---|
| Workspace split, core types and traits | Done |
| Toy commands through `ButtplugOutput` with one slot per motor | Done ([toy-command-path.md](toy-command-path.md)) |
| OSC listener as an `Input` module, routing each message once, avatar change as an event | Done ([toy-command-path.md](toy-command-path.md#routing)) |
| Smooth, Rate, Constant and level tweaks as `Processor` modules | Done |
| SPS/TPS as `Processor` modules | Waiting for the planned SPS/TPS rewrite; the current input processors stay in the app |
| Each toy's graph generated from its existing config, ticked at a fixed rate | Done ([toy-command-path.md](toy-command-path.md#the-graph)) |
| Output events from `ButtplugOutput` | Error and recovery done. Activity events ("motor started/stopped") wait for a design that handles Flip Input, where level 0 isn't "off" |
| Processor events published from the graph | Done |

## Tests

```
cd src-tauri
cargo test -p vibecheck-core
```

Covers address formatting, latest-value-wins on the signal plane, event order and causality, clamping and the depth limit, tracing event chains through the remembered history, refusing a module that reacts to the result of its own action, that motors on one toy don't overwrite each other in `CommandSlots`, `OscInput` over real UDP (tapping in order, bundles, avatar change events, signal writes, stopping when the tap closes, reporting a busy port), each processing mode's behaviour, `OutputLevels`, `ButtplugOutput`'s error events firing only on change, and the graph (publishing processor events from their node, dependency order, holding values until inputs exist, outputs keeping their last value, sinks only firing on change, durations and the safety layer, and rejecting bad wiring).

The app's tests (`cargo test -p vibecheck`) cover the SPS/TPS routing table, `run_input_processors`, the graph runner task, and the generated toy graphs end to end: nothing before the first input, modes loaded from config, one parameter driving several motors, Range and Idle after the mode, the first matching parameter winning, disabled features, Linear speed, the f32 rounding, Smooth following over its window, Rate winding down after the input stops, and SPS/TPS output chains.
