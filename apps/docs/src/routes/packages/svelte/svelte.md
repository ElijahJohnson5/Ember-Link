# **@ember-link/svelte**

`@ember-link/svelte` is the official Svelte integration for Ember Link. The package re-exports every type and runtime export from `@ember-link/core` and adds two component providers plus a `SvelteChannel` wrapper that exposes the channel's state through `$state`-backed reactive properties. Templates can interpolate `channel.others`, `channel.status`, and `channel.myPresence` directly and the SDK takes care of re-renders.

## **Installation**

```sh copyButton
yarn add @ember-link/svelte
```

## **Basic Usage**

```svelte copyButton
<!-- App.svelte -->
<script lang="ts">
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';
	import Page from './Page.svelte';
</script>

<EmberLinkProvider baseUrl="http://localhost:9000">
	<ChannelProvider channelName="test" presenceThrottle={33}>
		<Page />
	</ChannelProvider>
</EmberLinkProvider>
```

```svelte copyButton
<!-- Page.svelte -->
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	// `getChannelContext` returns a `SvelteChannel` wrapper, which
	// exposes reactive accessors backed by `$state`. The raw core
	// `Channel` object is still accessible through `getRawChannel()`.
	const channel = getChannelContext();

	channel.updatePresence({ data: 'test' });
</script>

{#each channel.others as other (other.clientId)}
	<div>{other.clientId}</div>
{/each}

<div>Current status: {channel.status}</div>
```

## **Providers**

### **EmberLinkProvider**

```svelte
<EmberLinkProvider {...CreateClientOptions}>
	{@render children()}
</EmberLinkProvider>
```

Creates an Ember Link `Client` from the props passed to the provider and stores it in the Svelte context. The props are the same as `CreateClientOptions` from `@ember-link/core`. The component re-runs `setOptions` reactively when props change, and tears the client down on destroy.

### **ChannelProvider**

```svelte
<ChannelProvider channelName={string} {...ChannelOptions}>
	{@render children()}
</ChannelProvider>
```

Joins the named channel and exposes a `SvelteChannel` wrapper in the Svelte context. Channel options are spread directly as component props rather than nested under an `options` prop. The component re-joins the channel reactively when `channelName` or any option changes, and destroys the wrapper on unmount.

## **SvelteChannel**

The object returned by `getChannelContext()`. The class wraps the core `Channel` with `$state`-backed reactive properties.

### **Reactive Properties**

| **Property**  | **Type**            | **Description**                                                                                                |
| ------------- | ------------------- | -------------------------------------------------------------------------------------------------------------- |
| `others`      | `User<P>[]`         | List of other peers connected to the channel. Updates when peers join, leave, or update presence.              |
| `myPresence`  | `P \| null`         | The local client's last-sent presence.                                                                          |
| `status`      | `Status`            | Current WebSocket status. See `Status` in [@ember-link/core](/packages/core).                                  |
| `storage`     | `SvelteStorage \| null` | The reactive storage wrapper for the channel, or `null` when no `storageProvider` is configured.            |

### **Methods**

| **Method**           | **Description**                                                                                                                                |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `updatePresence()`   | Replaces the local client's presence and broadcasts the new value.                                                                              |
| `getRawChannel()`    | Returns the underlying core `Channel<P, C>` for advanced use cases, such as subscribing directly to `channel.events`.                          |
| `getStorage()`       | Returns the `SvelteStorage` wrapper. Throws when no `storageProvider` is configured on the channel.                                            |

## **SvelteStorage**

A reactive wrapper around the core `IStorage` returned by `channel.storage`. The wrapper exposes `getArray` and `getMap` methods that produce `SvelteArrayStorage<T>` and `SvelteMapStorage<K, V>` instances.

### **SvelteArrayStorage&lt;T&gt;**

```typescript
const items = channel.storage.getArray<T>('items');
```

| **Member**            | **Description**                                                                                                                            |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `current`             | `$state`-backed `Array<T>` for interpolation in templates. Re-renders automatically.                                                       |
| `length`              | Length of the underlying array.                                                                                                            |
| `push(value)`         | Appends a value.                                                                                                                           |
| `insertAt(index, value)` | Inserts a value at the given index.                                                                                                     |
| `delete(index, length)` | Removes a range of values starting at the index.                                                                                          |
| `replace(index, value)` | Replaces the value at the index.                                                                                                          |
| `toArray()`           | Returns the underlying array as a plain `Array<T>`.                                                                                        |
| `forEach(callback)`   | Iterates over each value.                                                                                                                  |
| `subscribe(callback)` | Subscribes to raw `StorageEvent` updates. Most callers do not need this because `current` is already reactive.                              |

### **SvelteMapStorage&lt;K, V&gt;**

```typescript
const map = channel.storage.getMap<K, V>('key');
```

| **Member**            | **Description**                                                                                                                            |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `current`             | A `SvelteMap<K, V>` (reactive map from `svelte/reactivity`) holding the current state.                                                      |
| `size`                | Number of entries in the underlying map.                                                                                                   |
| `get(key)`            | Reads the value at `key`.                                                                                                                  |
| `set(key, value)`     | Adds or replaces the entry.                                                                                                                |
| `delete(key)`         | Removes the entry.                                                                                                                         |
| `has(key)`            | Returns `true` when the entry exists.                                                                                                      |
| `clear()`             | Removes every entry.                                                                                                                       |
| `entries()`           | Returns an iterator over `[key, value]` pairs.                                                                                             |
| `subscribe(callback)` | Subscribes to raw `StorageEvent` updates.                                                                                                  |

## **Example: Storage in a Template**

```svelte copyButton
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();
	const items = $derived(channel.storage?.getArray<{ name: string }>('items'));
	const flags = $derived(channel.storage?.getMap<string, boolean>('flags'));
</script>

{#if items}
	<button onclick={() => items.push({ name: 'Svelte collab' })}>Add</button>
	<ul>
		{#each items.current as item (item.name)}
			<li>{item.name}</li>
		{/each}
	</ul>
{/if}

{#if flags}
	{#each flags.current.entries() as [key, val] (key)}
		{#if val}
			<p>{key} is active</p>
		{/if}
	{/each}
{/if}
```

## **Related**

- [@ember-link/core](/packages/core) for the underlying `Channel`, `Client`, and event types.
- [@ember-link/storage](/packages/storage) for the underlying storage interfaces.
- [@ember-link/react](/packages/react) for the React integration.
- [Concepts → Storage](/concepts/storage) for a higher-level overview of storage.
