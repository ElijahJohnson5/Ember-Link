<script lang="ts">
	import CursorExample from '$lib/components/cursor/index.svelte';
	import { Button } from '$lib/components/ui/button';
</script>

<div class="flex h-full w-full flex-col">
	<div class="prose mx-auto w-full max-w-4xl px-6 pt-6 dark:prose-invert">
		<h1><strong>Live Cursors</strong></h1>
		<p>
			Two independent <code>ChannelProvider</code> instances are rendered side by side below. Both
			join the same channel name (<code>cursor-room</code>) on the same client, which means the
			Ember Link client borrows the channel twice and gives each provider its own ref-counted
			<code>leave</code> function. Moving your cursor in either panel broadcasts presence to the
			other panel, and the other panel renders that presence with a per-peer <code>requestAnimationFrame</code>
			lerp loop on top of the SDK's <code>presenceThrottle</code> option, for smooth visual motion at a
			low on-wire send rate.
		</p>
		<p>
			Open this page in two browser windows to collaborate with yourself across tabs. See <a
				href="/concepts/presence">Concepts → Presence</a
			> for the throttling and lifecycle details that the demo exercises, and <a
				href="/concepts/channels">Concepts → Channels</a
			> for the ref-counting behavior shown by the two providers.
		</p>
	</div>

	<div>
		<Button
			variant="link"
			class="px-6 pt-2"
			target="_blank"
			href="https://github.com/ElijahJohnson5/Ember-Link/tree/main/apps/docs/src/lib/components/cursor"
		>
			View the code here
		</Button>
	</div>

	<div class="flex w-full flex-grow justify-center p-6 pt-2">
		<div
			class="flex w-full flex-col items-center justify-center divide-y-2 border-2 md:flex-row md:divide-x-2 md:divide-y-0"
		>
			<CursorExample heightClass="h-full" />
			<CursorExample heightClass="h-full" />
		</div>
	</div>
</div>
