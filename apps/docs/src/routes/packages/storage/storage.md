# **@ember-link/storage**

`@ember-link/storage` defines the interfaces and core types for collaborative storage in the Ember Link ecosystem. The package describes the contract that any storage backend must implement, and it defines the `ArrayStorage` and `MapStorage` collection shapes that the framework integrations operate against. `@ember-link/storage` is consumed by both the framework integrations and the storage backends, and is published separately so that custom storage backends can implement it without depending on the rest of the SDK.

This page documents the types exposed by the package. For end-user usage of storage from the React or Svelte SDKs, see [Concepts → Storage](/concepts/storage). For the Yjs-based implementation, see [@ember-link/yjs-storage](/packages/yjs-storage).

## **Installation**

Most applications do not install `@ember-link/storage` directly, because the package is re-exported by `@ember-link/core` and by every framework integration. Install it explicitly only when authoring a custom storage backend.

```sh copyButton
yarn add @ember-link/storage
```

## **API**

### **IStorageProvider**

```typescript
interface IStorageProvider {
	type: StorageType;
	sync: (events: Observable<MessageEvents>, sender: NetworkSender) => Promise<boolean>;
	getStorage: () => IStorage;
}
```

The contract that every storage backend must implement.

| **Member**       | **Description**                                                                                                                                                |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `type`           | The `StorageType` enum value that identifies this backend on the wire. Required so the server knows which storage protocol to use for the channel.             |
| `sync`           | Called once by the channel during setup. The provider should subscribe to incoming network events and wire its outgoing updates through the supplied `sender`. |
| `getStorage`     | Returns the `IStorage` interface that application code interacts with.                                                                                          |

### **IStorage**

```typescript
interface IStorage {
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
```

The interface returned by `IStorageProvider.getStorage()`. Application code reaches collections by calling `getArray` or `getMap` with a name.

| **Member**           | **Description**                                                                                                                              |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `root`               | The backend-specific root object. The shape depends on the storage implementation (for the Yjs backend, the underlying `Y.Doc`).             |
| `applyUpdate`        | Applies an incoming update binary received from the server.                                                                                  |
| `getArray<T>(name)`  | Returns a named `ArrayStorage<T>` collection. Repeated calls with the same name return the same underlying collection.                       |
| `getMap<K, V>(name)` | Returns a named `MapStorage<K, V>` collection.                                                                                               |
| `subscribe`          | Subscribes to changes on a specific collection. Returns an unsubscribe function.                                                              |
| `events`             | Observable that fires when the backend produces an outgoing update for the network.                                                          |

### <a id="array-storage" href="#array-storage">**`ArrayStorage<T>`**</a>

```typescript
interface ArrayStorage<T> {
	readonly length: number;
	insertAt(index: number, value: T): void;
	replace(index: number, value: T): void;
	push(value: T): void;
	toArray(): Array<T>;
	delete(index: number, length: number): void;
	forEach(callback: (value: T, index: number, array: ArrayStorage<T>) => void): void;
	[Symbol.iterator](): IterableIterator<T>;
	subscribe(callback: (event: StorageEvent) => void): () => void;
}
```

A CRDT-backed array that syncs in real time across every peer in the channel.

| **Method**            | **Description**                                                                                              |
| --------------------- | ------------------------------------------------------------------------------------------------------------ |
| `length`              | Current number of elements.                                                                                  |
| `insertAt(index, v)`  | Inserts a value at the given index. Other peers see the insert at the same logical position after merging.   |
| `replace(index, v)`   | Replaces the value at the given index.                                                                       |
| `push(v)`             | Appends a value to the end of the array.                                                                     |
| `delete(index, len)`  | Deletes `len` items starting at `index`.                                                                     |
| `toArray()`           | Returns a shallow copy of the current state as a plain array.                                                |
| `forEach(cb)`         | Iterates over each value.                                                                                    |
| `[Symbol.iterator]`   | Allows `for...of` iteration.                                                                                 |
| `subscribe(cb)`       | Subscribes to `StorageEvent` updates. Returns an unsubscribe function.                                       |

### <a id="map-storage" href="#map-storage">**`MapStorage<K, V>`**</a>

```typescript
interface MapStorage<K extends string, V> {
	size: number;
	get(key: K): V | undefined;
	set(key: K, value: V): V;
	delete(key: K): void;
	has(key: K): boolean;
	clear(): void;
	[Symbol.iterator](): IterableIterator<[K, V]>;
	entries(): IterableIterator<[K, V]>;
	subscribe(callback: (event: StorageEvent) => void): () => void;
}
```

A CRDT-backed map with typed string keys and arbitrary value types.

| **Method**       | **Description**                                                                                |
| ---------------- | ---------------------------------------------------------------------------------------------- |
| `size`           | Number of entries.                                                                             |
| `get(key)`       | Returns the value at `key`, or `undefined`.                                                    |
| `set(key, v)`    | Adds or replaces the entry at `key`. Returns the value.                                        |
| `delete(key)`    | Removes the entry at `key`.                                                                    |
| `has(key)`       | Returns `true` when the entry exists.                                                          |
| `clear()`        | Removes every entry.                                                                           |
| `entries()`      | Returns an iterator over `[key, value]` pairs.                                                  |
| `subscribe(cb)`  | Subscribes to `StorageEvent` updates. Returns an unsubscribe function.                          |

### **StorageEvent**

```typescript
interface StorageEvent {
	changes: {
		keys: Map<string, { action: 'add' | 'update' | 'delete'; oldValue: any }>;
	};
}
```

The shape of each update emitted by `subscribe`. The `changes.keys` map describes the diff produced by the latest update. Most callers re-read the collection rather than walking the diff. The React and Svelte SDKs do this internally before exposing the new snapshot to the application.

## **Related**

- [@ember-link/yjs-storage](/packages/yjs-storage) for the Yjs-based implementation of `IStorageProvider`.
- [@ember-link/yjs-provider](/packages/yjs-provider) for direct access to the underlying `Y.Doc` from a Yjs-aware editor.
- [Concepts → Storage](/concepts/storage) for a higher-level overview of storage in Ember Link.
