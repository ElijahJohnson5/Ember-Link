export interface IStorageProvider {
  getStorage: () => Promise<IStorage>;
}

export interface ArrayStorage<T> {
  readonly length: number;
  insertAt: (index: number, value: T) => void;
  push: (value: T) => void;
  toArray: () => Array<T>;
  [Symbol.iterator](): IterableIterator<T>;
}

export interface MapStorage<K extends string, V> {
  size: number;
  get: (key: K) => V | undefined;
  set: (key: K, value: V) => V;
  delete: (key: K) => void;
  has: (key: K) => boolean;
  clear: () => void;
}

export interface IStorage {
  root: unknown;
  getArray<T>(name: string): ArrayStorage<T>;
  getMap<K extends string, V>(name: string): MapStorage<K, V>;
}
