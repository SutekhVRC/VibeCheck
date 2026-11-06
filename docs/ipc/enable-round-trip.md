# Example round trip: VRChat turns VibeCheck on

The two IPC planes form a loop. The backend never enables itself: it asks the
UI to do it through the [emit plane](emit-plane.md), and the UI calls back in
through the [command plane](command-plane.md).

```mermaid
sequenceDiagram
  participant VRC as VRChat
  participant DL as Disabled OSC listener
  participant CE as CoreEvents.tsx
  participant CP as Commands / call plane
  participant TM as Toy mgmt thread
  participant BP as Buttplug client
  participant UT as useToys.tsx

  VRC->>DL: OSC /avatar/parameters/vibecheck/state = true
  DL-->>CE: fe_core_event State(EnableAndScan)
  CE->>CP: invoke vibecheck_enable
  CP->>DL: stop disabled listener
  CP->>TM: TmSig::StartListening
  TM-->>CP: listening started
  CP-->>CE: Ok → isEnabled = true
  CE->>CP: invoke vibecheck_start_bt_scan
  CP->>BP: start_scanning
  BP-->>UT: device added → fe_toy_event Add(toy)
  Note over CE: UI stops the scan itself after 10s<br/>(invoke vibecheck_stop_bt_scan)
```
