import type {
  ArrayStorage,
  DefaultCustomMessageData,
  DefaultPresence,
  MapStorage
} from '@ember-link/core';
import { useChannel } from './channel-provider';
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from 'react';

export type ArrayStorageHookResult<T> = { current: Array<T> } & ArrayStorage<T>;

export type MapStorageHookResult<K extends string, V> = { current: Map<K, V> } & MapStorage<K, V>;

const emptyArray: unknown[] = [];

const getEmptyArray = <T>() => {
  return emptyArray as Array<T>;
};

const emptyMap: Map<unknown, unknown> = new Map();

const getEmptyMap = <K extends string, V>() => {
  return emptyMap as Map<K, V>;
};

export const useArrayStorage = <T, P extends DefaultPresence, C extends DefaultCustomMessageData>(
  name: string
): ArrayStorageHookResult<T> => {
  const channel = useChannel<P, C>();

  if (!channel.hasStorage()) {
    throw new Error('A storage provider must be configured to use storage');
  }

  const storage = useMemo(() => channel.getStorage(), [channel]);

  const inner = useMemo(() => storage.getArray<T>(name), [name, storage]);

  const [cachedInnerArray, setCachedInnerArray] = useState<Array<T>>([]);

  useEffect(() => {
    setCachedInnerArray([...inner]);
  }, [inner]);

  const subscribeFunction = useCallback(
    (callback: (array: Array<T>) => void) => {
      return inner.subscribe(() => {
        setCachedInnerArray([...inner]);
        callback([...inner]);
      });
    },
    [inner]
  );

  const syncedArray = useSyncExternalStore(
    subscribeFunction,
    () => {
      return cachedInnerArray;
    },
    () => getEmptyArray()
  );

  return { current: syncedArray, ...inner };
};

export const useMapStorage = <
  K extends string,
  V,
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
>(
  name: string
): MapStorageHookResult<K, V> => {
  const channel = useChannel<P, C>();

  if (!channel.hasStorage()) {
    throw new Error('A storage provider must be configured to use storage');
  }

  const storage = useMemo(() => channel.getStorage(), [channel]);

  const inner = useMemo(() => storage.getMap<K, V>(name), [name, storage]);

  const [cachedInnerMap, setCachedInnerMap] = useState<Map<K, V>>(new Map());

  useEffect(() => {
    setCachedInnerMap(new Map([...inner]));
  }, [inner]);

  const subscribeFunction = useCallback(
    (callback: (map: Map<K, V>) => void) => {
      return inner.subscribe(() => {
        setCachedInnerMap(new Map([...inner]));
        callback(new Map([...inner]));
      });
    },
    [inner]
  );

  const syncedMap = useSyncExternalStore(
    subscribeFunction,
    () => {
      return cachedInnerMap;
    },
    () => getEmptyMap()
  );

  return { current: syncedMap, ...inner };
};
