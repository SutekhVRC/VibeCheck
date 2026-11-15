# Command plane: frontend → backend (`invoke`)

Each command box shows the Tauri command name (`src-tauri/src/frontend/frontend_native.rs`)
and, below it, the call plane function it hands off to (`src-tauri/src/vcore/ipc/call_plane.rs`).

- **Dashed boxes** do their work inline in `frontend_native.rs` instead of going through the call plane.
- **Grey boxes** are registered but never called by the frontend.

```mermaid
flowchart LR
  subgraph FE["Frontend (src/)"]
    CE["CoreEvents.tsx"]
    UT["useToys.tsx"]
    US["useSimulate.tsx"]
    CF["Config.tsx"]
    UV["useVersion.tsx"]
    TL["Tooltip.tsx / ExternalLogo.tsx"]
  end

  subgraph CMD["Tauri commands (frontend_native.rs) → call plane (vcore/ipc/call_plane.rs)"]
    c_en["vibecheck_enable<br/>native_vibecheck_enable"]
    c_dis["vibecheck_disable<br/>native_vibecheck_disable"]
    c_ss["vibecheck_start_bt_scan<br/>native_vibecheck_start_bt_scan"]
    c_st["vibecheck_stop_bt_scan<br/>native_vibecheck_stop_bt_scan"]
    c_gc["get_vibecheck_config<br/>native_get_vibecheck_config"]
    c_sc["set_vibecheck_config<br/>native_set_vibecheck_config"]
    c_rc["reset_vibecheck_config<br/>native_reset_vibecheck_config"]
    c_at["alter_toy<br/>native_alter_toy (online toys)"]
    c_sim["simulate_device_feature<br/>native_simulate_device_feature"]
    c_clr["clear_osc_config<br/>native_clear_osc_config"]
    c_sync["sync_offline_toys<br/>(inline)"]
    c_rm["remove_offline_toy<br/>(inline)"]
    c_ver["vibecheck_version<br/>(inline)"]
    c_br["open_default_browser<br/>(inline)"]
    c_oq["get_oscquery_status<br/>native_get_oscquery_status"]
  end

  subgraph CORE["Backend core"]
    ST[("VCStateMutex<br/>running state, config, toy manager")]
    BP["Buttplug client"]
    TM["Toy management thread<br/>(tme_send_tx channel)"]
    DL["Disabled OSC listener"]
    FS[("Config files on disk")]
    DH["Toy device handles"]
    OQ["OSCQuery mode<br/>(osc/oscquery.rs)"]
  end

  CE --> c_en & c_dis & c_ss & c_st & c_gc
  CF --> c_sc & c_rc & c_clr & c_oq
  UT --> c_at & c_sync & c_rm
  US --> c_sim
  UV --> c_ver
  TL --> c_br

  c_en -- "TmSig::StartListening" --> TM
  c_en -- "stop" --> DL
  c_en -- "running = Running" --> ST
  c_dis -- "stop_scanning + stop_all_devices" --> BP
  c_dis -- "TmSig::TMHReset" --> TM
  c_dis -- "start" --> DL
  c_dis -- "running = Stopped" --> ST
  c_ss -- "start_scanning" --> BP
  c_st -- "stop_scanning" --> BP
  c_gc -- "read config" --> ST
  c_sc -- "write config" --> ST
  c_sc -- "save_config" --> FS
  c_at -- "ToyUpdate::AlterToy" --> TM
  c_at -- "save toy config<br/>(offline toys: inline path)" --> FS
  c_sim -- "feature level / stop" --> DH
  c_clr -- "delete VRChat OSC configs" --> FS
  c_sync -- "core_toy_manager.sync_frontend" --> ST
  c_rm -- "remove_offline_toy:<br/>refused while connected" --> ST
  c_rm -- "delete ToyConfigs files<br/>with that toy_name" --> FS
  c_rc -- "defaults, keeping oscquery_port" --> ST
  c_rc -- "save_config" --> FS
  c_rc -- "listen change: apply_oscquery,<br/>restart off listener" --> OQ
  c_sc -- "mode change: apply_oscquery,<br/>restart off listener" --> OQ
  c_oq -- "status: port, VRChat found" --> OQ

  classDef inline stroke-dasharray: 5 5
  classDef unused fill:#444,color:#aaa
  class c_sync,c_rm,c_ver,c_br inline
```

See also: [emit plane](emit-plane.md), [VRChat enable round trip](enable-round-trip.md).
