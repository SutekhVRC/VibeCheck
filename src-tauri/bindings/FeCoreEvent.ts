// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { FeScanEvent } from "./FeScanEvent";
import type { FeStateEvent } from "./FeStateEvent";

export type FeCoreEvent = { kind: "Scan", data: FeScanEvent } | { kind: "State", data: FeStateEvent };