# osc-tool

A debug tool for VibeCheck's OSC input. It can:

- **listen**: print the OSC arriving on a port, to check VRChat is actually sending to VibeCheck;
- **send**: send a message to VibeCheck as if VRChat sent it, to test toy parameters and the OSC API without VRChat;
- **sweep**: ramp a float parameter from one value to another, to test Range, Smooth and Rate mode.

Run it from `src-tauri/`:

```
cargo run -p osc-tool -- listen
cargo run -p osc-tool -- send Boop 0.5
cargo run -p osc-tool -- sweep Boop 0 1 --back
cargo run -p osc-tool -- COMMAND --help
```

## listen

Prints every message arriving on a port.

**Close VibeCheck first**, including the tray icon. VibeCheck holds its OSC port while it's open, even when switched off, so the two can't listen at the same time. If the port is busy, `listen` says so.

```
osc-tool listen                              # 127.0.0.1:9001, VRChat's default
osc-tool listen 9001 --filter Boop           # only addresses containing "Boop"
osc-tool listen 0.0.0.0:9001 --stats         # one summary line per second
```

| Option | What it does |
|---|---|
| `ADDRESS` | `host:port`, or just a port (meaning `127.0.0.1:port`). Default `127.0.0.1:9001`, which matches VibeCheck's default OSC Bind |
| `--filter TEXT` | Only show addresses containing `TEXT`, ignoring case. Repeat to allow several |
| `--stats` | Print messages per second and how many different addresses were seen, instead of every message |

Running `osc-tool` with no command also listens, so `osc-tool 9001 --filter Boop` works too.

```
Listening for OSC on 127.0.0.1:9001. Press Ctrl+C to stop.
    0.522s  ---- avatar changed ----
    0.522s  /avatar/change  "avtr_..." (string)
    0.522s  /avatar/parameters/Boop  0.85 (float)
    0.522s  /avatar/parameters/Boop  true (bool)
```

- Times are seconds since the tool started.
- Bundles are unpacked, so every message is shown on its own line.
- Packets that aren't valid OSC are reported instead of being silently dropped.
- If nothing arrives within 5 seconds, it prints a checklist of common causes.

| You see | It means |
|---|---|
| Your avatar's parameters scrolling past | VRChat is sending to this port. If VibeCheck still doesn't react, check the parameter names in the toy's settings match exactly |
| Nothing, then the checklist | VRChat isn't sending here: OSC is off, it's sending to a different port, or another app is in the way |
| "already in use" | Something else holds the port, most often VibeCheck itself |
| A parameter is missing | It may not be in the avatar's OSC config yet. Use **Settings > Troubleshooting > Reset VRChat OSC configs** in VibeCheck, then reload the avatar |

## send

Sends one message to VibeCheck. It sends from a temporary port, so **VibeCheck can stay open**; that's the point.

```
osc-tool send Boop true                      # a Constant-mode parameter
osc-tool send Boop 0.5                       # a Raw-mode parameter at 50%
osc-tool send Boop 0                         # back to idle
osc-tool send Boop 0.8 --repeat 5 --every 200
```

**Parameter names.** Anything not starting with `/` gets `/avatar/parameters/` in front, so `Boop` is sent as `/avatar/parameters/Boop`, the same as VRChat would send it. Use a full address for anything else, like `/avatar/change`.

**Values.**

| You type | Sent as |
|---|---|
| `true`, `false` | bool |
| `0.5`, `1`, `-0.25` | float. Numbers are always floats, since VRChat sends float parameters and VibeCheck only reads floats and bools |
| `f:0.5`, `i:3`, `b:true`, `s:text` | that type, explicitly |

Several values can follow the name; VibeCheck uses the last one.

| Option | What it does |
|---|---|
| `--to ADDRESS` | Where to send. Default `127.0.0.1:9001`, VibeCheck's default OSC Bind |
| `--repeat N` | Send N times (default 1) |
| `--every MS` | Milliseconds between repeats (default 1000) |

### Testing the OSC API

```
osc-tool send vibecheck/state true                       # turn VibeCheck on (works while it's off)
osc-tool send vibecheck/api/state false                  # turn VibeCheck off
osc-tool send vibecheck/api/anatomy/Vagina/enabled false # disable toys tagged Vagina
```

## sweep

Sends a float that moves steadily from `FROM` to `TO`.

```
osc-tool sweep Boop 0 1                              # 2 s ramp at 20 Hz
osc-tool sweep Boop 0 1 --seconds 0.3 --back         # a quick stroke, for Rate mode
osc-tool sweep Boop 1 0 --seconds 5 --rate 10        # slow fade out
```

| Option | What it does |
|---|---|
| `--seconds S` | How long the ramp takes (default 2) |
| `--rate HZ` | Messages per second (default 20) |
| `--back` | Ramp back to `FROM` afterwards |
| `--to ADDRESS` | Where to send (default `127.0.0.1:9001`) |

Values are rounded to hundredths, the same as VibeCheck does when it receives them, so what's printed is what VibeCheck sees. The ramp always ends exactly on `TO` (and on `FROM` with `--back`).

## When nothing happens

OSC over UDP has no delivery receipt, so "sent" doesn't mean anything received it. If VibeCheck doesn't react to `send` or `sweep`, check:

- VibeCheck is switched **on** (except `vibecheck/state true`, which is meant for when it's off);
- a feature on a connected toy uses the parameter, spelt exactly the same (names are case-sensitive);
- `--to` matches VibeCheck's **OSC Bind** in Settings.

To see what's actually going out, run `listen` on a spare port and `send --to` that port.
