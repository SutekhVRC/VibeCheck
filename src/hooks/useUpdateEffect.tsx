import { useEffect, useRef } from "react";

export function useUpdateEffect(
  effect: () => void,
  dependencyArray: unknown[]
) {
  const isMountedRef = useRef(false);

  useEffect(() => {
    if (isMountedRef.current) {
      effect();
    } else {
      isMountedRef.current = true;
    }
  }, dependencyArray);
}
