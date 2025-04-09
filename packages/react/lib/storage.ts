import type { DefaultCustomMessageData, DefaultPresence } from '@ember-link/core';
import { useChannel } from './channel-provider';
import { useCallback, useSyncExternalStore } from 'react';

export const useStorageArray = <T, P extends DefaultPresence, C extends DefaultCustomMessageData>(
  name: string
) => {
  const channel = useChannel<P, C>();

  if (!channel.hasStorage()) {
    throw new Error('A storage provider must be configured to use storage');
  }

  const storage = channel.getStorage();

  const array = storage.getArray<T>(name);

  const subscribeFunction = useCallback(
    (callback: (newArray: Array<T>) => void) => {
      return array.subscribe(() => {
        callback([...array]);
      });
    },
    [array]
  );

  const syncedArray = useSyncExternalStore(
    subscribeFunction,
    () => [...array],
    () => []
  );

  return syncedArray;
};

export const useStorageMap = <
  K extends string,
  V,
  P extends DefaultPresence,
  C extends DefaultCustomMessageData
>(
  name: string
) => {
  const channel = useChannel<P, C>();

  if (!channel.hasStorage()) {
    throw new Error('A storage provider must be configured to use storage');
  }

  const storage = channel.getStorage();

  const map = storage.getMap<K, V>(name);

  const subscribeFunction = useCallback(
    (callback: (newMap: Map<K, V>) => void) => {
      return map.subscribe(() => {
        callback(new Map(map.entries()));
      });
    },
    [map]
  );

  const syncedMap = useSyncExternalStore(
    subscribeFunction,
    () => new Map(map.entries()),
    () => new Map<K, V>()
  );

  return syncedMap;
};
