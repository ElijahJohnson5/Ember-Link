<script lang="ts" module>
	declare global {
		interface EmberLink {
			Presence: {
				cursor: {
					x: number;
					y: number;
				} | null;
			};
		}
	}
</script>

<script lang="ts">
	import { getChannelContext } from '@ember-link/svelte';
	import { onDestroy } from 'svelte';

	const COLORS = ['#DC2626', '#D97706', '#059669', '#7C3AED', '#DB2777'];

	const channel = getChannelContext();

	onDestroy(() => {
		channel.updatePresence({
			cursor: null
		});
	});
</script>

<div
	class="page flex items-center justify-center"
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
			cursor: { x: 100, y: 100 }
		});
	}}
>
	<div class="flex items-center justify-center text-center">
		{channel.myPresence && channel.myPresence.cursor
			? `${channel.myPresence.cursor.x} × ${channel.myPresence.cursor.y}`
			: 'Move your cursor to broadcast its position to other people in the Channel.'}
	</div>

	<div class="cursorsContainer">
		{#each channel.others as other (other.clientId)}
			{#if other.cursor}
				<svg
					class="cursor"
					id={`cursor-${other.clientId}`}
					width="24"
					height="36"
					viewBox="0 0 24 36"
					style={`transform: translateX(${other.cursor.x}px) translateY(${other.cursor.y}px);`}
					fill={COLORS[Math.floor(Math.random() * COLORS.length)]}
					xmlns="http://www.w3.org/2000/svg"
				>
					<path
						d="M5.65376 12.3673H5.46026L5.31717 12.4976L0.500002 16.8829L0.500002 1.19841L11.7841 12.3673H5.65376Z"
					/>
				</svg>
			{/if}
		{/each}
	</div>
</div>

<style>
	.page {
		height: 200px;
		width: 200px;
		position: relative;
	}

	.cursor {
		top: 0;
		left: 0;
	}

	.cursorsContainer {
		position: absolute;
		top: 0;
		left: 0;
	}
</style>
