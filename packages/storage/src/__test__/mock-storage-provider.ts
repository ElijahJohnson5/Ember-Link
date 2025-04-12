import {
  MessageEvents,
  NetworkSender,
  IStorage,
  ArrayStorage,
  MapStorage,
  StorageEvent,
  StorageEvents
} from '../index';

const createMockArrayStorage = <T>() => {
  const inner: Array<T> = [];
  const subscribers: Array<(event: StorageEvent) => void> = [];

  function notifySubscribers(event: StorageEvent) {
    subscribers.forEach((t) => {
      t(event);
    });
  }

  return {
    get length() {
      return inner.length;
    },

    insertAt(index: number, value: T) {
      inner.splice(index, 0, value);
      notifySubscribers({} as StorageEvent);
    },

    push(value: T) {
      inner.push(value);
      notifySubscribers({} as StorageEvent);
    },

    toArray() {
      return inner;
    },

    delete(index: number, length: number) {
      inner.splice(index, length);
      notifySubscribers({} as StorageEvent);
    },

    forEach(callback: (value: T, index: number, array: ArrayStorage<T>) => void) {
      inner.forEach((value, index) => {
        callback(value, index, this);
      });
    },

    subscribe(callback: (event: StorageEvent) => void) {
      const index = subscribers.push(callback);

      return () => {
        subscribers.splice(index - 1, 1);
      };
    },

    [Symbol.iterator](): IterableIterator<T> {
      return inner[Symbol.iterator]();
    }
  } satisfies ArrayStorage<T>;
};

const createMockMapStorage = <K extends string, V>() => {
  const inner = new Map<K, V>();
  const subscribers: Array<(event: StorageEvent) => void> = [];

  const notifySubscribers = (event: StorageEvent) => {
    subscribers.forEach((t) => {
      t(event);
    });
  };

  return {
    get size() {
      return inner.size;
    },

    get(key: K) {
      return inner.get(key);
    },

    set(key: K, value: V) {
      inner.set(key, value);

      notifySubscribers({} as StorageEvent);

      return value;
    },

    delete(key: K) {
      const returnValue = inner.delete(key);
      notifySubscribers({} as StorageEvent);

      return returnValue;
    },

    has(key: K) {
      return inner.has(key);
    },

    clear() {
      inner.clear();
      notifySubscribers({} as StorageEvent);
    },

    entries() {
      return inner.entries();
    },

    [Symbol.iterator](): IterableIterator<[K, V]> {
      return inner[Symbol.iterator]();
    },

    subscribe(callback: (event: StorageEvent) => void) {
      const index = subscribers.push(callback);

      return () => {
        subscribers.splice(index - 1, 1);
      };
    }
  } satisfies MapStorage<K, V>;
};

export const mockStorageProvider = {
  sync: function (
    _events: {
      subscribe<Key extends 'message'>(event: Key, callback: MessageEvents[Key]): () => void;
      subscribeOnce<Key extends 'message'>(event: Key, callback: MessageEvents[Key]): () => void;
      waitUntil<Key extends 'message'>(event: Key): Promise<Parameters<MessageEvents[Key]>>;
    },
    _sender: NetworkSender
  ): Promise<boolean> {
    throw new Error('Function not implemented.');
  },
  getStorage: function (): IStorage {
    return {
      root: this,
      applyUpdate: function (_event: Uint8Array): void {
        throw new Error('Function not implemented.');
      },
      getArray: function <T>(_name: string): ArrayStorage<T> {
        return createMockArrayStorage();
      },
      getMap: function <K extends string, V>(_name: string): MapStorage<K, V> {
        return createMockMapStorage();
      },
      subscribe: function <T>(
        object: ArrayStorage<T> | MapStorage<string, T>,
        callback: (event: StorageEvent) => void
      ): () => void {
        return object.subscribe(callback);
      },
      events: {
        subscribe: function <Key extends 'update'>(
          _event: Key,
          _callback: StorageEvents[Key]
        ): () => void {
          return () => {};
        },
        subscribeOnce: function <Key extends 'update'>(
          _event: Key,
          _callback: StorageEvents[Key]
        ): () => void {
          throw new Error('Function not implemented.');
        },
        waitUntil: function <Key extends 'update'>(
          _event: Key
        ): Promise<Parameters<StorageEvents[Key]>> {
          throw new Error('Function not implemented.');
        }
      }
    };
  },
  type: 'yjs' as const
};
