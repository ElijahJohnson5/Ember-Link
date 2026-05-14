<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';
	import { onDestroy } from 'svelte';
	import { Badge } from '$lib/components/ui/badge';
	import RemoteCursor from './remote-cursor.svelte';

	const channel = getChannelContext();

	onDestroy(() => {
		channel.updatePresence({
			cursor: null
		});
	});
</script>

<div
	class="page relative flex h-48 items-center justify-center md:h-60 lg:h-96"
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
		{#each channel.others as other (other.clientId)}
			{#if other.cursor}
				<RemoteCursor clientId={other.clientId} target={other.cursor} />
			{/if}
		{/each}
	</div>
</div>

<style>
	.page {
		width: 100%;
		height: 100%;
		position: relative;
	}

	.cursorsContainer {
		position: absolute;
		top: 0;
		left: 0;
	}
</style>
