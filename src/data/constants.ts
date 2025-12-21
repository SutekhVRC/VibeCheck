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

export const TOOLTIP = {
  VrChatGroup: {
    text: "Vibecheck VRChat Group",
    link: "VRChatGroup",
  },
  Discord: {
    text: "Vibecheck Discord",
    link: "Discord",
  },
  Github: {
    text: "Vibecheck Github",
    link: "Github",
  },
  Enabled: {
    text: "Enable/Disable this feature.",
    link: "",
  },
  OSC_Data: {
    text: "If vibecheck should send OSC data to VRChat",
    link: "ToyOptions",
  },
  Anatomy: {
    text: "Anatomy types can be used as a category filter to disable/enable multiple toys at the same time from VRChat using the VibeCheck OSC API",
    link: "ToyOptions",
  },
  InputProcessor: {
    text: "Choose the way VibeCheck processes input. Example: If your avatar is using SPS and you want VibeCheck to interact with it, switch to SPS.",
    link: "FeatureOptions",
  },
  InputFilter: {
    text: "Filter what parameters VibeCheck listens to for this feature.",
    link: "FeatureOptions",
  },
  LinearSpeed: {
    text: "Linear positional duration speed in milliseconds. Speed is determined by the toy itself, this is only requested speed.",
    link: "FeatureOptions",
  },
  FlipInput: {
    text: "Some toys use a flipped float input. Enable this if your toy seems to do the opposite motor level you were expecting.",
    link: "FeatureOptions",
  },
  Idle: {
    text: "Set the idle motor speed for this feature. Idle activates when there is no input. Your set idle speed won't activate until you send at least one float value in the valid min/max range you have set.",
    link: "FeatureOptions",
  },
  Range: {
    text: "The minimum/maximum motor speed that will be sent to the feature's motor.",
    link: "FeatureOptions",
  },
  Smooth: {
    text: "This smooths the float input by queueing the amount set with the slider, then transforming them into one value to send instead. If you aren't sending a lot of floats rapidly over OSC you probably want this disabled completely.",
    link: "",
  },
  Rate: {
    text: "This uses rate mode on the float input.",
    link: "",
  },
  Constant: {
    text: "The intensity your toy will activate when you have constant mode enabled.",
    link: "",
  },
  Simulate: {
    text: "Test feature power level.",
    link: "FeatureOptions",
  },
  OSC_Bind: {
    text: "OSC Receive Port (Default: 127.0.0.1:9001)",
    link: "",
  },
  OSC_Remote: {
    text: "OSC Receive Port (Default: 127.0.0.1:9001)",
    link: "",
  },
  OSC_Send: {
    text: "OSC Send Port (Default: 127.0.0.1:9000)",
    link: "",
  },
  ScanOnDisconnect: {
    text: "Automatically start scanning when a toy disconnects.",
    link: "",
  },
  MinimizeOnExit: {
    text: "Minimize VibeCheck instead of exiting.",
    link: "",
  },
  DesktopNotifications: {
    text: "Notifications for toy connect and disconnect.",
    link: "",
  },
  AdvancedToy: {
    text: "Show advanced toy options like osc data and anatomy",
    link: "",
  },
  AdvancedFeature: {
    text: "Show advanced options for features [vibrator, constrict, oscillate, etc], will show options like idle speed, flip input, simulate",
    link: "",
  },
  ToyUpdateRate: {
    text: "Set how frequently to send Bluetooth updates to this toy (in Hz). Higher values will feel more responsive but use more bandwidth and battery. However higher values may also make your toy more unstable.",
    link: "ToyOptions",
  },
} as const;
