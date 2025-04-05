<script lang="ts">
	import CursorExample from '$lib/components/cursor/index.svelte';
</script>

<div class="flex w-full justify-center p-6">
	<div
		class="flex w-full flex-col items-center justify-center divide-y-2 border-2 md:flex-row md:divide-x-2 md:divide-y-0"
	>
		<CursorExample />
		<CursorExample />
	</div>
</div>
