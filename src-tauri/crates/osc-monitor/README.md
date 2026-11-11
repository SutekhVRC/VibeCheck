# osc-monitor

A debug tool that prints the OSC messages arriving on a UDP port. Use it to check whether VRChat is actually sending OSC to the port VibeCheck listens on.

**Close VibeCheck first** (including the tray icon). VibeCheck holds its OSC port while it's open, even when switched off, so the two can't listen at the same time. If the port is busy, the monitor says so.

## Running

From `src-tauri/`:

```
cargo run -p osc-monitor                              # listen on 127.0.0.1:9001 (VRChat's default)
cargo run -p osc-monitor -- 9001 --filter Boop        # only addresses containing "Boop"
cargo run -p osc-monitor -- 0.0.0.0:9001 --stats      # one summary line per second
cargo run -p osc-monitor -- --help
```

| Option | What it does |
|---|---|
| `ADDRESS` | `host:port`, or just a port (meaning `127.0.0.1:port`). Default `127.0.0.1:9001`, which matches VibeCheck's default OSC Bind |
| `--filter TEXT` | Only show addresses containing `TEXT`, ignoring case. Repeat to allow several |
| `--stats` | Print messages per second and how many different addresses were seen, instead of every message |

## Output

```
Listening for OSC on 127.0.0.1:9001. Press Ctrl+C to stop.
    0.522s  ---- avatar changed ----
    0.522s  /avatar/change  "avtr_..." (string)
    0.522s  /avatar/parameters/Boop  0.85 (float)
    0.522s  /avatar/parameters/Boop  true (bool)
```

- Times are seconds since the monitor started.
- Bundles are unpacked, so every message is shown on its own line.
- Packets that aren't valid OSC are reported instead of being silently dropped.
- If nothing arrives within 5 seconds, it prints a checklist of common causes.

## Reading the results

| You see | It means |
|---|---|
| Your avatar's parameters scrolling past | VRChat is sending to this port. If VibeCheck still doesn't react, check the parameter names in the toy's settings match exactly |
| Nothing, then the checklist | VRChat isn't sending here: OSC is off, it's sending to a different port, or another app is in the way |
| "already in use" | Something else holds the port, most often VibeCheck itself |
| A parameter is missing | It may not be in the avatar's OSC config yet. Use **Settings > Troubleshooting > Reset VRChat OSC configs** in VibeCheck, then reload the avatar |
