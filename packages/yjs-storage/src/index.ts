import { createEventEmitter } from '@ember-link/event-emitter';
import {
  IStorageProvider,
  IStorage,
  ArrayStorage,
  MapStorage,
  StorageEvents,
  StorageEvent
} from '@ember-link/storage';

import * as Y from 'yjs';

export type YjsStorageProvider = IStorageProvider;

const createYArrayStorage = <T>(name: string, yArray: Y.Array<T>) => {
  return {
    subscribe(callback: (event: StorageEvent) => void) {
      const wrappedCallback = (yArrayEvent: Y.YArrayEvent<T>, _transaction: Y.Transaction) => {
        callback({ changes: yArrayEvent.changes });
      };

      yArray.observe(wrappedCallback);

      return () => {
        yArray.unobserve(wrappedCallback);
      };
    },

    toArray() {
      return yArray.toArray();
    },

    [Symbol.iterator](): IterableIterator<T> {
      return yArray[Symbol.iterator]();
    },

    insertAt(index: number, value: T): void {
      yArray.insert(index, [value]);
    },

    push(value: T): void {
      yArray.push([value]);
    },

    delete(index: number, length: number) {
      return yArray.delete(index, length);
    },

    forEach(callback: (value: T, index: number, array: ArrayStorage<T>) => void) {
      return yArray.forEach((value, index) => {
        callback(value, index, this);
      });
    },

    get length() {
      return yArray.length;
    }
  } satisfies ArrayStorage<T>;
};

const createYMapStorage = <V>(name: string, yMap: Y.Map<V>) => {
  return {
    entries(): IterableIterator<[string, V]> {
      return yMap.entries();
    },

    [Symbol.iterator](): IterableIterator<[string, V]> {
      return yMap[Symbol.iterator]();
    },

    get(key: string) {
      return yMap.get(key);
    },

    set(key: string, value: V) {
      return yMap.set(key, value);
    },

    delete(key: string) {
      return yMap.delete(key);
    },

    has(key: string) {
      return yMap.has(key);
    },

    clear() {
      return yMap.clear();
    },

    get size() {
      return yMap.size;
    },

    subscribe(callback: (event: StorageEvent) => void) {
      const wrappedCallback = (yArrayEvent: Y.YMapEvent<V>, _transaction: Y.Transaction) => {
        callback({ changes: yArrayEvent.changes });
      };

      yMap.observe(wrappedCallback);

      return () => {
        yMap.unobserve(wrappedCallback);
      };
    }
  } satisfies MapStorage<string, V>;
};

export function createYJSStorageProvider(): IStorageProvider {
  const eventEmitter = createEventEmitter<StorageEvents>();

  const doc = new Y.Doc();

  doc.on('update', (data, origin) => {
    if (origin !== 'backend') {
      eventEmitter.emit('update', data);
    }
  });

  const storage: IStorage = {
    root: doc,
    getArray: (name: string) => {
      return createYArrayStorage(name, doc.getArray(name));
    },

    getMap: <K extends string, V>(name: string) => {
      return createYMapStorage(name, doc.getMap(name)) as MapStorage<K, V>;
    },

    subscribe: (object, callback) => {
      return object.subscribe(callback);
    },

    applyUpdate: (event) => {
      Y.applyUpdate(doc, event, 'backend');
    },

    events: eventEmitter.observable
  };

  return {
    sync: async (events, sender) => {
      return new Promise((resolve) => {
        const data = Y.encodeStateVector(doc);

        sender.message({
          data: Array.from(data),
          syncType: 'SyncStep1'
        });

        events.subscribe('message', (event) => {
          if (event.syncType === 'SyncStep1') {
            sender.message({
              data: Array.from(Y.encodeStateAsUpdate(doc, Uint8Array.from(event.data))),
              syncType: 'SyncStep2'
            });
          } else if (event.syncType === 'SyncStep2') {
            Y.applyUpdate(doc, Uint8Array.from(event.data));
          } else if (event.syncType === 'SyncDone') {
            resolve(true);
          }
        });
      });
    },
    getStorage: () => storage,
    type: 'yjs'
  };
}
