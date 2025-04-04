<script lang="ts">
  import { selectedLibrary } from '$lib/library.svelte';
  import CursorExample from '$lib/components/cursor/index.svelte';
</script>

{#if selectedLibrary.current.value === 'js' || selectedLibrary.current.value === 'ts' }

```typescript
import { createClient } from '@ember-link/core';

const client = createClient({
	baseUrl: 'http://localhost:9000'
});

const { channel } = client.joinChannel('test');

/**
 * Subscribe to every others presence updates.
 * The callback will be called if
 * someone else enters or leaves the channel
 * or when someones presence is updated
 */
channel.events.others.subscribe('join', (user) => {
	console.log('User join: ', user);
});

channel.events.others.subscribe('update', (user) => {
	console.log('User update: ', user);
});

channel.events.others.subscribe('leave', (user) => {
	console.log('User leave: ', user);
});
```

{/if}

{#if selectedLibrary.current.value === 'react' }

```tsx
import { ChannelProvider, EmberLinkProvider } from '@ember-link/react';

function App() {
	return (
		<EmberLinkProvider baseUrl="http://localhost:9000">
			<ChannelProvider channelName="test" options={{}}>
				<Page />
			</ChannelProvider>
		</EmberLinkProvider>
	);
}

function Page() {
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
}
```

{/if}

{#if selectedLibrary.current.value === 'svelte' }

```svelte
<script lang="ts">
	import '../app.css';
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';

	let { children } = $props();
</script>

<EmberLinkProvider baseUrl="http://localhost:9000">
	<ChannelProvider channelName="test">
		{@render children()}
	</ChannelProvider>
</EmberLinkProvider>
```

```svelte
<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';

	const channel = getChannelContext();
</script>

<div>
	{channel.myPresence}
	{#each channel.others as other (other.clientId)}
		{other.clientId}
	{/each}
</div>
```

{/if}

<!--
<div class="flex items-center justify-center">
  <CursorExample />
  <CursorExample />
</div> -->
