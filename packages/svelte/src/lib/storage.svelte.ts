import type { ArrayStorage, IStorage, MapStorage, StorageEvent } from '@ember-link/core';
import { SvelteMap } from 'svelte/reactivity';

export class SvelteArrayStorage<T> implements ArrayStorage<T> {
	private inner: ArrayStorage<T>;
	public current = $state<Array<T>>([]);

	constructor(arrayStorage: ArrayStorage<T>) {
		this.inner = arrayStorage;

		this.current = [...this.inner];

		arrayStorage.subscribe(() => {
			this.current = [...this.inner];
		});
	}

	public get length() {
		return this.inner.length;
	}

	toArray() {
		return this.inner.toArray();
	}
	[Symbol.iterator](): IterableIterator<T> {
		return this.inner[Symbol.iterator]();
	}

	insertAt(index: number, value: T): void {
		this.inner.insertAt(index, value);
	}

	push(value: T): void {
		this.inner.push(value);
	}

	delete(index: number, length: number) {
		return this.inner.delete(index, length);
	}

	forEach(callback: (value: T, index: number, array: ArrayStorage<T>) => void) {
		return this.inner.forEach((value, index) => {
			callback(value, index, this);
		});
	}

	subscribe(callback: (event: StorageEvent) => void) {
		return this.inner.subscribe(callback);
	}
}

export class SvelteMapStorage<K extends string, V> implements MapStorage<K, V> {
	private inner: MapStorage<K, V>;
	public current = new SvelteMap<K, V>();

	constructor(mapStorage: MapStorage<K, V>) {
		this.inner = mapStorage;

		for (const [key, value] of this.inner) {
			this.current.set(key, value);
		}

		mapStorage.subscribe(() => {
			this.current.clear();
			for (const [key, value] of this.inner) {
				this.current.set(key, value);
			}
		});
	}

	entries(): IterableIterator<[K, V]> {
		return this.inner.entries();
	}

	[Symbol.iterator](): IterableIterator<[K, V]> {
		return this.inner[Symbol.iterator]();
	}

	get(key: K) {
		return this.inner.get(key);
	}

	set(key: K, value: V) {
		return this.inner.set(key, value);
	}

	delete(key: K) {
		return this.inner.delete(key);
	}

	has(key: K) {
		return this.inner.has(key);
	}

	clear() {
		return this.inner.clear();
	}

	public get size() {
		return this.inner.size;
	}

	subscribe(callback: (event: StorageEvent) => void) {
		return this.inner.subscribe(callback);
	}
}

export class SvelteStorage {
	storage: IStorage;

	constructor(storage: IStorage) {
		this.storage = storage;
	}

	getArray<T>(name: string): SvelteArrayStorage<T> {
		const array = this.storage.getArray<T>(name);

		return new SvelteArrayStorage<T>(array);
	}

	getMap<K extends string, V>(name: string): SvelteMapStorage<K, V> {
		const map = this.storage.getMap<K, V>(name);

		return new SvelteMapStorage(map);
	}
}
