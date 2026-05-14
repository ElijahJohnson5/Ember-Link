<script lang="ts">
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';
</script>

# **Client**

The `Client` is the entry point to the SDK. The Client owns the WebSocket connection and the authentication lifecycle. An application typically creates one Client. Channels are then borrowed from that Client by name.

## **Creating a Client**

In core JavaScript and TypeScript code the Client is created by calling `createClient`. In the React and Svelte SDKs the Client is created internally by `EmberLinkProvider` from the props you pass to the provider.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
import { createClient } from '@ember-link/core';

const client = createClient({
	baseUrl: 'http://localhost:9000'
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
import { createClient } from '@ember-link/core';

const client = createClient({
	baseUrl: 'http://localhost:9000'
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { EmberLinkProvider } from '@ember-link/react';

<EmberLinkProvider baseUrl="http://localhost:9000">
	{children}
</EmberLinkProvider>
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { EmberLinkProvider } from '@ember-link/svelte';
	let { children } = $props();
</script>

<EmberLinkProvider baseUrl="http://localhost:9000">
	{@render children()}
</EmberLinkProvider>
```

{/snippet}
</LibrarySelectorTabs>

## **Client Options**

The `CreateClientOptions` interface defines the configuration accepted by both `createClient` and `EmberLinkProvider`.

| **Option**             | **Type**                       | **Required** | **Description**                                                                                                                                                                          |
| ---------------------- | ------------------------------ | ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `baseUrl`              | `string`                       | yes          | The base URL of the Ember Link server. The SDK uses this for both authentication requests and the WebSocket URL.                                                                          |
| `authEndpoint`         | `string \| AuthCallback`       | no           | A URL the SDK will `POST` to with `{ channelName }`, or an async callback that returns `{ token }` directly. This option is required for any server that does not have `ALLOW_UNAUTHORIZED` enabled. |
| `jwtSignerPublicKey`   | `string`                       | no           | A PEM-formatted RSA public key that the SDK uses to verify the signature of tokens returned by `authEndpoint`. This option is required whenever `authEndpoint` is set.                  |
| `multiTenant`          | `{ tenantId: string }`         | no           | Sends `tenant_id` on every request. Pair this option with a server running in multi-tenant mode.                                                                                         |
| `polyfills`            | `{ websocket?: IWebSocket }`   | no           | A drop-in replacement for the global `WebSocket` constructor, for environments that do not expose one (for example, some Node test runners).                                             |

## **Authentication Flow**

The `authEndpoint` option accepts one of two shapes. Both produce a signed JWT that the SDK then verifies against `jwtSignerPublicKey` before opening the WebSocket connection.

```typescript copyButton
// URL form. The SDK POSTs JSON `{ channelName }` and expects a JSON
// response of `{ token: string }`.
authEndpoint: 'https://your-app.com/api/emberlink-token'

// Callback form. The implementation is yours, as long as it returns
// either `{ token }` or `{ error, reason }`.
authEndpoint: async (channelName) => {
	const res = await fetch('/api/emberlink-token', {
		method: 'POST',
		body: JSON.stringify({ channelName })
	});
	return res.json(); // { token: string }
}
```

The SDK caches valid tokens until 30 seconds before the `exp` claim on the token. As a result your `authEndpoint` sees roughly one request per channel per token lifetime, rather than one request per `joinChannel` call.

## **Joining Channels**

A channel is borrowed from the Client by calling `joinChannel` with the channel name and an optional options object.

```typescript copyButton
const { channel, leave } = client.joinChannel('demo', {
	presenceThrottle: 33
});
```

Repeated calls to `joinChannel` with the same `channelName` return the same underlying channel, ref-counted internally by the Client. Each caller receives its own `leave` function. See [Channels](/concepts/channels) for the full lifecycle.

## **Tearing Down the Client**

```typescript copyButton
client.destroy();
```

`destroy()` tears down every channel that the Client has created and closes their WebSocket connections. The method is useful when the Client lives in a long-running process and you need a clean shutdown. Calling `joinChannel` on a Client after `destroy()` has been called produces undefined behavior, so create a new Client instead. The React and Svelte providers call `destroy()` on unmount automatically.
