<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';
	import { onDestroy } from 'svelte';
	import { Badge } from '$lib/components/ui/badge';
	import RemoteCursor from './remote-cursor.svelte';

	// Forwarded from the wrapping `<CursorExample />` so callers can choose
	// a fixed minimum (homepage hero) or `h-full` (the /cursors examples
	// page where the demo should fill the available container height).
	let { heightClass = 'min-h-48 md:min-h-60 lg:min-h-96' }: { heightClass?: string } =
		$props();

	const channel = getChannelContext();

	// Defensive dedup. The live sandbox occasionally surfaces the same
	// peer twice during reconnect bursts, and the keyed `{#each}` below
	// throws `each_key_duplicate` if two entries share a clientId. Keep
	// the latest payload for each id and discard earlier copies.
	const uniqueOthers = $derived.by(() => {
		const seen = new Map();
		for (const o of channel.others) seen.set(o.clientId, o);
		return Array.from(seen.values()) as typeof channel.others;
	});

	onDestroy(() => {
		channel.updatePresence({
			cursor: null
		});
	});
</script>

<div
	class="page relative flex w-full min-w-0 flex-1 items-center justify-center {heightClass}"
	onpointermove={(event) => {
		const rect = event.currentTarget.getBoundingClientRect();
		const x = event.clientX - rect.left;
		const y = event.clientY - rect.top;

		channel.updatePresence({
			cursor: { x: Math.round(x), y: Math.round(y) }
		});
	}}
	onpointerleave={() => {
		channel.updatePresence({
			cursor: null
		});
	}}
	ontouchstart={(event) => {
		const rect = event.currentTarget.getBoundingClientRect();
		const x = event.touches[0].clientX - rect.left;
		const y = event.touches[0].clientY - rect.top;

		channel.updatePresence({
			cursor: { x: Math.round(x), y: Math.round(y) }
		});
	}}
	ontouchmove={(event) => {
		const rect = event.currentTarget.getBoundingClientRect();
		const x = event.touches[0].clientX - rect.left;
		const y = event.touches[0].clientY - rect.top;

		channel.updatePresence({
			cursor: { x: Math.round(x), y: Math.round(y) }
		});
	}}
	ontouchend={() => {
		channel.updatePresence({
			cursor: null
		});
	}}
	ontouchcancel={() => {
		channel.updatePresence({
			cursor: null
		});
	}}
>
	<div class="absolute right-2 top-2">
		<Badge>
			{channel.status.toUpperCase()}
		</Badge>
	</div>

	<div class="flex select-none items-center justify-center text-center">
		{channel.myPresence && channel.myPresence.cursor
			? `${channel.myPresence.cursor.x} × ${channel.myPresence.cursor.y}`
			: 'Move your cursor to broadcast its position to other people in the Channel.'}
	</div>

	<div class="cursorsContainer">
		{#each uniqueOthers as other (other.clientId)}
			{#if other.cursor}
				<RemoteCursor clientId={other.clientId} target={other.cursor} />
			{/if}
		{/each}
	</div>
</div>

<style>
	/*
		`.page` used to set width:100%, height:100%, position:relative.
		The 100% height beat the Tailwind h-48/h-60/h-96 utility classes
		(Svelte-scoped class adds specificity), which made the demo
		collapse to 0px on the homepage hero where the parent isn't a
		flex container with flex-grow. The Tailwind classes on the
		element itself already handle width / height / position, so the
		scoped rule is now redundant.
	*/

	.cursorsContainer {
		position: absolute;
		top: 0;
		left: 0;
	}
</style>
