<script lang="ts">
  import { selectedLibrary } from '$lib/library.svelte';
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';
</script>

# **Getting Started**

Ember Link makes adding real-time collaboration to your app effortless. This guide will help you set up Ember Link and start using its core features.

## **Installation**

You can install Ember Link via **npm** or **yarn** or any other package manager you use:

{#if selectedLibrary.current.value === 'js' || selectedLibrary.current.value === 'ts'}

```sh
npm install @ember-link/core
# or
yarn add @ember-link/core
```

{/if}

{#if selectedLibrary.current.value === 'react'}

```sh
npm install @ember-link/react
# or
yarn add @ember-link/react
```

{/if}

{#if selectedLibrary.current.value === 'svelte'}

```sh
npm install @ember-link/svelte
# or
yarn add @ember-link/svelte
```

{/if}

## **Running the Ember Link Server**

The easiest way to get started is by running the latest Ember Link Docker image locally.

1. **Download the Docker image:**

   ```sh copyButton
   docker pull emberlinkio/ember-link:latest
   ```

2. **Run the Docker container:**
   This will run the server on port `9000` and expose it for your application to connect. The `ALLOW_UNAUTHORIZED` flag is enabled for easier testing.

   ```sh copyButton
   docker run -d -p 9000:9000 --env PORT=9000 --env HOST=0.0.0.0 --env ALLOW_UNAUTHORIZED=true emberlinkio/ember-link:latest
   ```

### **Server Configuration**

View all server configuration environment variables <a href="https://github.com/ElijahJohnson5/Ember-Link?tab=readme-ov-file#server-config" target="_blank">here</a>

## **Connecting to Ember Link**

{#if selectedLibrary.current.value === 'js' || selectedLibrary.current.value === 'ts'}

After installing, import and create a client instance with the host and port you are running the server on:

{/if}

{#if selectedLibrary.current.value === 'react' || selectedLibrary.current.value === 'svelte'}

After installing, wrap your root component with an **EmberLinkProvider**

{/if}

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
import { ChannelProvider, EmberLinkProvider } from '@ember-link/react';

function App() {
	return <EmberLinkProvider baseUrl="http://localhost:9000">{children}</EmberLinkProvider>;
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

{#if selectedLibrary.current.value === 'js' || selectedLibrary.current.value === 'ts'}

Once you have a client instance you can then connect to a **Channel**

{/if}

{#if selectedLibrary.current.value === 'react' || selectedLibrary.current.value === 'svelte'}

Then where you plan on using Ember Link you can wrap that component with a **ChannelProvider** to connect to a **Channel**

{/if}

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyBytton
// Returns the channel and a function that should be called
// once a user leaves the channel
const { channel, leave } = client.joinChannel('test');
```

{/snippet}
{#snippet ts()}

```typescript copyBytton
// Returns the channel and a function that should be called
// once a user leaves the channel
const { channel, leave } = client.joinChannel('test');
```

{/snippet}

{#snippet react()}

```tsx copyButton
function Channel() {
	return (
		<ChannelProvider channelName="test" options={{}}>
			<Page />
		</ChannelProvider>
	);
}
```

{/snippet}

{#snippet svelte()}

```svelte copyButton
<script lang="ts">
	import { ClientProvider } from '@ember-link/svelte';

	let { children } = $props();
</script>

<ChannelProvider channelName="test">
	<Page />
</ChannelProvider>
```

{/snippet}
</LibrarySelectorTabs>

## **Listening to User Presence**

Track when users join or leave the collaboration session and update your own presence:

{#if selectedLibrary.current.value === 'js' || selectedLibrary.current.value === 'ts'}

```typescript copyButton
channel.events.others.subscribe('join', (user) => {
	console.log('User join: ', user);
});

channel.events.others.subscribe('update', (user) => {
	console.log('User update: ', user);
});

channel.events.others.subscribe('leave', (user) => {
	console.log('User leave: ', user);
});

// Sent when we disconnect for some reason so that there
// aren't ghost users hanging around
channel.events.others.subscribe('reset', () => {
	console.log('Users reset');
});

// Update your own presence
channel.updatePresence({ status: 'online' });

// Listen to events with your own presence
channel.events.subscribe('presence', (presence) => {
	console.log('My Presence was updated:', presence);
});
```

{/if}

{#if selectedLibrary.current.value === 'svelte'}

```svelte copyButton
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();

	// Update your own presence
	channel.updatePresence({ status: 'online' });
</script>

<div>
	{channel.myPresence}
	{#each channel.others as other (other.clientId)}
		{other.clientId}
	{/each}
</div>
```

{/if}

{#if selectedLibrary.current.value === 'react'}

```tsx copyButton
import { useMyPresence, useOthers } from '@ember-link/react';
import { useEffect } from 'react';

export const Page = () => {
	const others = useOthers();
	const [myPresence, setMyPresence] = useMyPresence();

	useEffect(() => {
		setMyPresence({ status: 'online' });
	}, [setMyPresence]);

	return (
		<div>
			{myPresence}
			{others.map((other) => {
				return <div>{other.clientId}</div>;
			})}
		</div>
	);
};
```

{/if}

<!-- ## **Using Shared Storage (CRDTs)**

Ember Link provides **conflict-free replicated data types (CRDTs)** to sync state across users:

```typescript copyButton
const doc = client.getDocument('shared-data');

doc.update((state) => {
	state.text = 'Hello, collaborative world!';
});

doc.subscribe((state) => {
	console.log('Updated state:', state.text);
});
``` -->

## **Listening to Websocket status**

{#if selectedLibrary.current.value === 'js' || selectedLibrary.current.value === 'ts'}

```typescript copyButton
channel.events.subscribe('status', (status) => {
	console.log('Current websocket status: ', status);
});
```

{/if}

{#if selectedLibrary.current.value === 'svelte'}

```svelte copyButton
<script lang="ts">
	const channel = getChannelContext();
</script>

<div>
	{channel.status}
</div>
```

{/if}

{#if selectedLibrary.current.value === 'react'}

```tsx copyButton
import { useChannel } from '@ember-link/react';
import { useEffect } from 'react';

export const Page = () => {
	const channel = useChannel();

	useEffect(() => {
		const unsub = channel.events.subscribe('status', (status) => {
			console.log('Current websocket status: ', status);
		});

		return () => {
			unsub();
		};
	}, [channel]);

	return (
		/*
			Display the status as a badge
		*/
	);
};
```

{/if}

## **Next Steps**

- Check out the [full API documentation for each package](/packages)
- Check out examples on the [github repo](https://github.com/ElijahJohnson5/Ember-Link/tree/main/examples)
- Join the Ember Link community on [Discord](https://discord.gg/YU2wGQtgE7) for support & updates
