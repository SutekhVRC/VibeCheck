# Emit plane: backend → frontend (events)

Events go out through `src-tauri/src/vcore/ipc/emit_plane.rs`, except
diagnostics batches, which `toy_handling/runtime/diagnostics.rs` emits
itself. The frontend event names live in `LISTEN` in `src/data/constants.ts`.

The dotted arrows on the right show the frontend calling back into the
[command plane](command-plane.md) in response to an event.

```mermaid
flowchart LR
  subgraph SRC["Backend event sources"]
    BCE["Buttplug client events<br/>(toy_handling/runtime/client_event_handler.rs)"]
    OSCL["Disabled OSC listener<br/>(osc/logic.rs)"]
    BAT["Battery refresh<br/>(osc/logic.rs)"]
    API["OSC API<br/>(osc_api/mod.rs)"]
    ALT["alter_toy<br/>(call_plane.rs + offline path in frontend_native.rs)"]
    ERR["Notice handler<br/>(error_signal_handler/reporter.rs)"]
    SCN["Scan timer<br/>(vcore/scan.rs)"]
    TRAY["Tray menu<br/>(tray.rs)"]
    PAN["Panic stop<br/>(vcore/panic.rs: button, hotkey,<br/>tray, OSC API)"]
    CAT["Avatar catalog<br/>(osc/avatar_catalog.rs, via vcore/state.rs)"]
    DIA["Diagnostics task<br/>(toy_handling/runtime/diagnostics.rs)"]
  end

  subgraph EP["Emit plane (vcore/ipc/emit_plane.rs)"]
    ETE["emit_toy_event<br/>→ fe_toy_event"]
    ECE["emit_core_event<br/>→ fe_core_event"]
    EER["emit_error<br/>→ fe_error"]
    EAV["emit_avatar_event<br/>→ fe_avatar_event"]
  end
  EDI["fe_diagnostics<br/>(emitted by the diagnostics task)"]

  subgraph FE["Frontend listeners"]
    UT["useToys.tsx"]
    CE["CoreEvents.tsx"]
    TOAST["Toast notification<br/>(error / warning / info)"]
    AC["AvatarCatalog.tsx"]
    DG["useDiagnostics.ts"]
  end

  CMDS["Tauri commands<br/>(command plane)"]

  BCE -- "Add (device connected)" --> ETE
  BCE -- "Remove (device disconnected)" --> ETE
  BCE -- "Scan(Start)<br/>scan-on-disconnect" --> ECE
  BAT -- "Update (battery level)" --> ETE
  ALT -- "Update (settings changed)" --> ETE
  OSCL -- "State(EnableAndScan)<br/>vibecheck/api/state = true" --> ECE
  API -- "State(Disable)<br/>vibecheck/api/state = false" --> ECE
  SCN -- "Scan(Stop)<br/>scan length ran out" --> ECE
  TRAY -- "State(Enable / Disable /<br/>Scan / EnableAndScan)" --> ECE
  PAN -- "Safety(Panicked / Resumed)" --> ECE
  ERR -- "Error / Warning / Info<br/>(repeats skipped)" --> EER
  CAT -- "the whole catalog,<br/>on every change" --> EAV
  DIA -- "batches while<br/>diagnostics are on" --> EDI

  ETE --> UT
  ECE --> CE
  EER --> CE
  CE --> TOAST
  EAV --> AC
  EDI --> DG

  UT -. "on Remove: sync_offline_toys" .-> CMDS
  CE -. "EnableAndScan: enable + start_bt_scan<br/>Enable: enable · Scan: start_bt_scan<br/>Disable: stop_bt_scan + disable" .-> CMDS
```

## Event payloads

| Event | Rust type | Variants or contents |
|---|---|---|
| `fe_toy_event` | `FeToyEvent` | `Add(FeVCToy)`, `Remove(u32)`, `Update(FeVCToy)` |
| `fe_core_event` | `FeCoreEvent` | `Scan(Start \| Stop)`, `State(EnableAndScan \| Disable \| Enable \| Scan)`, `Safety(Panicked \| Resumed)` |
| `fe_error` | `FrontendError` | `Error(String)`, `Warning(String)`, `Info(String)` |
| `fe_avatar_event` | `FeAvatarCatalog` | `status`, `parameters`, `zones` (the avatar's SPS zones), `tps` |
| `fe_diagnostics` | `FeDiagnostics` | `reset`, changed `signals`, every toy's graph values (`toys`), new `log` entries |

- `fe_error` carries problems found by background work; see [Errors](../runtime/errors.md).
- `fe_avatar_event` is sent whenever the catalog changes; `get_avatar_catalog` returns the same thing on demand. See [Avatar catalog](../runtime/toy-command-path.md#avatar-catalog).
- `fe_diagnostics` is only sent while the diagnostics view has turned diagnostics on (`set_diagnostics_enabled`).

Desktop notifications (a toy connecting, or an app's own through
`SessionContext::notify`) aren't events: they go to the system directly,
and only while Settings > Desktop notifications is on.

See also: [command plane](command-plane.md), [VRChat enable round trip](enable-round-trip.md).
