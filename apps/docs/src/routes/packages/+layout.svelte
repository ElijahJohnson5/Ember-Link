<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children }: { children: Snippet } = $props();
</script>

<div class="prose dark:prose-invert mx-auto w-full max-w-4xl p-6">
	{@render children()}
</div>
