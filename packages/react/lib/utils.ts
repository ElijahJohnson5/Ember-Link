import { useRef } from 'react';

const emptyArray: unknown[] = [];

export const getEmptyArray = <T>() => {
  return emptyArray as Array<T>;
};

/**
 * Stabilize an object reference across renders when its top-level fields
 * are shallow-equal to the previous render. Useful for using an options
 * bag in a `useMemo`/`useEffect` dependency array without recreating
 * downstream resources every time the parent re-renders with a fresh
 * (but equivalent) literal.
 */
export function useShallowMemo<T>(value: T): T {
  const ref = useRef(value);
  if (!shallowEqual(value, ref.current)) {
    ref.current = value;
  }
  return ref.current;
}

function shallowEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (
    a === null ||
    b === null ||
    typeof a !== 'object' ||
    typeof b !== 'object'
  ) {
    return false;
  }
  const aRec = a as Record<string, unknown>;
  const bRec = b as Record<string, unknown>;
  const aKeys = Object.keys(aRec);
  const bKeys = Object.keys(bRec);
  if (aKeys.length !== bKeys.length) return false;
  for (const k of aKeys) {
    if (aRec[k] !== bRec[k]) return false;
  }
  return true;
}
