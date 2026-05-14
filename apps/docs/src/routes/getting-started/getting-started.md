<script lang="ts">
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';
</script>

# **Getting Started**

Ember Link is a real-time collaboration SDK. This guide walks you through installing the SDK, running an Ember Link server, creating a client, joining a channel, broadcasting presence, and wiring up authentication. Pick the library you are using at the top right of the page so every code sample below shows the variant for your stack.

## **Installation**

Install the SDK for the library you are using. The React and Svelte packages re-export everything the core package exports, so you do not need to depend on both.

<LibrarySelectorTabs>
{#snippet js()}

```sh copyButton
yarn add @ember-link/core
```

{/snippet}
{#snippet ts()}

```sh copyButton
yarn add @ember-link/core
```

{/snippet}
{#snippet react()}

```sh copyButton
yarn add @ember-link/react
```

{/snippet}
{#snippet svelte()}

```sh copyButton
yarn add @ember-link/svelte
```

{/snippet}
</LibrarySelectorTabs>

## **Running the Ember Link Server**

The SDK needs to talk to a running Ember Link server. The fastest way to get one running locally is the Docker image.

1. **Download the Docker image:**

   ```sh copyButton
   docker pull emberlinkio/ember-link:latest
   ```

2. **Run the Docker container:**
   The container exposes port `9000`. The `ALLOW_UNAUTHORIZED` flag disables the JWT check so that you can prototype without setting up authentication first. See the [Self-hosting](/self-hosting/docker) section for the production setup, and [Self-hosting → Cloudflare Workers](/self-hosting/cloudflare-workers) if you would prefer to run Ember Link on the edge.

   ```sh copyButton
   docker run -d -p 9000:9000 \
     --env PORT=9000 \
     --env HOST=0.0.0.0 \
     --env ALLOW_UNAUTHORIZED=true \
     emberlinkio/ember-link:latest
   ```

### **Server Configuration**

The full list of server environment variables is documented in [Self-hosting → Server config](/self-hosting/config). You can also view the same list on the [project README](https://github.com/ElijahJohnson5/Ember-Link?tab=readme-ov-file#server-config).

## **Creating a Client**

The `Client` owns the WebSocket connection. You create one per application, not per channel. In the React and Svelte SDKs the client is created for you by `EmberLinkProvider` from the `baseUrl` and other options you pass to the provider.

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

export default function App({ children }: { children: React.ReactNode }) {
	return (
		<EmberLinkProvider baseUrl="http://localhost:9000">
			{children}
		</EmberLinkProvider>
	);
}
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

## **Joining a Channel**

A channel is a named room on the server. Any client that joins the same channel name on the same server can see the other clients in that channel and share its storage with them.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
// `leave` returns the borrowed channel to the client's internal
// ref-counted pool. Call it once when you are done with the channel.
const { channel, leave } = client.joinChannel('demo', {
	presenceThrottle: 33
});

leave();
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const { channel, leave } = client.joinChannel('demo', {
	presenceThrottle: 33
});

leave();
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { ChannelProvider } from '@ember-link/react';

export function Room({ children }: { children: React.ReactNode }) {
	return (
		<ChannelProvider channelName="demo" options={{ presenceThrottle: 33 }}>
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

	let { children } = $props();
</script>

<ChannelProvider channelName="demo" presenceThrottle={33}>
	{@render children()}
</ChannelProvider>
```

{/snippet}
</LibrarySelectorTabs>

The `presenceThrottle` option coalesces presence sends so that a 120Hz `pointermove` becomes roughly 30 messages per second on the wire. See [Concepts → Presence](/concepts/presence) for the full list of channel options.

## **Sending and Reading Presence**

Presence is per-user, ephemeral state, such as a cursor position, a typing indicator, or a selected color. It is removed when the user disconnects from the channel.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
// Broadcast your own state to the channel.
channel.updatePresence({ cursor: { x: 0, y: 0 } });

// Subscribe to updates from other clients in the channel.
// `user` has the shape `EmberLink['Presence'] & { clientId: string }`,
// so the presence fields appear directly on the user object.
channel.events.others.subscribe('update', (user) => {
	console.log(user.clientId, 'moved to', user.cursor);
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
declare global {
	interface EmberLink {
		Presence: { cursor: { x: number; y: number } | null };
	}
}

channel.updatePresence({ cursor: { x: 0, y: 0 } });

channel.events.others.subscribe('update', (user) => {
	// `user` is `{ clientId: string } & EmberLink['Presence']`.
	// `user.cursor` is typed `{ x: number; y: number } | null`.
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
import { useMyPresence, useOthers } from '@ember-link/react';

export function CursorLayer() {
	const others = useOthers();
	const [, setPresence] = useMyPresence();

	return (
		<div
			onPointerMove={(e) =>
				setPresence({ cursor: { x: e.clientX, y: e.clientY } })
			}
		>
			{others.length} other peer(s) online
		</div>
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
		channel.updatePresence({ cursor: { x: e.clientX, y: e.clientY } })}
>
	{channel.others.length} other peer(s) online
</div>
```

{/snippet}
</LibrarySelectorTabs>

That snippet is a complete live-cursor app. Open the page in two browser tabs and the count of other peers will update as the second tab joins.

## **Authenticating Users**

`ALLOW_UNAUTHORIZED=true` is meant for prototyping. In production the server requires every client to present a signed JWT. The client requests this token from your own backend by way of the `authEndpoint` option, and verifies the response signature against the `jwtSignerPublicKey` option.

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
const client = createClient({
	baseUrl: 'https://collab.example.com',
	authEndpoint: '/api/emberlink-token',
	jwtSignerPublicKey: process.env.PUBLIC_JWT_SIGNER_KEY
});
```

{/snippet}
{#snippet ts()}

```typescript copyButton
const client = createClient({
	baseUrl: 'https://collab.example.com',
	// `authEndpoint` accepts either a URL the SDK POSTs to with the
	// payload `{ channelName }`, or an async callback returning
	// `{ token }`.
	authEndpoint: '/api/emberlink-token',
	jwtSignerPublicKey: process.env.PUBLIC_JWT_SIGNER_KEY!
});
```

{/snippet}
{#snippet react()}

```tsx copyButton
<EmberLinkProvider
	baseUrl="https://collab.example.com"
	authEndpoint="/api/emberlink-token"
	jwtSignerPublicKey={process.env.NEXT_PUBLIC_JWT_SIGNER_KEY}
>
	{children}
</EmberLinkProvider>
```

{/snippet}
{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { EmberLinkProvider } from '@ember-link/svelte';
	import { env } from '$env/dynamic/public';

	let { children } = $props();
</script>

<EmberLinkProvider
	baseUrl="https://collab.example.com"
	authEndpoint="/api/emberlink-token"
	jwtSignerPublicKey={env.PUBLIC_JWT_SIGNER_KEY}
>
	{@render children()}
</EmberLinkProvider>
```

{/snippet}
</LibrarySelectorTabs>

The token returned by your backend must be signed with the private half of `jwtSignerPublicKey` and include the channels the user is allowed to join. The full token shape is documented in [Self-hosting → Server config](/self-hosting/config).

## **Next Steps**

- Read [Concepts → Channels](/concepts/channels) to see the channel lifecycle in detail, including the ref-counting behavior of `leave()`.
- Read [Concepts → Storage](/concepts/storage) to add a shared CRDT document to your channel for data that should outlive the session.
- Browse the [Live cursors example](/cursors) to see the homepage cursor demo with its source code.
- View the full API for each package under [SDKs](/packages).
- Join the Ember Link community on [Discord](https://discord.gg/YU2wGQtgE7) for support and updates.
