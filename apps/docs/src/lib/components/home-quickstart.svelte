<script lang="ts">
	import LibrarySelectorTabs from '$lib/components/library-selector-tabs.svelte';

	// Plain .svelte (no mdsvex). The snippets below render simple
	// <pre><code> blocks — we lose Shiki highlighting here for the sake
	// of having reactive tabs that actually work. (Shiki via mdsvex
	// kept colliding with bits-ui's controlled mode for tabs.)
	//
	// Reusing the project-wide LibrarySelectorTabs component so the
	// look + behavior is consistent with the rest of the site, and so
	// a single change to the styled chrome flows everywhere.
</script>

<LibrarySelectorTabs>
	{#snippet js()}
		<pre
			class="overflow-x-auto bg-shiki-background p-4 text-sm leading-relaxed"><code>{`import { createClient } from '@ember-link/core';

const client = createClient({ baseUrl: 'http://localhost:9000' });
const { channel } = client.joinChannel('demo', {
  // Coalesce presence sends so a 120Hz pointermove
  // becomes ~30 messages/sec on the wire.
  presenceThrottle: 33
});

channel.events.others.subscribe('update', (user) => {
  console.log('cursor:', user.cursor);
});

channel.updatePresence({ cursor: { x: 0, y: 0 } });`}</code></pre>
	{/snippet}

	{#snippet ts()}
		<pre
			class="overflow-x-auto bg-shiki-background p-4 text-sm leading-relaxed"><code>{`import { createClient } from '@ember-link/core';

declare global {
  interface EmberLink {
    // Augment once; every API picks up your types.
    Presence: { cursor: { x: number; y: number } | null };
  }
}

const client = createClient({ baseUrl: 'http://localhost:9000' });
const { channel } = client.joinChannel('demo', { presenceThrottle: 33 });

channel.events.others.subscribe('update', (user) => {
  // user.cursor is { x: number; y: number } | null
});`}</code></pre>
	{/snippet}

	{#snippet react()}
		<pre
			class="overflow-x-auto bg-shiki-background p-4 text-sm leading-relaxed"><code>{`import {
  EmberLinkProvider,
  ChannelProvider,
  useOthers,
  useMyPresence
} from '@ember-link/react';

export default function App() {
  return (
    <EmberLinkProvider baseUrl="http://localhost:9000">
      <ChannelProvider channelName="demo" options={{ presenceThrottle: 33 }}>
        <Page />
      </ChannelProvider>
    </EmberLinkProvider>
  );
}

function Page() {
  const others = useOthers();
  const [me, setMe] = useMyPresence();

  return (
    <div onPointerMove={(e) => setMe({ cursor: { x: e.clientX, y: e.clientY } })}>
      {others.length} other peer(s)
    </div>
  );
}`}</code></pre>
	{/snippet}

	{#snippet svelte()}
		<pre
			class="overflow-x-auto bg-shiki-background p-4 text-sm leading-relaxed"><code>{`<!-- +layout.svelte -->
<` + `script lang="ts">
  import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';
  let { children } = $props();
<` + `/script>

<EmberLinkProvider baseUrl="http://localhost:9000">
  <ChannelProvider channelName="demo" presenceThrottle={33}>
    {@render children()}
  </ChannelProvider>
</EmberLinkProvider>

<!-- +page.svelte -->
<` + `script lang="ts">
  import { getChannelContext } from '@ember-link/svelte';
  const channel = getChannelContext();
<` + `/script>

<div
  onpointermove={(e) =>
    channel.updatePresence({ cursor: { x: e.clientX, y: e.clientY } })}
>
  {channel.others.length} other peer(s)
</div>`}</code></pre>
	{/snippet}
</LibrarySelectorTabs>
