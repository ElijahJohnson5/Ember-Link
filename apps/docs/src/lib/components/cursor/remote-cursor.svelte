<script lang="ts" module>
	/**
	 * Stable color palette. We deterministically pick one entry per
	 * client id (rather than `Math.random()`) so the same peer's cursor
	 * always renders the same color — and doesn't flicker between
	 * colors on each render of the parent.
	 */
	const COLORS = ['#DC2626', '#D97706', '#059669', '#7C3AED', '#DB2777'];

	function colorForId(id: string): string {
		let hash = 0;
		for (let i = 0; i < id.length; i++) {
			hash = (hash * 31 + id.charCodeAt(i)) >>> 0;
		}
		return COLORS[hash % COLORS.length];
	}

	/**
	 * Lerp factor per animation frame. 0.25 catches up within ~4-5 frames
	 * (~70ms @ 60Hz). Higher = snappier, lower = smoother but laggier.
	 * Pairs with the 33ms presenceThrottle on the channel to give
	 * fluid motion without flooding the WebSocket.
	 */
	const SMOOTHING = 0.25;
</script>

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';

	// Don't destructure — `target` needs to be read reactively from
	// inside the rAF tick so we always lerp toward the latest position.
	const props: {
		clientId: string;
		target: { x: number; y: number };
	} = $props();

	let element: SVGSVGElement | undefined = $state();

	// Rendered position. Non-reactive so updates from the rAF loop
	// don't trigger Svelte re-renders — we just imperatively set
	// `element.style.transform` each frame.
	let rx = props.target.x;
	let ry = props.target.y;
	let rafId = 0;

	function applyTransform() {
		if (element) {
			element.style.transform = `translate3d(${rx}px, ${ry}px, 0)`;
		}
	}

	function tick() {
		const tx = props.target.x;
		const ty = props.target.y;
		const dx = tx - rx;
		const dy = ty - ry;

		if (Math.abs(dx) < 0.1 && Math.abs(dy) < 0.1) {
			// Within sub-pixel of target. Snap once so the GPU layer can settle.
			if (rx !== tx || ry !== ty) {
				rx = tx;
				ry = ty;
				applyTransform();
			}
		} else {
			rx += dx * SMOOTHING;
			ry += dy * SMOOTHING;
			applyTransform();
		}

		rafId = requestAnimationFrame(tick);
	}

	onMount(() => {
		applyTransform();
		rafId = requestAnimationFrame(tick);
	});

	onDestroy(() => {
		cancelAnimationFrame(rafId);
	});
</script>

<svg
	bind:this={element}
	xmlns="http://www.w3.org/2000/svg"
	width="24"
	height="24"
	viewBox="0 0 24 24"
	fill="none"
	stroke={colorForId(props.clientId)}
	stroke-width="2"
	stroke-linecap="round"
	stroke-linejoin="round"
	class="cursor lucide lucide-mouse-pointer2-icon lucide-mouse-pointer-2"
>
	<path
		d="M4.037 4.688a.495.495 0 0 1 .651-.651l16 6.5a.5.5 0 0 1-.063.947l-6.124 1.58a2 2 0 0 0-1.438 1.435l-1.579 6.126a.5.5 0 0 1-.947.063z"
	/>
</svg>

<style>
	.cursor {
		position: absolute;
		top: 0;
		left: 0;
		/* JS drives the transform via rAF lerp — `will-change` puts the
		   cursor on its own compositor layer so each frame is GPU-only. */
		will-change: transform;
		pointer-events: none;
	}
</style>
