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

class YArrayStorage<T> implements ArrayStorage<T> {
  private name: string;
  private yArray: Y.Array<T>;

  constructor(name: string, yArray: Y.Array<T>) {
    this.yArray = yArray;
    this.name = name;
  }

  subscribe(callback: (event: StorageEvent) => void) {
    const wrappedCallback = (yArrayEvent: Y.YArrayEvent<T>, _transaction: Y.Transaction) => {
      callback({ changes: yArrayEvent.changes });
    };

    this.yArray.observe(wrappedCallback);

    return () => {
      this.yArray.unobserve(wrappedCallback);
    };
  }

  toArray() {
    return this.yArray.toArray();
  }
  [Symbol.iterator](): IterableIterator<T> {
    return this.yArray[Symbol.iterator]();
  }

  insertAt(index: number, value: T): void {
    this.yArray.insert(index, [value]);
  }

  push(value: T): void {
    this.yArray.push([value]);
  }

  delete(index: number, length: number) {
    return this.yArray.delete(index, length);
  }

  forEach(callback: (value: T, index: number, array: ArrayStorage<T>) => void) {
    return this.yArray.forEach((value, index, array) => {
      callback(value, index, this);
    });
  }

  public get length() {
    return this.yArray.length;
  }
}

class YMapStorage<V> implements MapStorage<string, V> {
  private name: string;
  private yMap: Y.Map<V>;

  constructor(name: string, yMap: Y.Map<V>) {
    this.yMap = yMap;
    this.name = name;
  }

  get(key: string) {
    return this.yMap.get(key);
  }

  set(key: string, value: V) {
    return this.yMap.set(key, value);
  }

  delete(key: string) {
    return this.yMap.delete(key);
  }

  has(key: string) {
    return this.yMap.has(key);
  }

  clear() {
    return this.yMap.clear();
  }

  public get size() {
    return this.yMap.size;
  }

  subscribe(callback: (event: StorageEvent) => void) {
    const wrappedCallback = (yArrayEvent: Y.YMapEvent<V>, _transaction: Y.Transaction) => {
      callback({ changes: yArrayEvent.changes });
    };

    this.yMap.observe(wrappedCallback);

    return () => {
      this.yMap.unobserve(wrappedCallback);
    };
  }
}

export function createYJSStorageProvider(): IStorageProvider {
  const eventEmitter = createEventEmitter<StorageEvents>();

  const doc = new Y.Doc();

  doc.on('update', (data) => {
    eventEmitter.emit('update', data);
  });

  const storage: IStorage = {
    root: doc,
    getArray: (name: string) => {
      return new YArrayStorage(name, doc.getArray(name));
    },

    getMap: (name: string) => {
      return new YMapStorage(name, doc.getMap(name));
    },

    subscribe: (object, callback) => {
      return object.subscribe(callback);
    },

    events: eventEmitter.observable
  };

  return {
    getStorage: () => storage
  };
}
