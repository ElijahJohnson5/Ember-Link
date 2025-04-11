import {
  MessageEvents,
  NetworkSender,
  IStorage,
  ArrayStorage,
  MapStorage,
  StorageEvent,
  StorageEvents
} from '@ember-link/core';

class MockArrayStorage<T> implements ArrayStorage<T> {
  inner: Array<T> = [];
  subscribers: Array<(event: StorageEvent) => void> = [];

  get length() {
    return this.inner.length;
  }

  insertAt(index: number, value: T) {
    this.inner.splice(index, 0, value);
    this.notifySubscribers({} as StorageEvent);
  }

  push(value: T) {
    this.inner.push(value);
    this.notifySubscribers({} as StorageEvent);
  }

  toArray() {
    return this.inner;
  }

  delete(index: number, length: number) {
    this.inner.splice(index, length);
    this.notifySubscribers({} as StorageEvent);
  }

  forEach(callback: (value: T, index: number, array: ArrayStorage<T>) => void) {
    this.inner.forEach((value, index) => {
      callback(value, index, this);
    });
  }

  subscribe(callback: (event: StorageEvent) => void) {
    const index = this.subscribers.push(callback);

    return () => {
      this.subscribers.splice(index - 1, 1);
    };
  }

  [Symbol.iterator](): IterableIterator<T> {
    return this.inner[Symbol.iterator]();
  }

  private notifySubscribers(event: StorageEvent) {
    this.subscribers.forEach((t) => {
      t(event);
    });
  }
}

class MockMapStorage<K extends string, V> implements MapStorage<K, V> {
  inner: Map<K, V> = new Map();
  subscribers: Array<(event: StorageEvent) => void> = [];

  get size() {
    return this.inner.size;
  }

  get(key: K) {
    return this.inner.get(key);
  }
  set(key: K, value: V) {
    this.inner.set(key, value);

    this.notifySubscribers({} as StorageEvent);

    return value;
  }
  delete(key: K) {
    const returnValue = this.inner.delete(key);
    this.notifySubscribers({} as StorageEvent);

    return returnValue;
  }
  has(key: K) {
    return this.inner.has(key);
  }
  clear() {
    this.inner.clear();
    this.notifySubscribers({} as StorageEvent);
  }
  entries() {
    return this.inner.entries();
  }
  [Symbol.iterator](): IterableIterator<[K, V]> {
    return this.inner[Symbol.iterator]();
  }

  subscribe(callback: (event: StorageEvent) => void) {
    const index = this.subscribers.push(callback);

    return () => {
      this.subscribers.splice(index - 1, 1);
    };
  }

  private notifySubscribers(event: StorageEvent) {
    this.subscribers.forEach((t) => {
      t(event);
    });
  }
}

export const mockStorageProvider = {
  sync: function (
    events: {
      subscribe<Key extends 'message'>(event: Key, callback: MessageEvents[Key]): () => void;
      subscribeOnce<Key extends 'message'>(event: Key, callback: MessageEvents[Key]): () => void;
      waitUntil<Key extends 'message'>(event: Key): Promise<Parameters<MessageEvents[Key]>>;
    },
    sender: NetworkSender
  ): Promise<boolean> {
    throw new Error('Function not implemented.');
  },
  getStorage: function (): IStorage {
    return {
      root: this,
      applyUpdate: function (event: Uint8Array): void {
        throw new Error('Function not implemented.');
      },
      getArray: function <T>(name: string): ArrayStorage<T> {
        return new MockArrayStorage<T>();
      },
      getMap: function <K extends string, V>(name: string): MapStorage<K, V> {
        return new MockMapStorage<K, V>();
      },
      subscribe: function <T>(
        object: ArrayStorage<T> | MapStorage<string, T>,
        callback: (event: StorageEvent) => void
      ): () => void {
        return object.subscribe(callback);
      },
      events: {
        subscribe: function <Key extends 'update'>(
          event: Key,
          callback: StorageEvents[Key]
        ): () => void {
          return () => {};
        },
        subscribeOnce: function <Key extends 'update'>(
          event: Key,
          callback: StorageEvents[Key]
        ): () => void {
          throw new Error('Function not implemented.');
        },
        waitUntil: function <Key extends 'update'>(
          event: Key
        ): Promise<Parameters<StorageEvents[Key]>> {
          throw new Error('Function not implemented.');
        }
      }
    };
  },
  type: 'yjs' as const
};
