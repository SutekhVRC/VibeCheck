import type { FeAvatarCatalog } from "@bindings/FeAvatarCatalog";
import type { FeFeatureCheck } from "@bindings/FeFeatureCheck";
import type { FeVCToyFeature } from "@bindings/FeVCToyFeature";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  createContext,
  ReactNode,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";
import { INVOKE, LISTEN } from "../data/constants";

/*
 * The current avatar's parameters, read by the backend from VRChat's
 * OSCQuery (src-tauri/src/osc/avatar_catalog.rs). One listener for the whole
 * app; the backend sends the catalog again whenever it changes.
 */

type AvatarCatalogContext = {
  catalog: FeAvatarCatalog;
  /** Reads the avatar from VRChat again now */
  refresh: () => void;
};

const LOOKING: FeAvatarCatalog = { status: "Looking", parameters: [] };

const Context = createContext<AvatarCatalogContext>({
  catalog: LOOKING,
  refresh: () => null,
});

export function AvatarCatalogProvider({ children }: { children: ReactNode }) {
  const [catalog, setCatalog] = useState<FeAvatarCatalog>(LOOKING);

  useEffect(() => {
    let cancelled = false;
    // Listen first, so a change between the two calls isn't missed
    const unlisten = listen<FeAvatarCatalog>(LISTEN.AVATAR, (event) =>
      setCatalog(event.payload),
    );
    invoke<FeAvatarCatalog>(INVOKE.GET_AVATAR_CATALOG)
      .then((current) => {
        if (!cancelled) setCatalog(current);
      })
      .catch(() => null); // Stays "Looking" until the backend says otherwise
    return () => {
      cancelled = true;
      unlisten.then((stop) => stop());
    };
  }, []);

  const refresh = useCallback(() => {
    invoke(INVOKE.REFRESH_AVATAR_CATALOG).catch(() => null);
  }, []);

  return (
    <Context.Provider value={{ catalog, refresh }}>{children}</Context.Provider>
  );
}

export function useAvatarCatalog() {
  return useContext(Context);
}

/** How long to wait after the last change before checking a feature */
const CHECK_DELAY_MS = 250;

/**
 * What `feature`'s settings miss on the current avatar, rechecked when the
 * feature or the avatar changes. Null while the avatar isn't known, so
 * nothing is flagged without a reason.
 */
export function useFeatureCheck(feature: FeVCToyFeature) {
  const { catalog } = useAvatarCatalog();
  const [check, setCheck] = useState<FeFeatureCheck | null>(null);

  // Only what the check looks at, so dragging a slider doesn't recheck
  const latest = useRef(feature);
  latest.current = feature;
  const checked = JSON.stringify([
    feature.osc_parameters.map((p) => p.parameter),
    feature.penetration_system.pen_system_type,
    feature.penetration_system.pen_system_input_filter,
  ]);

  useEffect(() => {
    if (catalog.status !== "Ready") {
      setCheck(null);
      return;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      invoke<FeFeatureCheck | null>(INVOKE.CHECK_FEATURE, {
        feature: latest.current,
      })
        .then((result) => {
          if (!cancelled) setCheck(result);
        })
        .catch(() => {
          if (!cancelled) setCheck(null);
        });
    }, CHECK_DELAY_MS);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [checked, catalog]);

  return check;
}
