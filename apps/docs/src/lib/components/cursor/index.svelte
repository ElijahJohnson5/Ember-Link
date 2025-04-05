<script lang="ts">
	import Cursor from '$lib/components/cursor/cursor.svelte';
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';

</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<ChannelProvider channelName="cursor-docs">
		<Cursor />
	</ChannelProvider>
</EmberLinkProvider>
