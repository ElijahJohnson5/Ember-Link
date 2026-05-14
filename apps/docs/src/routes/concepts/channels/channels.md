<script lang="ts">
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';
</script>

# **Channels**

A `Channel` is a named room on the server. Every client that joins the same channel name on the same server can read each other's presence, share each other's storage, and broadcast custom messages to one another. This page documents how to join a channel, what options are available, how the SDK ref-counts channels internally, and how to read and subscribe to channel state.

## **Joining a Channel**

A channel is joined either by calling `joinChannel` on a `Client` directly, or by mounting a `ChannelProvider` in a React or Svelte tree. In all cases the call accepts the channel name as the first argument and an optional options object as the second.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
const { channel, leave } = client.joinChannel('demo', {
	presenceThrottle: 33,
	initialPresence: { cursor: null }
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const { channel, leave } = client.joinChannel('demo', {
	presenceThrottle: 33,
	initialPresence: { cursor: null }
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
<ChannelProvider channelName="demo" options={{ presenceThrottle: 33 }}>
	<Room />
</ChannelProvider>
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<!-- Svelte's ChannelProvider spreads channel options directly as
     component props, so `presenceThrottle` is a top-level attribute
     rather than being nested under an `options` prop as it is in
     React. -->
<ChannelProvider channelName="demo" presenceThrottle={33}>
	<Room />
</ChannelProvider>
```

{/snippet}
</LibrarySelectorTabs>

### **Channel Options**

Every option is optional. In core code the options are passed as the second argument to `joinChannel`. In React they are passed as the `options` prop on `ChannelProvider`. In Svelte they are spread directly as component props on `ChannelProvider`.

| **Option**           | **Type**             | **Default** | **Description**                                                                                                                                  |
| -------------------- | -------------------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `initialPresence`    | `P`                  | none        | Presence value sent on join, before any explicit `updatePresence` call.                                                                          |
| `presenceThrottle`   | `number` (ms)        | `0`         | Coalesces presence sends to at most one per `presenceThrottle` milliseconds. A value of `33` produces roughly 30 messages per second on the wire. A value of `0` or an omitted option disables throttling. |
| `storageProvider`    | `IStorageProvider`   | none        | Enables shared CRDT storage on the channel. See [Storage](/concepts/storage) for the available providers.                                        |
| `autoConnect`        | `boolean`            | `true`      | When `false`, the channel is created but no WebSocket connection is opened until you call `channel.connect()` yourself.                          |

## **Ref-counting**

If two parts of your application both ask the client for `'demo'`, they receive the same underlying channel. The client maintains one WebSocket connection and one server-side room for the channel. Each caller receives its own `leave` function. The channel is torn down only when the last `leave` function fires. This makes it safe to mount `ChannelProvider` for the same channel in two unrelated places in your component tree without worrying about double-connecting.

## **Reading State**

The `Channel` object exposes a small set of accessors for its current state. The accessors differ slightly across SDKs. The core SDK uses getter methods, because the underlying state is not reactive at the core level. The React SDK wraps those getters in hooks backed by `useSyncExternalStore`. The Svelte SDK wraps them in `$state`-backed properties on the `SvelteChannel` instance.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
channel.getStatus();    // Status
channel.getOthers();    // User<P>[]
channel.getPresence();  // P | null    The local client's last-sent presence.
channel.getName();      // string      The channel name.
channel.hasStorage();   // boolean     True when a storageProvider is configured.
```

{/snippet}
{#snippet ts()}

```typescript copyButton
channel.getStatus();    // Status
channel.getOthers();    // User<P>[]
channel.getPresence();  // P | null
channel.getName();      // string
channel.hasStorage();   // boolean
```

{/snippet}
{#snippet react()}

```tsx copyButton
const status = useStatus();             // Status
const others = useOthers();             // User<P>[]
const [me, setMe] = useMyPresence();    // [P | null, (next: P) => void]
const channel = useChannel();           // The raw Channel<P, C> as an escape hatch.
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';
	const channel = getChannelContext();
</script>

<!-- The properties below are $state-backed and update reactively. -->
{channel.status}             <!-- Status -->
{channel.others.length}      <!-- User<P>[] -->
{channel.myPresence}         <!-- P | null -->

<button onclick={() => channel.updatePresence({ cursor: null })}>
	Clear
</button>
```

{/snippet}
</LibrarySelectorTabs>

### **Connection Status Values**

The `Status` type is one of `'initial' | 'connecting' | 'connected' | 'reconnecting' | 'closed' | 'disconnected'`. A newly created channel starts at `'initial'`, transitions through `'connecting'` to `'connected'`, and may transition through `'reconnecting'` if the WebSocket drops and the SDK attempts to recover.

## **Subscribing to Events**

The `channel.events` observable exposes five top-level events for channel-wide state changes, and a nested `others` observable that fires once per peer.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
// Top-level channel events.
channel.events.subscribe('status', (s) => {});
channel.events.subscribe('presence', (myPresence) => {});
channel.events.subscribe('others', (allOthers) => {});
channel.events.subscribe('customMessage', (msg) => {});
channel.events.subscribe('destroy', () => {});

// Per-peer events fire once per peer rather than over the whole list.
// The `user` argument has the shape `P & { clientId: string }`, so
// presence fields appear directly on the user object (for example
// `user.cursor`, not `user.presence.cursor`).
channel.events.others.subscribe('join',   (user) => {});
channel.events.others.subscribe('leave',  (user) => {});
channel.events.others.subscribe('update', (user) => {});
channel.events.others.subscribe('reset',  () => {});

// Every `subscribe` returns its own unsubscribe function.
const unsub = channel.events.subscribe('status', console.log);
unsub();
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const unsub = channel.events.others.subscribe('update', (user) => {
	// `user` is `EmberLink['Presence'] & { clientId: string }`. The
	// presence fields appear directly on the user object, so you reach
	// them with `user.cursor`, `user.color`, and so on.
});
unsub();
```

{/snippet}
{#snippet react()}

```tsx copyButton
// The React hooks (useStatus, useOthers, useMyPresence) handle
// subscription and cleanup for you. Subscribe to `channel.events`
// directly only when you need an event that no hook exposes.
useEffect(() => {
	return channel.events.others.subscribe('join', (user) => {
		console.log('joined:', user.clientId);
	});
}, [channel]);
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();

	// The reactive properties on `channel` (such as `channel.others`
	// and `channel.status`) already update for you. Reach the raw
	// event stream through `getRawChannel()` only when you need a
	// side effect on a specific event.
	$effect(() => {
		const raw = channel.getRawChannel();
		return raw.events.others.subscribe('join', (user) => {
			console.log('joined:', user.clientId);
		});
	});
</script>
```

{/snippet}
</LibrarySelectorTabs>

## **Leaving a Channel**

```typescript
leave();
```

`leave` is idempotent. Calling it more than once on the same handle logs a warning and is otherwise a no-op. Once the last ref-counted reference to a channel has called its `leave` function, the SDK destroys the channel and the server-side room is freed.

## **`destroy()` Compared with `leave()`**

The raw `Channel` object exposes a `destroy()` method as an escape hatch. `destroy()` tears down the WebSocket and unsubscribes every listener immediately, bypassing the ref-counting system used by `leave()`. In almost every case the right method to call is `leave()`. The React and Svelte providers automatically call `destroy()` on unmount, after their own ref-counted `leave()` has fired.
