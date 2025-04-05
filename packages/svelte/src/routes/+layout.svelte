<script lang="ts">
	import { ChannelProvider } from '$lib/index.js';
	import EmberLinkProvider from '$lib/ember-link-provider.svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<ChannelProvider channelName="123">
		{@render children()}
	</ChannelProvider>
</EmberLinkProvider>
