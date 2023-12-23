import { useEffect, useRef } from "react";

export function useUpdateEffect<T>(effect: () => void, dependencyArray: T[]) {
  const isMountedRef = useRef(false);

  useEffect(() => {
    if (isMountedRef.current) {
      effect();
    } else {
      isMountedRef.current = true;
    }
  }, dependencyArray);
}
