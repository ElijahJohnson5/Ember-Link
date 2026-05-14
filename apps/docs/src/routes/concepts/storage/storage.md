<script lang="ts">
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';
</script>

# **Storage**

Storage is a CRDT-backed document shared across every client in a channel. The server persists every storage update, so storage is the right place for any data that should outlive the session, including the body of a document, the items in a todo list, or the shapes on a canvas. Storage is opt-in. Any channel that does not configure a `storageProvider` does not pay the cost of syncing storage.

## **Installing a Storage Provider**

Ember Link ships one storage backend today, built on Yjs.

```sh copyButton
yarn add @ember-link/yjs-storage
```

## **Configuring the Channel**

The storage provider is passed to the channel as the `storageProvider` option. In React the option is set inside the `options` prop of `ChannelProvider`. In Svelte the option is spread directly as a component prop.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
import { createClient } from '@ember-link/core';
import { createYJSStorageProvider } from '@ember-link/yjs-storage';

const client = createClient({ baseUrl: 'http://localhost:9000' });

const { channel } = client.joinChannel('doc-42', {
	storageProvider: createYJSStorageProvider()
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
import { createClient } from '@ember-link/core';
import { createYJSStorageProvider } from '@ember-link/yjs-storage';

const client = createClient({ baseUrl: 'http://localhost:9000' });

const { channel } = client.joinChannel('doc-42', {
	storageProvider: createYJSStorageProvider()
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { ChannelProvider } from '@ember-link/react';
import { createYJSStorageProvider } from '@ember-link/yjs-storage';
import { useMemo } from 'react';

function Room({ children }: { children: React.ReactNode }) {
	const provider = useMemo(() => createYJSStorageProvider(), []);

	return (
		<ChannelProvider channelName="doc-42" options={{ storageProvider: provider }}>
			{children}
		</ChannelProvider>
	);
}
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { ChannelProvider } from '@ember-link/svelte';
	import { createYJSStorageProvider } from '@ember-link/yjs-storage';

	const provider = createYJSStorageProvider();
</script>

<ChannelProvider channelName="doc-42" storageProvider={provider}>
	<!-- children -->
</ChannelProvider>
```

{/snippet}
</LibrarySelectorTabs>

`createYJSStorageProvider()` is cheap to call but should still be memoized in React. Passing a fresh object identity to `ChannelProvider` on every render causes the channel to be re-borrowed.

## **Reading and Writing**

Storage exposes two CRDT-backed collection types: arrays and maps. Both collections keep their contents conflict-free across any number of clients and any pattern of concurrent edits.

### **Arrays**

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
const todos = channel.getStorage().getArray<{ id: string; text: string }>('todos');

todos.push({ id: crypto.randomUUID(), text: 'Buy milk' });
todos.insertAt(0, { id: crypto.randomUUID(), text: 'Priority!' });
todos.delete(0, 1);

const unsub = todos.subscribe((event) => {
	// `event.changes` describes the diff. Most callers will simply
	// re-read the array, which is what the React and Svelte SDKs do
	// internally.
	console.log('todos changed', todos.toArray());
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const todos = channel.getStorage().getArray<{ id: string; text: string }>('todos');
todos.push({ id: crypto.randomUUID(), text: 'Buy milk' });
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { useArrayStorage } from '@ember-link/react';

function TodoList() {
	const todos = useArrayStorage<{ id: string; text: string }>('todos');

	return (
		<>
			<button onClick={() => todos.push({ id: crypto.randomUUID(), text: 'New' })}>
				Add
			</button>
			<ul>
				{todos.current.map((t) => (
					<li key={t.id}>{t.text}</li>
				))}
			</ul>
		</>
	);
}
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();
	const todos = $derived(
		channel.storage?.getArray<{ id: string; text: string }>('todos')
	);
</script>

{#if todos}
	<button onclick={() => todos.push({ id: crypto.randomUUID(), text: 'New' })}>
		Add
	</button>
	<ul>
		{#each todos.current as todo (todo.id)}
			<li>{todo.text}</li>
		{/each}
	</ul>
{/if}
```

{/snippet}
</LibrarySelectorTabs>

### **Maps**

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
const meta = channel.getStorage().getMap<string, string>('meta');

meta.set('title', 'My document');
meta.set('owner', 'jane@example.com');
meta.delete('owner');

const title = meta.get('title');     // 'My document'
const has   = meta.has('owner');     // false
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const meta = channel.getStorage().getMap<string, string>('meta');
meta.set('title', 'My document');
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { useMapStorage } from '@ember-link/react';

function DocumentMeta() {
	const meta = useMapStorage<string, string>('meta');

	return (
		<input
			value={meta.current.get('title') ?? ''}
			onChange={(e) => meta.set('title', e.target.value)}
		/>
	);
}
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();
	const meta = $derived(channel.storage?.getMap<string, string>('meta'));
</script>

{#if meta}
	<input
		value={meta.current.get('title') ?? ''}
		oninput={(e) => meta.set('title', e.currentTarget.value)}
	/>
{/if}
```

{/snippet}
</LibrarySelectorTabs>

## **About CRDTs**

A CRDT is a data structure that allows concurrent edits from multiple clients to be merged deterministically, regardless of the order in which the edits arrive at any given client. Ember Link uses Yjs as its CRDT implementation. If you have previously worked with Liveblocks Storage, Y-WebRTC, or Y-WebSocket, the same data model applies.

## **Persistence**

The server persists every storage update to its configured datastore. The Docker image uses SQLite by default. The Cloudflare Workers target uses Durable Objects storage. Clients that join an existing channel receive a snapshot of the current state on connect.

## **Yjs-aware Editors**

If your application is integrating a Yjs-aware editor such as Tiptap, BlockNote, or ProseMirror, use [`@ember-link/yjs-provider`](/packages/yjs-provider) instead of the storage hooks. The Yjs provider exposes the underlying `Y.Doc` so the editor can sync against it directly. See the [Collaborative editor example](/collaborative) for an end-to-end Tiptap integration.
