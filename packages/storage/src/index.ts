import { Observable } from '@ember-link/event-emitter';
import { StorageSyncMessage, StorageType } from '@ember-link/protocol';

export type MessageEvents = {
  message: (message: StorageSyncMessage) => void;
};

export type NetworkSender = {
  message: (data: StorageSyncMessage) => void;
};

export interface IStorageProvider {
  sync: (events: Observable<MessageEvents>, sender: NetworkSender) => Promise<boolean>;
  getStorage: () => IStorage;
  type: StorageType;
}

export interface ArrayStorage<T> {
  readonly length: number;
  insertAt: (index: number, value: T) => void;
  push: (value: T) => void;
  toArray: () => Array<T>;
  delete: (index: number, length: number) => void;
  forEach: (callback: (value: T, index: number, array: ArrayStorage<T>) => void) => void;
  [Symbol.iterator](): IterableIterator<T>;

  subscribe: (callback: (event: StorageEvent) => void) => () => void;
}

export interface MapStorage<K extends string, V> {
  size: number;
  get: (key: K) => V | undefined;
  set: (key: K, value: V) => V;
  delete: (key: K) => void;
  has: (key: K) => boolean;
  clear: () => void;

  subscribe: (callback: (event: StorageEvent) => void) => () => void;
}

export interface StorageEvent {
  changes: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    keys: Map<string, { action: 'add' | 'update' | 'delete'; oldValue: any }>;
  };
}

export type StorageEvents = {
  update: (event: Uint8Array) => void;
};

export interface IStorage {
  root: unknown;
  applyUpdate(event: Uint8Array): void;

  getArray<T>(name: string): ArrayStorage<T>;
  getMap<K extends string, V>(name: string): MapStorage<K, V>;

  subscribe<T>(
    object: ArrayStorage<T> | MapStorage<string, T>,
    callback: (event: StorageEvent) => void
  ): () => void;

  events: Observable<StorageEvents>;
}
