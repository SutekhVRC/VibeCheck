# Emit plane: backend → frontend (events)

All events go out through `src-tauri/src/vcore/ipc/emit_plane.rs`. The frontend
event names live in `LISTEN` in `src/data/constants.ts`.

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
    ERR["Error signal handler<br/>(error_signal_handler/state_comm.rs)"]
  end

  subgraph EP["Emit plane (vcore/ipc/emit_plane.rs)"]
    ETE["emit_toy_event<br/>→ fe_toy_event"]
    ECE["emit_core_event<br/>→ fe_core_event"]
    EER["emit_error<br/>→ fe_error"]
  end

  subgraph FE["Frontend listeners"]
    UT["useToys.tsx"]
    CE["CoreEvents.tsx"]
    TOAST["Toast notification<br/>(error / warning / info)"]
  end

  CMDS["Tauri commands<br/>(command plane)"]

  BCE -- "Add (device connected)" --> ETE
  BCE -- "Remove (device disconnected)" --> ETE
  BCE -- "Scan(Start)<br/>scan-on-disconnect" --> ECE
  BAT -- "Update (battery level)" --> ETE
  ALT -- "Update (settings changed)" --> ETE
  OSCL -- "State(EnableAndScan)<br/>vibecheck/api/state = true" --> ECE
  API -- "State(Disable)<br/>vibecheck/api/state = false" --> ECE
  ERR -- "Error(message)" --> EER

  ETE --> UT
  ECE --> CE
  EER --> CE
  CE --> TOAST

  UT -. "on Remove: sync_offline_toys" .-> CMDS
  CE -. "EnableAndScan: enable + start_bt_scan<br/>Disable: stop_bt_scan + disable" .-> CMDS
```

## Event payloads

| Event | Rust type | Variants |
|---|---|---|
| `fe_toy_event` | `FeToyEvent` | `Add(FeVCToy)`, `Remove(u32)`, `Update(FeVCToy)` |
| `fe_core_event` | `FeCoreEvent` | `Scan(Start)`, `State(EnableAndScan \| Disable)` |
| `fe_error` | `FrontendError` | `Error(String)`, `Warning(String)`, `Info(String)` |

See also: [command plane](command-plane.md), [VRChat enable round trip](enable-round-trip.md).
