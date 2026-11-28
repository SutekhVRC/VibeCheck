# Contributing

Thanks for helping out! Bugs and ideas are welcome as [issues](https://github.com/SutekhVRC/VibeCheck/issues) or on [Discord](https://discord.gg/C8ACSGSyfq). For code, open a pull request against `main`.

## Building from source

VibeCheck is a [Tauri](https://tauri.app/) app with a Rust backend and a React frontend.

You'll need [Rust](https://www.rust-lang.org/tools/install), [Node.js](https://nodejs.org/) with [Yarn](https://yarnpkg.com/) (enable it with `corepack enable`) and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
yarn install       # install frontend dependencies
yarn tauri dev     # run the app in development mode
yarn tauri build   # build an installer
```

The Rust code is a Cargo workspace rooted at `src-tauri/`: the app, the [`vibecheck-core`](./docs/runtime/vibecheck-core.md) library that holds the module runtime, and `osc-tool`. To write an input, processor or output, see [Writing modules](./docs/runtime/writing-modules.md). [Toy command path](./docs/runtime/toy-command-path.md) follows an avatar parameter all the way to a toy's motor, and [Errors](./docs/runtime/errors.md) covers how problems reach the user. Run the tests from `src-tauri/` with `cargo test --workspace`.

## Testing OSC without VRChat

The workspace includes [`osc-tool`](./src-tauri/crates/osc-tool/README.md), a small command-line tool for checking VibeCheck's OSC input and reading an avatar's parameters from VRChat. Run it from `src-tauri/`:

```bash
cargo run -p osc-tool -- listen                       # is VRChat sending? (close VibeCheck first)
cargo run -p osc-tool -- send Boop 0.5                # send a parameter to VibeCheck as if VRChat did
cargo run -p osc-tool -- sweep Boop 0 1 --back        # ramp a parameter up and back down
cargo run -p osc-tool -- avatar --out avatar.json     # save the avatar's parameters from VRChat
```
