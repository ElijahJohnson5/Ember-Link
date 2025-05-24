import $, { Observable } from 'oby';
import { signal } from 'alien-signals';

export class ReactiveMap<K = unknown, V = unknown> implements Map<K, V> {
  private collection = signal<null>(null);
  private storages = new Map<K, ReturnType<typeof signal<null>>>();
  private vals: Map<K, V>;

  constructor();
  constructor(entries: readonly (readonly [K, V])[] | null);
  constructor(iterable: Iterable<readonly [K, V]>);
  constructor(
    existing?: readonly (readonly [K, V])[] | Iterable<readonly [K, V]> | null | undefined
  ) {
    // TypeScript doesn't correctly resolve the overloads for calling the `Map`
    // constructor for the no-value constructor. This resolves that.
    this.vals = existing ? new Map(existing) : new Map();
  }

  private readStorageFor(key: K): void {
    const { storages } = this;
    let storage = storages.get(key);

    if (storage === undefined) {
      storage = signal(null);
      storages.set(key, storage);
    }

    storage();
  }

  private dirtyStorageFor(key: K): void {
    const storage = this.storages.get(key);

    if (storage) {
      storage(null);
    }
  }

  // **** KEY GETTERS ****
  get(key: K): V | undefined {
    // entangle the storage for the key
    this.readStorageFor(key);

    return this.vals.get(key);
  }

  has(key: K): boolean {
    this.readStorageFor(key);

    return this.vals.has(key);
  }

  // **** ALL GETTERS ****
  entries(): MapIterator<[K, V]> {
    this.collection();

    return this.vals.entries();
  }

  keys(): MapIterator<K> {
    this.collection();

    return this.vals.keys();
  }

  values(): MapIterator<V> {
    this.collection();

    return this.vals.values();
  }

  forEach(fn: (value: V, key: K, map: Map<K, V>) => void): void {
    this.collection();

    this.vals.forEach(fn);
  }

  get size(): number {
    this.collection();

    return this.vals.size;
  }

  [Symbol.iterator](): MapIterator<[K, V]> {
    this.collection();

    return this.vals[Symbol.iterator]();
  }

  get [Symbol.toStringTag](): string {
    return this.vals[Symbol.toStringTag];
  }

  // **** KEY SETTERS ****
  set(key: K, value: V): this {
    this.dirtyStorageFor(key);
    this.collection(null);

    this.vals.set(key, value);

    return this;
  }

  delete(key: K): boolean {
    this.dirtyStorageFor(key);
    this.collection(null);

    return this.vals.delete(key);
  }

  // **** ALL SETTERS ****
  clear(): void {
    this.storages.forEach((s) => s(null));
    this.collection(null);

    this.vals.clear();
  }
}
