<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';
	import { getChannelContext } from '@ember-link/svelte';

	const COLORS = ['#DC2626', '#D97706', '#059669', '#7C3AED', '#DB2777'];

	let cursorPosition = $state<{ x: number; y: number } | null>(null);
	let otherCursors = new SvelteMap<string, { x: number; y: number } | null>();	

	const channel = getChannelContext("test");

  channel.events.subscribe('presence', (presence) => {
    cursorPosition = presence?.cursor ?? null;
  });

  channel.events.others.subscribe('join', (user) => {
    otherCursors.set(user.clientId, user.cursor);
  });

  channel.events.others.subscribe('update', (user) => {
    otherCursors.set(user.clientId, user.cursor);
  });

  channel.events.others.subscribe('leave', (user) => {
    otherCursors.delete(user.clientId);
  });
</script>

<svelte:document
	onpointermove={(event) => {
    channel.updatePresence({
      cursor: { x: Math.round(event.clientX), y: Math.round(event.clientY) }
    });
	}}
	onpointerleave={() => {
    channel.updatePresence({
      cursor: null
    });
	}}
/>

<div class="wholePage">
	<div>
		{cursorPosition
			? `${cursorPosition.x} × ${cursorPosition.y}`
			: 'Move your cursor to broadcast its position to other people in the Channel.'}
	</div>

	<div class="cursorsContainer">
		{#each otherCursors as [id, cursor]}
			{#if cursor}
				<svg
					class="cursor"
					id={`cursor-${id}`}
					width="24"
					height="36"
					viewBox="0 0 24 36"
					style={`transform: translateX(${cursor.x}px) translateY(${cursor.y}px);`}
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
	.wholePage {
		height: 100%;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	.cursor {
		position: absolute;
		top: 0;
		left: 0;
	}

	.cursorsContainer {
		position: absolute;
		top: 0;
		left: 0;
		width: 100vw;
		height: 100vh;
	}
</style>
