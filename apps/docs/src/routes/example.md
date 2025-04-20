<script lang="ts">
  import { selectedLibrary } from '$lib/library.svelte';
  import CursorExample from '$lib/components/cursor/index.svelte';
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';

</script>

<LibrarySelectorTabs>
{#snippet js()}

```typescript copyButton
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

{/snippet}

{#snippet ts()}

```typescript copyButton
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

{/snippet}

{#snippet react()}

```tsx copyButton
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

{/snippet}

{#snippet svelte()}

```svelte copyButton
<!-- +layout.tsx -->

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

```svelte copyButton
<!-- +page.tsx -->

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

{/snippet}

</LibrarySelectorTabs>
