# @ember-link/storage

## What is @ember-link/storage?

> **@ember-link/storage** provides the interfaces and core types for collaborative storage in the EmberLink ecosystem. It powers the real-time shared data layer for arrays, maps, and custom CRDT implementations across frameworks.

This package is designed to be used by developers building custom storage backends or extending existing storage types. It defines a pluggable interface (**IStorageProvider**) and core reactive data structures like **ArrayStorage** and **MapStorage**.

---

## Installation

```bash copyButton
yarn add @ember-link/storage
```

## API

### **IStorageProvider**

Used to connect a custom storage backend to the EmberLink runtime.

| Property/Method     | Description                                                              |
| ------------------- | ------------------------------------------------------------------------ |
| `type: StorageType` | The type of storage being implemented (used for syncing/identification). |
| `sync()`            | Connects the storage provider to a network source and syncs data.        |
| `getStorage()`      | Returns an object implementing **IStorage**.                             |

---

### **IStorage**

The main interface returned by **getStorage()**, containing typed accessors for working with collaborative storage.

| Method                           | Description                                  |
| -------------------------------- | -------------------------------------------- |
| `getArray<T>(name: string)`      | Returns a named `ArrayStorage<T>` CRDT.      |
| `getMap<K, V>(name: string)`     | Returns a named `MapStorage<K, V>` CRDT.     |
| `applyUpdate(event: Uint8Array)` | Applies an incoming update from the network. |
| `subscribe()`                    | Subscribes to changes for a specific CRDT.   |
| `events`                         | Observable stream of raw update events.      |

---

### <a id="array-storage" href="#array-storage">**`ArrayStorage<T>`**</a>

CRDT-like array structure that syncs in real time.

| Method                | Description                                     |
| --------------------- | ----------------------------------------------- |
| `insertAt()`          | Inserts a value at a specific index.            |
| `replace()`           | Replaces a value at a specific index.           |
| `push()`              | Appends a value to the end.                     |
| `delete()`            | Deletes a number of items starting at an index. |
| `toArray()`           | Returns a shallow copy as a standard array.     |
| `forEach()`           | Iterates over each item in the storage.         |
| `subscribe()`         | Subscribes to updates and changes to the array. |
| `[Symbol.iterator]()` | Allows iteration via `for...of`.                |

---

### <a id="map-storage" href="#map-storage">**`MapStorage<K, V>`**</a>

CRDT-like map structure with support for typed keys and values.

| Method            | Description                                    |
| ----------------- | ---------------------------------------------- |
| `get(key)`        | Retrieves the value associated with a key.     |
| `set(key, value)` | Adds or updates a key-value pair.              |
| `delete(key)`     | Deletes a key-value pair.                      |
| `has(key)`        | Checks if the key exists.                      |
| `clear()`         | Clears the entire map.                         |
| `entries()`       | Returns an iterator over `[key, value]` pairs. |
| `subscribe()`     | Subscribes to updates and changes to the map.  |

---

### `StorageEvent`

All array and map updates emit a `StorageEvent`, which includes:

```ts
type StorageEvent = {
	changes: {
		keys: Map<
			string,
			{
				action: 'add' | 'update' | 'delete';
				oldValue: any;
			}
		>;
	};
};
```
