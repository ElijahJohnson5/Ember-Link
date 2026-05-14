<script lang="ts">
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';
</script>

# **Custom Messages**

Custom messages are a typed pub/sub layer between every client in a channel. The SDK does not inspect the payload. The application defines the message shape and the SDK delivers messages to every connected peer. Custom messages are a good fit for ephemeral events that do not belong in [presence](/concepts/presence) or [storage](/concepts/storage), such as a "user X just clicked the buzzer" notification, a floating reaction emoji, or a chat message that an external backend is about to persist.

## **Declaring the Message Shape**

A discriminated union is the recommended shape for custom messages, because the receive callback can narrow on the discriminator before reading any payload fields.

```typescript copyButton
declare global {
	interface EmberLink {
		Custom:
			| { kind: 'reaction'; emoji: string; from: string }
			| { kind: 'buzzer' }
			| { kind: 'cursor-flash'; clientId: string };
	}
}
export {};
```

See [Type augmentation](/concepts/type-augmentation) for details on the augmentation mechanism.

## **Sending a Custom Message**

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
channel.sendCustomMessage({
	kind: 'reaction',
	emoji: '🎉',
	from: 'me'
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
channel.sendCustomMessage({
	kind: 'reaction',
	emoji: '🎉',
	from: 'me'
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { useCustomMessage } from '@ember-link/react';

function ReactionButton() {
	// `useCustomMessage` handles both directions. The callback fires
	// for every incoming custom message. The returned function sends
	// an outgoing custom message.
	const sendMessage = useCustomMessage((msg) => {
		if (msg.kind === 'reaction') showReaction(msg.emoji);
	});

	return (
		<button onClick={() => sendMessage({ kind: 'reaction', emoji: '🎉', from: 'me' })}>
			🎉
		</button>
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

<button
	onclick={() =>
		channel.getRawChannel().sendCustomMessage({
			kind: 'reaction',
			emoji: '🎉',
			from: 'me'
		})}
>
	🎉
</button>
```

{/snippet}
</LibrarySelectorTabs>

## **Receiving a Custom Message**

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
const unsub = channel.events.subscribe('customMessage', (msg) => {
	if (msg.kind === 'reaction') {
		showReaction(msg.emoji);
	}
});

unsub();
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const unsub = channel.events.subscribe('customMessage', (msg) => {
	// `msg` is narrowed to your `EmberLink.Custom` union.
	switch (msg.kind) {
		case 'reaction':
			showReaction(msg.emoji);
			break;
		case 'buzzer':
			flashScreen();
			break;
	}
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
// The callback in `useCustomMessage` runs on every incoming message.
// `useCustomMessage` continues to work even when the application
// only consumes messages.
useCustomMessage((msg) => {
	console.log('got', msg);
});
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();

	$effect(() => {
		const raw = channel.getRawChannel();
		return raw.events.subscribe('customMessage', (msg) => {
			console.log('got', msg);
		});
	});
</script>
```

{/snippet}
</LibrarySelectorTabs>

## **Delivery Semantics**

Custom messages are fire-and-forget. The server fans each message out to every client that is currently connected to the channel. The SDK does not replay past messages on connect, so a client that connects after a message has been sent does not see that message. Applications that need persistence or replay should write the event into a [storage](/concepts/storage) array instead.

The sender of a custom message does not receive its own message back through the `customMessage` event. The local UI should update optimistically alongside the call to `sendCustomMessage`.

## **When to Use Storage Instead**

The table below summarizes when custom messages are the right tool, and when [storage](/concepts/storage) is a better fit.

| **Requirement**                                  | **Use**                                                         |
| ------------------------------------------------ | --------------------------------------------------------------- |
| Late-joiners should see past events.             | Storage (CRDT array).                                           |
| The event must be persisted on the server.       | Storage.                                                        |
| Conflict-free merges across offline clients.     | Storage.                                                        |
| Throwaway "this just happened" notifications.    | Custom messages.                                                |
| A reaction emoji animation.                      | Custom messages.                                                |
| Chat where the application does not have its own backend. | Storage. The application almost always wants message history. |
