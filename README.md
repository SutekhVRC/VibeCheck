<p align="center">
  <img src="./docs/VibeCheckBanner.png" alt="VibeCheck" width="100%" />
</p>

<p align="center">
  <b>Connect Bluetooth sex toys to your VRChat avatar using VRChat's OSC.</b>
</p>

<p align="center">
  <a href="https://github.com/SutekhVRC/VibeCheck/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/SutekhVRC/VibeCheck?style=for-the-badge&color=pink&label=release" /></a>
  <a href="https://github.com/SutekhVRC/VibeCheck/releases/latest"><img alt="Total downloads" src="https://img.shields.io/github/downloads/SutekhVRC/VibeCheck/total?style=for-the-badge&color=pink" /></a>
  <a href="./LICENSE"><img alt="License" src="https://img.shields.io/github/license/SutekhVRC/VibeCheck?style=for-the-badge&color=pink" /></a>
</p>

<p align="center">
  <a href="https://github.com/SutekhVRC/VibeCheck/releases/latest"><b>Download</b></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/SutekhVRC/VibeCheck/wiki"><b>Wiki</b></a>
  &nbsp;·&nbsp;
  <a href="https://discord.gg/C8ACSGSyfq"><b>Discord</b></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/SutekhVRC/VibeCheck/issues"><b>Report a bug</b></a>
</p>

---

## Features

- **Bluetooth toy support** through [Buttplug.io](https://buttplug.io/): connect any [supported toy](https://iostindex.com/?filter0ButtplugSupport=4) directly, no extra app needed. Toys show up as soon as they connect.
- **Driven by your avatar**: map VRChat avatar parameters to each toy motor over OSC.
- **SPS and TPS support**: SPS 3 and older SPS (including mixing the two), with depth, touch and frot, plus TPS. Input filters pick which sockets or plugs drive each toy.
- **Fine-tuning per motor**: output range, idle speed, flip input, plus Smooth, Rate and Constant processing modes. Rate follows how fast you stroke.
- **Safety controls**: a panic stop (sidebar button, **Ctrl + Alt + P**, the tray menu or an avatar parameter) and a master intensity slider for every toy.
- **In-game control** with the [VibeCheck OSC API](./docs/OSCAPI.md): turn VibeCheck on or off, panic stop, and enable or disable toys by anatomy from your avatar menu.
- **Battery levels sent back to VRChat** as OSC data for your avatar to use.
- **Windows and Linux**, with in-app updates on both (AppImage, `.deb` and `.rpm` on Linux). On Linux, VRChat runs through Proton as usual.
- **Quality of life**: themes, a tray menu with live status, a configurable scan length, scanning automatically when a toy disconnects, desktop notifications, and minimising instead of closing.

## Getting started

1. **Get a toy** that [Buttplug.io supports](https://iostindex.com/?filter0ButtplugSupport=4).
2. **Connect it**: follow [Connecting Toys](https://github.com/SutekhVRC/VibeCheck/wiki/Connecting-Toys).
3. **Install VibeCheck** from the [latest release](https://github.com/SutekhVRC/VibeCheck/releases/latest): the installer on Windows, or the AppImage, `.deb` or `.rpm` on Linux.
4. **Set up your avatar** with a known system or your own: [Avatar Setup](https://github.com/SutekhVRC/VibeCheck/wiki/Avatar-Setup).
5. **Set up the app**: [VibeCheck App Setup](https://github.com/SutekhVRC/VibeCheck/wiki/VibeCheck-App-Setup).

## Documentation

| Guide | What it covers |
|---|---|
| [App Setup](./docs/AppSetup.md) | Enabling VibeCheck and toy settings |
| [Avatar Setup](./docs/AvatarSetup.md) | OSC, premade prefabs, custom contact setups and TPS |
| [Bluetooth](./docs/Bluetooth.md) | Requirements, getting connected and common Bluetooth issues |
| [OSC API](./docs/OSCAPI.md) | Parameters to turn VibeCheck on and off, panic stop and toggle toys by anatomy from inside VRChat |
| [FAQ](./docs/Faq.md) | App, connection and avatar configuration issues |

More guides live on the [wiki](https://github.com/SutekhVRC/VibeCheck/wiki), including SPS setup in [Avatar Setup](https://github.com/SutekhVRC/VibeCheck/wiki/Avatar-Setup) and every option in Settings.

## Community

<p align="center">
  <a href="https://discord.gg/C8ACSGSyfq">
    <img alt="Join the VibeCheck Discord" src="https://img.shields.io/badge/Discord-Join%20the%20community-5865F2?style=for-the-badge&logo=discord&logoColor=white" height="40" />
  </a>
  &nbsp;
  <a href="https://vrc.group/VIBE.9503">
    <img alt="VibeCheck VRChat group" src="https://img.shields.io/badge/VRChat-VibeCheck%20group-1C1C1C?style=for-the-badge&logo=vrchat&logoColor=white" height="40" />
  </a>
</p>

<p align="center">
  Get help, share avatar setups, suggest features and hang out.<br />
  <b><a href="https://discord.gg/C8ACSGSyfq">discord.gg/C8ACSGSyfq</a></b>
</p>

## Bugs and feature requests

Found a bug or have an idea? Please [open an issue](https://github.com/SutekhVRC/VibeCheck/issues) or tell us on [Discord](https://discord.gg/C8ACSGSyfq). Reports help a lot!

## Building from source

VibeCheck is a [Tauri](https://tauri.app/) app with a Rust backend and a React frontend.

You'll need [Rust](https://www.rust-lang.org/tools/install), [Node.js](https://nodejs.org/) with [Yarn](https://yarnpkg.com/) (enable it with `corepack enable`) and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
yarn install       # install frontend dependencies
yarn tauri dev     # run the app in development mode
yarn tauri build   # build an installer
```

The Rust code is a Cargo workspace rooted at `src-tauri/`: the app, the [`vibecheck-core`](./docs/runtime/vibecheck-core.md) library that holds the module runtime, and `osc-tool`. [Toy command path](./docs/runtime/toy-command-path.md) follows an avatar parameter all the way to a toy's motor. Run the tests from `src-tauri/` with `cargo test --workspace`.

### Testing OSC without VRChat

The workspace includes [`osc-tool`](./src-tauri/crates/osc-tool/README.md), a small command-line tool for checking VibeCheck's OSC input. Run it from `src-tauri/`:

```bash
cargo run -p osc-tool -- listen                       # is VRChat sending? (close VibeCheck first)
cargo run -p osc-tool -- send Boop 0.5                # send a parameter to VibeCheck as if VRChat did
cargo run -p osc-tool -- sweep Boop 0 1 --back        # ramp a parameter up and back down
```

## License

VibeCheck is licensed under the [Apache License 2.0](./LICENSE).
