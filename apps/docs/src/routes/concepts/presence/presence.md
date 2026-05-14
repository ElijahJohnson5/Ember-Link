<script lang="ts">
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';
</script>

# **Presence**

Presence is per-user, ephemeral state on a channel. Typical examples include the current cursor position, a typing indicator, or the colour the user has selected. Presence is removed automatically when the user disconnects from the channel. This page covers how to declare the shape of your presence type, how to send and read presence, how to use the `presenceThrottle` channel option to coalesce sends, and the lifecycle events that the SDK emits when presence changes.

## **Declaring the Presence Shape**

Presence may be any `Record<string, unknown>`. The shape is declared once globally so that every SDK call site infers it automatically.

```typescript copyButton
declare global {
	interface EmberLink {
		Presence: {
			cursor: { x: number; y: number } | null;
			color: string;
			isTyping: boolean;
		};
	}
}
export {};
```

See [Type augmentation](/concepts/type-augmentation) for the reasoning behind this pattern and for the per-call override mechanism.

## **Sending Presence**

The local client broadcasts its presence by calling `updatePresence` with the full presence object. `updatePresence` replaces the previous presence state in its entirety rather than merging it, so the caller must send the complete object each time, or maintain a local copy and spread it.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
channel.updatePresence({
	cursor: { x: 12, y: 8 },
	color: '#7C3AED',
	isTyping: false
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
channel.updatePresence({
	cursor: { x: 12, y: 8 },
	color: '#7C3AED',
	isTyping: false
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { useMyPresence } from '@ember-link/react';

function CursorLayer() {
	const [, setPresence] = useMyPresence();

	return (
		<div
			onPointerMove={(e) =>
				setPresence({
					cursor: { x: e.clientX, y: e.clientY },
					color: '#7C3AED',
					isTyping: false
				})
			}
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
</script>

<div
	onpointermove={(e) =>
		channel.updatePresence({
			cursor: { x: e.clientX, y: e.clientY },
			color: '#7C3AED',
			isTyping: false
		})}
></div>
```

{/snippet}
</LibrarySelectorTabs>

## **Throttling Presence Sends**

Pointer events fire at the screen's refresh rate, which is usually 60Hz and can be as high as 120Hz on newer hardware. Sending one WebSocket message per pointer event saturates the channel's bandwidth and degrades the experience for every connected peer.

The `presenceThrottle` option on `joinChannel` configures the SDK to coalesce presence sends. The first call inside the throttle window fires immediately. Subsequent calls within the window are deferred and their state is overwritten by later calls. The most recent state then fires on a trailing timer at the end of the window.

```typescript copyButton
client.joinChannel('demo', {
	presenceThrottle: 33  // roughly 30Hz on the wire.
});
```

The table below offers some reasonable values for common use cases.

| **Throttle**  | **On-wire rate**       | **Use case**                                                                                                              |
| ------------- | ---------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `0` or omitted | every call             | Tests, or when the caller is already debouncing presence updates upstream.                                                |
| `16`          | roughly 60Hz           | Animation-smooth cursors. Uses noticeably more bandwidth than the default.                                                |
| `33`          | roughly 30Hz           | The recommended default for cursor tracking. Visually smooth at typical screen sizes and uses half the messages of `16`.   |
| `100`         | roughly 10Hz           | Typing indicators, status badges, and other presence values where the user is unlikely to perceive sub-100ms latency.     |

For ultra-smooth cursor motion at low send rates, pair `presenceThrottle: 33` with a per-peer `requestAnimationFrame` lerp on the rendering side. See the [Live cursors example](/cursors) for a complete implementation.

## **Reading Your Own Presence**

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
const me = channel.getPresence();   // P | null

channel.events.subscribe('presence', (next) => {
	// The callback fires once after every successful updatePresence.
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const me = channel.getPresence();
channel.events.subscribe('presence', (next) => {});
```

{/snippet}
{#snippet react()}

```tsx copyButton
const [me, setMe] = useMyPresence();
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
{channel.myPresence}   <!-- $state-backed; re-renders automatically. -->
```

{/snippet}
</LibrarySelectorTabs>

## **Reading Other Peers**

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
const others = channel.getOthers();   // User<P>[]

channel.events.others.subscribe('update', (user) => {
	// `user` has shape `P & { clientId: string }`, so the presence
	// fields appear directly on the user object. For the presence
	// type declared above, the user would look like:
	// { clientId: 'abc', cursor: {…}, color: '#…', isTyping: false }
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const others = channel.getOthers();
channel.events.others.subscribe('update', (user) => {
	// `user` is `EmberLink['Presence'] & { clientId: string }`.
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
const others = useOthers();
// `others` has the type `({ clientId: string } & Presence)[]`.
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
{#each channel.others as other (other.clientId)}
	<div style="color: {other.color}">{other.clientId}</div>
{/each}
```

{/snippet}
</LibrarySelectorTabs>

## **Lifecycle Events**

The `channel.events.others` observable fires four event types in response to peer activity.

| **Event**   | **Description**                                                                                                                                                                                |
| ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `join`      | Fires when a new peer appears with presence set, or when a previously-silent peer sets presence for the first time.                                                                            |
| `update`    | Fires when a known peer calls `updatePresence`.                                                                                                                                                |
| `leave`     | Fires when a known peer disconnects from the channel or sets its presence to `null`.                                                                                                            |
| `reset`     | Fires when the local channel disconnects from the server. The SDK clears the local list of peers to avoid keeping stale data around. Peers then re-appear via `join` events on reconnection. |

The `reset` event is worth highlighting. When the local WebSocket drops, the SDK can no longer trust that the peers it knew about are still present, so it clears the peer list locally. After the connection recovers, the server re-sends the current presence state for every peer, and those peers re-join via `join` events.
