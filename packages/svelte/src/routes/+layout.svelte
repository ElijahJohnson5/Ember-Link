<script lang="ts">
	import ChannelProvider from '$lib/channel-provider.svelte';
	import EmberLinkProvider from '$lib/ember-link-provider.svelte';
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<ChannelProvider channelName="test">
		{@render children()}
	</ChannelProvider>
</EmberLinkProvider>
