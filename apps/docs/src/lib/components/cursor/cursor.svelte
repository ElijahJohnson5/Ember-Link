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
	class="page flex h-48 items-center justify-center md:h-60 lg:h-96"
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
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
					fill="none"
					stroke={COLORS[Math.floor(Math.random() * COLORS.length)]}
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					style={`transform: translateX(${other.cursor.x}px) translateY(${other.cursor.y}px);`}
					class="cursor lucide lucide-mouse-pointer2-icon lucide-mouse-pointer-2"
				>
					<path
						d="M4.037 4.688a.495.495 0 0 1 .651-.651l16 6.5a.5.5 0 0 1-.063.947l-6.124 1.58a2 2 0 0 0-1.438 1.435l-1.579 6.126a.5.5 0 0 1-.947.063z"
					/>
				</svg>
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
