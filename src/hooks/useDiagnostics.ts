import type { FeDiagLogEntry } from "@bindings/FeDiagLogEntry";
import type { FeDiagnostics } from "@bindings/FeDiagnostics";
import type { FeDiagToy } from "@bindings/FeDiagToy";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { INVOKE, LISTEN } from "../data/constants";

/** Most log entries kept (the backend keeps its own, larger, log) */
const LOG_LIMIT = 1000;

/** How much each batch moves the shown update rate (smooths it out) */
const RATE_SMOOTHING = 0.3;

/** A value an input wrote, as shown in the diagnostics view */
export type DiagValue = {
  module: string;
  port: string;
  value: number;
  isBool: boolean;
  writes: number;
  /** Read from VRChat's OSCQuery because VRChat hasn't sent it over OSC */
  fromOscQuery: boolean;
  /** When it last changed (Date.now()) */
  changedAt: number;
  /** Updates per second, smoothed */
  rate: number;
};

export type DiagView = {
  values: DiagValue[];
  toys: FeDiagToy[];
  log: FeDiagLogEntry[];
};

const EMPTY: DiagView = { values: [], toys: [], log: [] };

type Store = {
  values: Map<string, DiagValue>;
  toys: FeDiagToy[];
  log: FeDiagLogEntry[];
  lastBatchAt: number;
};

function emptyStore(): Store {
  return { values: new Map(), toys: [], log: [], lastBatchAt: Date.now() };
}

/**
 * Turns the backend's diagnostics on while mounted and keeps the latest
 * values, toy graph values and log. While paused, batches are still applied
 * but the view isn't updated, so resuming shows the current state.
 */
export function useDiagnostics(paused: boolean) {
  const store = useRef<Store>(emptyStore());
  const pausedRef = useRef(paused);
  const [view, setView] = useState<DiagView>(EMPTY);

  function publish() {
    const s = store.current;
    setView({ values: [...s.values.values()], toys: s.toys, log: s.log });
  }

  function apply(batch: FeDiagnostics) {
    const now = Date.now();
    const s = store.current;
    if (batch.reset) {
      s.values = new Map();
    }
    const dt = Math.max((now - s.lastBatchAt) / 1000, 0.01);
    s.lastBatchAt = now;

    // Every value's rate moves towards this batch's: 0 for ones that
    // didn't change
    const changed = new Map(
      batch.signals.map((v) => [`${v.module}\u0000${v.port}`, v]),
    );
    for (const [key, value] of s.values) {
      if (!changed.has(key)) {
        value.rate *= 1 - RATE_SMOOTHING;
        if (value.rate < 0.05) value.rate = 0;
        s.values.set(key, { ...value });
      }
    }
    for (const [key, v] of changed) {
      const old = s.values.get(key);
      const instant = old ? Math.max(v.writes - old.writes, 0) / dt : 0;
      s.values.set(key, {
        module: v.module,
        port: v.port,
        value: v.value,
        isBool: v.is_bool,
        writes: v.writes,
        fromOscQuery: v.from_oscquery,
        changedAt: now,
        rate: old
          ? old.rate * (1 - RATE_SMOOTHING) + instant * RATE_SMOOTHING
          : 0,
      });
    }

    s.toys = batch.toys;
    if (batch.log.length > 0) {
      s.log = [...s.log, ...batch.log].slice(-LOG_LIMIT);
    }
    if (!pausedRef.current) publish();
  }

  useEffect(() => {
    pausedRef.current = paused;
    if (!paused) publish();
  }, [paused]);

  useEffect(() => {
    const unlistenPromise = listen<FeDiagnostics>(LISTEN.DIAGNOSTICS, (e) =>
      apply(e.payload),
    );
    invoke(INVOKE.SET_DIAGNOSTICS_ENABLED, { enabled: true }).catch(() => {});
    return () => {
      invoke(INVOKE.SET_DIAGNOSTICS_ENABLED, { enabled: false }).catch(
        () => {},
      );
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  /** Forget everything shown; new values and messages keep arriving */
  function clear() {
    store.current = emptyStore();
    publish();
  }

  return { view, clear };
}
