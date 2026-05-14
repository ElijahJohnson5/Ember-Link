# **@ember-link/yjs-provider**

`@ember-link/yjs-provider` exposes the raw `Y.Doc` of a channel as a Yjs provider. The package is the right tool when an application is using a Yjs-aware editor (Tiptap, BlockNote, ProseMirror, and so on) that wants to sync against the underlying CRDT document directly, bypassing the `ArrayStorage` and `MapStorage` collection APIs.

For application-level state (todo lists, canvas shapes, document metadata, and so on), use [@ember-link/yjs-storage](/packages/yjs-storage) and the storage hooks instead. The two packages can be used together on the same channel if needed.

## **Installation**

```sh copyButton
yarn add @ember-link/yjs-provider
```

`@ember-link/yjs-provider` depends on `yjs` as a peer dependency.

## **Basic Usage**

```typescript copyButton
import { createClient } from '@ember-link/core';
import { getYjsProviderForChannel } from '@ember-link/yjs-provider';

const client = createClient({ baseUrl: 'http://localhost:9000' });
const { channel } = client.joinChannel('my-doc');

// `provider` is an `EmberLinkYjsProvider`. The underlying `Y.Doc`
// is accessible through `provider.doc` and can be passed to any
// Yjs-aware editor extension.
const provider = getYjsProviderForChannel(channel);
```

### **With Tiptap**

```typescript copyButton
import { Editor } from '@tiptap/core';
import { Collaboration } from '@tiptap/extension-collaboration';
import { CollaborationCaret } from '@tiptap/extension-collaboration-caret';
import { getYjsProviderForChannel } from '@ember-link/yjs-provider';

const provider = getYjsProviderForChannel(channel);

new Editor({
	extensions: [
		Collaboration.configure({ document: provider.getYDoc() }),
		CollaborationCaret.configure({
			provider,
			user: { name: 'Alice', color: '#7C3AED' }
		})
	]
});
```

See the [Collaborative editor example](/collaborative) for the complete Tiptap integration.

## **API**

### **getYjsProviderForChannel**

```typescript
function getYjsProviderForChannel(channel: Channel): EmberLinkYjsProvider;
```

Returns the `EmberLinkYjsProvider` bound to the supplied channel. The function memoises the result internally with a `WeakMap` keyed on the channel, so repeated calls for the same channel return the same provider instance.

When the channel emits its `destroy` event, the SDK calls `provider.destroy()` automatically and removes the entry from the internal map.

### **EmberLinkYjsProvider**

The class exposed as the return value of `getYjsProviderForChannel`. Instances expose the underlying `Y.Doc` and an awareness object compatible with the standard Yjs awareness protocol consumed by collaborative editor extensions.

| **Member**     | **Description**                                                                                                            |
| -------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `getYDoc()`    | Returns the `Y.Doc` that backs the provider. Pass the return value to Yjs-aware editor extensions.                          |
| `awareness`    | The `Awareness` instance for cursor and selection sharing in compatible editors.                                            |
| `synced`       | Boolean getter. `true` once the provider has finished its initial sync with the server.                                     |
| `on('synced', cb)` | Fires when the initial sync with the server completes. The provider extends `lib0/observable`, so `off` and `emit` are also available. |
| `destroy()`    | Tears down the provider and unsubscribes from the channel. Called automatically when the channel emits `destroy`.            |

## **Related**

- [@ember-link/yjs-storage](/packages/yjs-storage) for application-level shared state.
- [@ember-link/core](/packages/core) for the `Channel` type that `getYjsProviderForChannel` takes as input.
- [Collaborative editor example](/collaborative) for an end-to-end Tiptap integration.
