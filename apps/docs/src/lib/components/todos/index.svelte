<script lang="ts">
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';
	import { createYJSStorageProvider } from '@ember-link/yjs-storage';
	import Todos from './todos.svelte';
</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<ChannelProvider
		channelName="todos-docs"
		storageProvider={createYJSStorageProvider()}
		initialPresence={{ isTyping: false }}
	>
		<Todos />
	</ChannelProvider>
</EmberLinkProvider>
