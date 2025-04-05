<script lang="ts">
	import Editor from '$lib/components/collaborative/editor.svelte';
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';
</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<ChannelProvider channelName="collab-docs">
		<Editor />
	</ChannelProvider>
</EmberLinkProvider>
