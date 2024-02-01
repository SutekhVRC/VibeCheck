export type ObjectValues<T> = T[keyof T];

export const LISTEN = {
  CORE_EVENT: "fe_core_event",
  TOY_EVENT: "fe_toy_event",
} as const;

export const INVOKE = {
  ALTER_TOY: "alter_toy",
  CLEAR_OSC_CONFIG: "clear_osc_config",
  SIMULATE_TOY_FEATURE: "simulate_device_feature",
  VERSION: "vibecheck_version",
  START_SCAN: "vibecheck_start_bt_scan",
  STOP_SCAN: "vibecheck_stop_bt_scan",
  ENABLE: "vibecheck_enable",
  DISABLE: "vibecheck_disable",
  GET_CONFIG: "get_vibecheck_config",
  SET_CONFIG: "set_vibecheck_config",
  OPEN_BROWSER: "open_default_browser",
  OFFLINE_SYNC: "sync_offline_toys",
} as const;

export const OSC = {
  PARAM_PREFIX: "/avatar/parameters/",
  DATA_PREFIX: "vibecheck/osc_data/",
} as const;
