<script lang="ts">
	import Cursor from '$lib/components/cursor/cursor.svelte';
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';

</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<!--
		presenceThrottle: ~30Hz cap on presence sends. Coalesces every
		pointermove into a single message per 33ms window — works hand-in-
		hand with the rAF lerp in <RemoteCursor> to give Figma-style smooth
		motion without flooding the WebSocket.
	-->
	<ChannelProvider channelName="cursor-docs" presenceThrottle={33}>
		<Cursor />
	</ChannelProvider>
</EmberLinkProvider>
