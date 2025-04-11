import type { ArrayStorage, DefaultCustomMessageData, DefaultPresence, MapStorage, StorageEvent } from '@ember-link/core';
import { useChannel } from './channel-provider';
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from 'react';

export type ArrayStorageHookResult<T> = { current: Array<T> } & ArrayStorage<T>;

export type MapStorageHookResult<K extends string, V> = { current: Map<K, V> } & MapStorage<K, V>;

export const useArrayStorage = <T, P extends DefaultPresence, C extends DefaultCustomMessageData>(
  name: string
): ArrayStorageHookResult<T> => {
  const channel = useChannel<P, C>();

  if (!channel.hasStorage()) {
    throw new Error('A storage provider must be configured to use storage');
  }

  const storage = useMemo(() => channel.getStorage(), [channel]);

  const inner = useMemo(() => storage.getArray<T>(name), [name, storage]);

  const [cachedInnerArray, setCachedInnerArray] = useState<Array<T>>([])

  useEffect(() => {
    setCachedInnerArray([...inner]);
  }, [inner])

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
    () => []
  );

  const arrayStorage = useMemo(() => {
    return {
      get length() {
        return inner.length;
      },
      insertAt: function (index: number, value: T): void {
        inner.insertAt(index, value);
      },
      push: function (value: T): void {
        inner.push(value);
      },
      toArray: function (): T[] {
        return inner.toArray();
      },
      delete: function (index: number, length: number): void {
        inner.delete(index, length);
      },
      forEach: function (callback: (value: T, index: number, array: ArrayStorage<T>) => void): void {
        return inner.forEach(callback);
      },
      subscribe: function (callback: (event: StorageEvent) => void): () => void {
        return inner.subscribe(callback);
      },
      [Symbol.iterator]: function (): IterableIterator<T> {
        return inner[Symbol.iterator]();
      }
    } satisfies ArrayStorage<T>
  }, [inner])


  return { current: syncedArray, ...arrayStorage };
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
  }, [inner])

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
    () => new Map()
  );

  const mapStorage = useMemo(() => {
    return {
      get size() {
        return inner.size;
      },
      get: function (key: K): V | undefined {
        return inner.get(key);
      },
      set: function (key: K, value: V): V {
        return inner.set(key, value);
      },
      delete: function (key: K): void {
        return inner.delete(key);
      },
      has: function (key: K): boolean {
        return inner.has(key);
      },
      clear: function (): void {
        inner.clear();
      },
      entries: function (): IterableIterator<[K, V]> {
        return inner.entries();
      },
      subscribe: function (callback: (event: StorageEvent) => void): () => void {
        return inner.subscribe(callback);
      },
      [Symbol.iterator]: function (): IterableIterator<[K, V]> {
        return inner[Symbol.iterator]();
      }
    } satisfies MapStorage<K, V>
  }, [inner])


  return { current: syncedMap, ...mapStorage };
};
