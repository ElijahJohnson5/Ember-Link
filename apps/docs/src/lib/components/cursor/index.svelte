<script lang="ts">
	import Cursor from '$lib/components/cursor/cursor.svelte';
	import { EmberLinkProvider, ChannelProvider } from '@ember-link/svelte';

	// `heightClass` is a Tailwind height class applied to the cursor demo's
	// root div. Default gives a sensible minimum on pages where the parent
	// has no defined height (the homepage hero). Pass `h-full` on pages
	// where the demo should fill the available vertical space (the /cursors
	// example page).
	let { heightClass = 'min-h-48 md:min-h-60 lg:min-h-96' }: { heightClass?: string } =
		$props();
</script>

<EmberLinkProvider baseUrl="https://ember-link-sandbox.onrender.com">
	<!--
		presenceThrottle: ~30Hz cap on presence sends. Coalesces every
		pointermove into a single message per 33ms window. Works hand in
		hand with the rAF lerp in <RemoteCursor> to give Figma-style smooth
		motion without flooding the WebSocket.
	-->
	<ChannelProvider channelName="cursor-docs" presenceThrottle={33}>
		<Cursor {heightClass} />
	</ChannelProvider>
</EmberLinkProvider>
