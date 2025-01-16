import { IStorageProvider, IStorage, ArrayStorage, MapStorage } from '@ember-link/storage';

import * as Y from 'yjs';

class YArrayStorage<T> implements ArrayStorage<T> {
  private name: string;
  private yArray: Y.Array<T>;

  constructor(name: string, yArray: Y.Array<T>) {
    this.yArray = yArray;
    this.name = name;
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

  size: number;
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
}

export function createYJSStorageProvider(): IStorageProvider {
  const doc = new Y.Doc();

  const storage: IStorage = {
    root: doc,
    getArray: (name: string) => {
      return new YArrayStorage(name, doc.getArray(name));
    },
    getMap: (name: string) => {
      return new YMapStorage(name, doc.getMap(name));
    }
  };

  return {
    getStorage: async () => storage
  };
}
