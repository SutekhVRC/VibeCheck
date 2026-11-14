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
| `graph.rs` | `GraphBuilder`, `Graph`, `GraphSnapshot`, `SinkValue` | Yes: one graph per toy |
| `diagnostics.rs` | `Diagnostics`, `LogEntry` | Yes: the diagnostics view |
| `slots.rs` | `CommandSlots` | Yes |
| `modules/buttplug.rs` | `ButtplugOutput`, `Actuator`, `LegacyLayout`, `send_to_actuator` | Yes |
| `modules/osc.rs` | `OscInput` | Yes |
| `modules/modes.rs` | `Mode`, `ModeSettings`, `RawMode`, `SmoothMode`, `RateMode`, `ConstantMode` | Yes |
| `modules/levels.rs` | `OutputLevels` | Yes |
| `module.rs` | `Input`, `Processor`, `Output` traits, `PortSpec`, `InputCtx`, `process_one` | Yes |
| `signal.rs` | `Signal`, `SignalPlane`, `SignalSeq`, `PortValue` | Yes: written by `OscInput`, read by graphs |
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

The plane also numbers its writes, so a reader can ask what changed: `changed_since(module, prefix, since)` returns the ports of `module` whose name starts with `prefix` and that were written after `since`, each once with its latest value, oldest write first, plus the `SignalSeq` to pass next time (`SignalSeq::default()` is "from the start"). Graphs use it for [watching ports](#watching-ports). It scans every slot, which is fine for the hundreds of OSC parameters an avatar sends.

For views that show everything, `changed_since_all(since)` does the same for every module, as `PortValue { port, signal, writes }`, where `writes` counts the port's writes since it was last cleared (for an update rate). `clears()` counts how many times ports have been cleared, so a view keeping its own copy knows when to start over (e.g. after an avatar change).

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
| `Processor` | Transforms signals once per graph tick, may emit events | `fn tick(&mut self, ins: &[Signal], dt, out: &mut [Option<Signal>], events: &mut Vec<EventOut>)`, plus optional `watches` / `watched` and `reset` |
| `Output` | Acts on a device or service | `async fn apply(&mut self, cmds: &[ClampedCmd])` |

Modules are ordinary Rust types compiled into the app. `async-trait` keeps `Input` and `Output` usable as trait objects.

`InputCtx` gives an input its own module id and lets it write signals and publish events on its own ports. `with_diagnostics(d)` connects it to a [`Diagnostics`](#diagnostics) log, which `log(label, value)` writes to while diagnostics are on.

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
- `connect(from, to)` wires a source into a node input. If `from` belongs to a node, it's that node's output; otherwise it's read from the signal plane (e.g. an OSC parameter). Each input takes exactly one source.
- `add_sink(from, target, duration_ms)` sends a node output to a target port, such as a motor.
- `build()` rejects unknown nodes or ports, inputs wired twice or not at all, sinks fed from something other than a node, and cycles. It orders nodes so each runs after the nodes it reads from.

**Ticking** (`Graph::tick`):

1. Nodes run in dependency order, each reading the latest value of its inputs. A watching node first gets its [watched](#watching-ports) changes. A node only runs once **all** its inputs have a value, so nothing happens before the first input arrives. A node with no inputs (e.g. one that only watches) runs every tick.
2. A node output keeps its last value until the node writes a new one (`None` means unchanged).
3. Each sink whose value **changed** since it last fired goes through `SafetyLayer::clamp` and is returned as a `ClampedCmd`, unless the safety layer refuses it. Unchanged sinks send nothing, so idle graphs don't flood outputs. A refused value still counts as the sink's last value, so it isn't retried or replayed later (after a resume or arming): the sink waits for its next change.

Events processors emit during a tick are published on the `EventPlane` passed to `tick`, with the node's port as the source (e.g. `proc:avg/crossed`).

**Resetting** (`Graph::reset`) starts the graph over without rebuilding it: every processor's `reset()` is called (`Processor::reset` defaults to nothing; Smooth and Rate clear their state), node outputs are cleared, and sinks forget their last value, so the next value is sent even if it repeats the old one. Watching nodes are given every watched value still on the plane again on the next tick, as a new graph would be, so a reset that runs late doesn't lose values written just before it. Clear the plane first for values that should be forgotten. The app resets every graph on avatar change.

### Watching ports

Some processors can't have their inputs wired when the graph is built, because they don't know them yet: an SPS processor reads one set of parameters per socket, whatever sockets the avatar has. Such a processor declares **watches** instead:

```rust
fn watches(&self) -> Vec<Watch> {
    vec![Watch::new(ModuleInstanceId::new("osc"), "/avatar/parameters/OGB/")]
}

fn watched(&mut self, changed: &[(PortId, Signal)]) {
    for (port, value) in changed { /* remember it */ }
}
```

- `watches` is asked once, when the node is added. A `Watch` is a module plus a port-name prefix.
- Each tick, before `tick`, the graph calls `watched` with every matching port written since the node's last tick: each port once, with its latest value, oldest write first (watch by watch if there are several). It isn't called when nothing changed. The first time, it gets every matching value already on the plane, so a rebuilt graph picks up current values, as wired inputs do.
- Watched values are delivered even if the node's wired inputs aren't ready yet, so none are missed.
- Like any latest-value input, a port written twice within one tick is seen once, with its last value.

The app's input processors (SPS/TPS) run this way; see [toy-command-path.md](toy-command-path.md#input-processors).

`Graph::tick` doesn't sleep or spawn anything; the caller decides the rate. The app ticks at 100 Hz (`graph_runner.rs`) and passes the real time since the last tick as `dt`, capped at 0.1 s.

Node outputs live inside the graph, not on the signal plane. `Graph::snapshot()` copies them out for views (diagnostics, later a graph editor): every node output with its latest value (`None` before its first), and every sink as a `SinkValue { from, target, last }`, where `last` is the level it last sent, before the safety layer.

## Diagnostics

`Diagnostics` is a shared on/off switch and a bounded log (2000 entries by default) for live troubleshooting views. It's for what never reaches the signal plane: `OscInput` logs every message, including ones nothing uses and argument types the plane doesn't keep (Int, String).

- It's off until a view turns it on (`set_enabled`). `log(module, label, value)` checks the switch before calling `value`, so formatting costs nothing while off.
- `entries_since(seq)` returns the entries newer than `seq` and the seq to pass next time. The oldest entry is dropped when the log is full. Turning it off or on empties it.
- Any input can log through its `InputCtx`, so a future input gets a log in the view without any view changes.

In the app, one `Diagnostics` lives in the global state and is shared with every listening session. Each session's diagnostics task (`runtime/diagnostics.rs`) sends the view a batch 10 times a second while it's on: the signal plane's changes, every toy's graph snapshot (copied by the graph runner every 10th tick), and new log entries.

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
- While diagnostics are on, every message (avatar changes included) is logged with its address and arguments, formatted like `osc-tool listen` (`0.5 (float)`, `3 (int)`).
- On `/avatar/change`, it clears every value it has written (`InputCtx::clear_signals`) before publishing the event, so the clear stays in message order.
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
| `RateMode` | follows stroke speed: distance moved per second, averaged over about 0.15 s, times `rate_sensitivity`, with 2 full ranges per second (about one in-and-out stroke) as full power. Movement within 0.02 of the last counted position is ignored as jitter. When the speed drops, the level eases off exponentially, falling to about a third in `0.15 / rate_wind_down` s. Reaching 0 is just a position (no reset), and the first value after creation or reset only sets the starting position | ignored |
| `ConstantMode` | `constant_level` if >= 0.5, else 0 | `constant_level` if true, else 0 |

Raw and Constant behave exactly as the app's original modes. Smooth and Rate were redefined in time when the graph moved to a fixed tick: they used to work per OSC message (average the last N messages; decay only when a message arrived), which made them depend on VRChat's send rate and let Rate stick at its last level. The tests pin each mode's behaviour.

### `OutputLevels` (`modules::levels`)

Idle Speed and Range, as a processor and as `apply(level)`:

1. exactly 0 becomes `idle`;
2. anything else is clamped to `min..=max` (a reversed range is tolerated instead of panicking).

Flip Input isn't applied here, so levels stay intensities through the safety layer. `levels::flip(level)` (`1 - level`, rounded to hundredths) is what outputs use to apply it.

### `ButtplugOutput`

An `Output` for one Buttplug device. Each port maps to an `ActuatorSpec`: an `Actuator` plus `flip`. Flipped actuators get `1 - level` (`device_level`), applied here after the safety layer. `set_actuators` replaces the mapping, e.g. when Flip Input changes.

It uses the Buttplug 12 client (`buttplug_client` 11, message spec v4), where commands go to a numbered device **feature**:

| `Actuator` | Sends | Notes |
|---|---|---|
| `Level { feature, output }` | the output (vibrate, rotate, oscillate, constrict, position) at the level | Rotation is always clockwise. The level is a percentage of the output's steps |
| `Position { feature }` | position with duration | The command's `duration_ms` is the movement time |

`send_to_actuator(device, actuator, level, duration_ms)` sends one level outside the output, e.g. for the app's Test button.

`LegacyLayout::of(device)` numbers a device's actuators the way message spec v3 did (scalars, rotators and linears each from 0), which is how the app's toy configs number features; see [toy-command-path.md](toy-command-path.md#feature-type--buttplug-command). `scalar(i)`, `rotator(i)` and `linear(i)` give the `Actuator` for each.

`apply` sends every command, even if one fails, and returns the first error. A command for a port the output doesn't have is an `Error::UnknownPort`.

With `with_events(module, events)`, it publishes `EventKind::OutputError` on its `error` port when sending **starts** failing (value: the error as text) and when it **works again** (value: `None`). It doesn't fire on every failed send, so a disconnected toy at 20 Hz doesn't flood the event plane.

## Migration status

| Step | State |
|---|---|
| Workspace split, core types and traits | Done |
| Toy commands through `ButtplugOutput` with one slot per motor | Done ([toy-command-path.md](toy-command-path.md)) |
| OSC listener as an `Input` module, avatar change as an event | Done ([toy-command-path.md](toy-command-path.md#routing)) |
| Smooth, Rate, Constant and level tweaks as `Processor` modules | Done |
| Input processors (SPS/TPS) as graph nodes | Done: they watch their parameters on the signal plane, and the per-toy OSC routing is gone ([toy-command-path.md](toy-command-path.md#input-processors)). SPS is rewritten for both the legacy and SPS3 formats, and TPS follows the deepest orifice or penetrator |
| Each toy's graph generated from its existing config, ticked at a fixed rate | Done ([toy-command-path.md](toy-command-path.md#the-graph)) |
| Output events from `ButtplugOutput` | Error and recovery done. Activity events ("motor started/stopped") wait for a design that handles Flip Input, where level 0 isn't "off" |
| Processor events published from the graph | Done |

## Tests

```
cd src-tauri
cargo test -p vibecheck-core
```

Covers address formatting, latest-value-wins on the signal plane, event order and causality, clamping and the depth limit, tracing event chains through the remembered history, refusing a module that reacts to the result of its own action, that motors on one toy don't overwrite each other in `CommandSlots`, `OscInput` over real UDP (tapping in order, bundles, avatar change events, signal writes, stopping when the tap closes, reporting a busy port), each processing mode's behaviour, `LegacyLayout` numbering actuators like spec v3, `OutputLevels`, `ButtplugOutput`'s error events firing only on change, and the graph (publishing processor events from their node, dependency order, holding values until inputs exist, outputs keeping their last value, sinks only firing on change, durations and the safety layer, rejecting bad wiring, and watching nodes seeing ports that were never wired), `changed_since` returning each new value once, in write order, `write_if_absent` never overwriting, a late reset replaying what's still on the plane, and `OscInput` clearing the old avatar's values. Diagnostics: `changed_since_all` covering every module with write counts and counting clears, graph snapshots before and after input, the log only formatting while on, returning only newer entries and dropping the oldest when full, and `OscInput` logging every message, including an Int the plane never sees.

The app's tests (`cargo test -p vibecheck`) cover `InputProcessorNode` (reading only its prefixes, input filter tags), the graph runner task, and the generated toy graphs end to end: nothing before the first input, modes loaded from config, one parameter driving several motors, Range and Idle after the mode, the first matching parameter winning, disabled features, Linear speed, the f32 rounding, Smooth following over its window, Rate winding down after the input stops, and input processor chains next to parameter chains.
