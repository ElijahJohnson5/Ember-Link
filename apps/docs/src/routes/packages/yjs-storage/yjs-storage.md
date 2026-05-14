# **@ember-link/yjs-storage**

`@ember-link/yjs-storage` is the Yjs-backed implementation of `IStorageProvider`. The package exposes a single factory function, `createYJSStorageProvider`, that returns a provider you pass as the `storageProvider` option on `joinChannel`. The provider gives the channel shared `ArrayStorage` and `MapStorage` collections that converge conflict-free across every peer.

For a higher-level overview of storage, including the React and Svelte usage patterns, see [Concepts → Storage](/concepts/storage). For the underlying interfaces this package implements, see [@ember-link/storage](/packages/storage). For integration with a Yjs-aware editor like Tiptap, use [@ember-link/yjs-provider](/packages/yjs-provider) instead.

## **Installation**

```sh copyButton
yarn add @ember-link/yjs-storage
```

## **Basic Usage**

```typescript copyButton
import { createClient } from '@ember-link/core';
import { createYJSStorageProvider } from '@ember-link/yjs-storage';

const client = createClient({ baseUrl: 'http://localhost:9000' });

const { channel } = client.joinChannel('doc-42', {
	storageProvider: createYJSStorageProvider()
});

const todos = channel.getStorage().getArray<{ id: string; text: string }>('todos');
todos.push({ id: crypto.randomUUID(), text: 'Buy milk' });
```

## **API**

### **createYJSStorageProvider**

```typescript
function createYJSStorageProvider(): IStorageProvider;
```

Returns a fresh `IStorageProvider` backed by a new `Y.Doc`. The provider implements the `IStorageProvider` interface from `@ember-link/storage`, which means the channel uses it transparently. Calling `channel.getStorage()` after the provider has been wired up returns an `IStorage` whose `getArray` and `getMap` methods produce CRDT-backed collections.

The factory is cheap to call but should be memoised in framework code so that passing fresh object identity to a provider component (such as React's `ChannelProvider`) does not cause the channel to be re-borrowed on every render.

### **YjsStorageProvider**

```typescript
type YjsStorageProvider = IStorageProvider;
```

A type alias re-exported as a convenience. Equivalent to `IStorageProvider`.

## **What You Get**

Collections retrieved from `channel.getStorage()` while this provider is active behave according to the Yjs CRDT model:

- **Concurrent edits merge deterministically.** Two clients editing the same array or map at the same time produce the same final state on every peer, regardless of network reordering.
- **Offline edits are preserved.** A client that loses its connection, makes edits, and then reconnects has its edits merged with whatever happened during the disconnection.
- **The server persists every update.** New clients receive the current state as a snapshot on join, then live updates as they happen.

## **Choosing Between yjs-storage and yjs-provider**

The two Yjs packages serve different goals.

| **Use**                          | **Reach for**                                                                                              |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Application-level state          | `@ember-link/yjs-storage`. Use `useArrayStorage`, `useMapStorage`, or the equivalent Svelte storage hooks.   |
| A Yjs-aware editor extension     | [`@ember-link/yjs-provider`](/packages/yjs-provider). The provider exposes the raw `Y.Doc` for the editor.   |

The two packages can be used on the same channel if needed, though most applications use one or the other.

## **Related**

- [@ember-link/storage](/packages/storage) for the underlying interfaces.
- [@ember-link/yjs-provider](/packages/yjs-provider) for Yjs `Doc` integration with editors.
- [Concepts → Storage](/concepts/storage) for the higher-level overview.
