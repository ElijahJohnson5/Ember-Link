# @ember-link/svelte

## What is @ember-link/svelte?

> **@ember-link/svelte** @ember-link/svelte is the official Svelte integration for the Ember Link SDK. It provides context providers and wrappers that make it easy to connect to real-time channels, manage presence, and send custom messages using idiomatic Svelte patterns.

## Installation

```bash copyButton
yarn install @ember-link/svelte
```

## Basic Usage

```svelte copyButton
<script lang="ts">
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';
</script>

<EmberLinkProvider baseUrl="http://localhost:9000">
	<ChannelProvider channelName="test">
		<Page />
	</ChannelProvider>
</EmberLinkProvider>
```

```svelte copyButton
<!-- Page.svelte -->

<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	// Gets a channel wrapped in a SvelteChannel which has direct accessors for many of the things you need to access
	const channel = getChannelContext();

	channel.updatePresence({ data: 'test' });

	// Get the base channel if you need to
	const rawChannel = channel.getRawChannel();
</script>

{#each channel.others as other (other.clientId)}
	<div>{other.clientId}</div>
{/each}

<div>Current status: {channel.status}</div>
```

## API

### EmberLinkProvider

Simple provider that takes the same parameters as the **createClient** method does.

### ChannelProvider

Simple provider that connects to the channel name that is given, can only be used within an **EmberLinkProvider**

### SvelteChannel

A wrapper around the base Channel, enhanced with reactive $state bindings for Svelte.

#### Reactive State

| Property     | Description                                        |
| ------------ | -------------------------------------------------- |
| `others`     | Array of other connected clients + their presence  |
| `myPresence` | Your own presence state                            |
| `status`     | WebSocket status: "connected", "disconnected" etc. |

#### Methods

| Method             | Description                                 |
| ------------------ | ------------------------------------------- |
| `updatePresence()` | Update your presence on the server          |
| `getRawChannel()`  | Access the underlying core `Channel` object |
| `getStorage()`     | Access CRDT storage bound to this channel   |

### SvelteStorage

A reactive wrapper over collaborative CRDT-backed storage.

`SvelteArrayStorage<T>`

```ts copyButton
const items = storage.getArray<T>('key');
```

- .current – Reactive array state (for $state updates)

- Methods: push(), remove(), etc. All methods can be found [here](/packages/storage)

`SvelteMapStorage<K, V>`

```ts copyButton
const map = storage.getMap<K, V>('key');
```

- .current – Reactive array state (for $state updates)

- Methods: set(), delete(), clear(), etc. All methods can be found [here](/packages/storage)

#### Example Usage

```svelte
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();
	const storage = channel.getStorage();

	const items = storage.getArray<{ name: string }>('items');
	const flags = storage.getMap<string, boolean>('flags');

	$effect(() => {
		items.push({ name: 'Svelte collab ❤️' });
		flags.set('debug', true);
	});
</script>

<ul>
	{#each items.current as item (item.name)}
		<li>{item.name}</li>
	{/each}
</ul>

{#each Object.entries(flags.current) as [key, val] (key)}
	{#if val}
		<p>{key} is active</p>
	{/if}
{/each}
```

## 📚 Related

- [@ember-link/core](/packages/core) – Core WebSocket + channel logic

- [@ember-link/storage](/packages/storage) - Low-level CRDT API

- [@ember-link/react](/packages/react) – React integration
