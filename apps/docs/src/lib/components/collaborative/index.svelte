<script lang="ts">
	import Editor from '$lib/components/collaborative/editor.svelte';
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';
</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<!--
		Fresh channel name so we start from an empty Y.Doc on the
		sandbox. The previous `collab-docs` channel had accumulated
		junk content from earlier test sessions.
	-->
	<ChannelProvider channelName="collab-docs-v2">
		<Editor />
	</ChannelProvider>
</EmberLinkProvider>
